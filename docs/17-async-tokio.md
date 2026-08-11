# 17 – Async mit `tokio`: select, JoinSet, Timeout, Registry

## Ziel

Dieselbe Server-Struktur wie in Lektion 16 — aber async. Die zwei
Umständlichkeiten der sync-Lösung fallen weg: **kein Polling** mehr (Loop
wartet gleichzeitig auf `accept` *und* Shutdown), und Tasks sind **kooperativ
abbrechbar** (was Threads nicht sind). Der reine Kern (`handle`, `to_http`,
`parse_request`) bleibt unverändert — nur die I/O-Schale wird async.

Vier Etappen, jede testgetrieben und einzeln getaggt:

1. **4a** async-Port + minimaler Shutdown (`tokio::select!`) — `lesson-17-async`
2. **4b** voll-graceful via `JoinSet` — `lesson-17-joinset`
3. **4c** per-Verbindung-Timeout (Slow-Loris) — `lesson-17-timeout`
4. **4d** Registry: eine Verbindung gezielt abbrechen — `lesson-17-registry`

## Was async überhaupt ist (kurz)

Ein `Future` ist ein Wert, der *noch nicht fertig* ist — er tut nichts, bis er
**gepollt** wird. `.await` sagt: „poll dieses Future; ist es nicht fertig, gib
den Thread frei, damit andere Arbeit laufen kann." Genau das ist der Gewinn: ein
`read().await`, das auf Daten wartet, **blockiert den Thread nicht** (anders als
`std`-`read`, das den ganzen OS-Thread festhält). Ein **Task** (`tokio::spawn`)
ist eine grün-gescheduelte Einheit — ~KB statt ~MiB pro OS-Thread, deshalb
tausende gleichzeitig möglich.

`#[tokio::main]` / `#[tokio::test]` sind Makros, die eine Runtime aufsetzen und
`block_on(async { ... })` um deinen Code legen. `tokio::net::TcpListener`/
`TcpStream` sind *eigene* Typen (nicht `std::net`), und `.await`-basiertes Lesen/
Schreiben braucht die Traits `AsyncReadExt`/`AsyncWriteExt` im Scope.

## 4a — `tokio::select!` ersetzt das ganze Polling-Konstrukt

In L16 brauchte der stoppbare Loop `set_nonblocking` + `WouldBlock`-Polling +
`sleep(50ms)`, weil `accept()` blockierte und das Flag sonst nie geprüft würde.
Async macht das überflüssig:

```rust
tokio::pin!(shutdown);
loop {
    tokio::select! {
        result = listener.accept() => { /* neue Verbindung */ }
        _ = &mut shutdown => { break; }   // Shutdown-Signal
    }
}
```

`select!` pollt **beide** Zweige gleichzeitig und nimmt den, der zuerst fertig
wird. Kein Polling, keine Leerlauf-Latenz, kein `sleep`. `serve` hängt fast immer
im `accept().await` — kommt das Signal, greift der `shutdown`-Zweig.

### `tokio::pin!(shutdown)` — warum nötig?

`select!` muss `shutdown` über *viele* Loop-Runden **wiederholt pollen**
(`&mut shutdown`). Pollen geht nur über `Pin<&mut Future>` (die Signatur von
`Future::poll` verlangt `self: Pin<&mut Self>`). Grund: ein `async`-Block ist ein
Zustandsautomat, der sich **selbst referenzieren** kann; würde man ihn nach dem
ersten Poll im Speicher verschieben, zeigten interne Zeiger ins Leere → UB.
`Pin` ist die Typ-Garantie „wird nicht mehr bewegt". `shutdown` kommt per Wert
frei beweglich rein; `tokio::pin!` shadowt es zu einer an den Stack gepinnten
Variante → `&mut shutdown` wird ein gültiger, wiederholt pollbarer `Pin<&mut _>`.

> Merksatz: `listener.accept()` ist frisch pro Runde → kein Pin nötig.
> `shutdown` überlebt alle Runden und wird wiederholt gepollt → Pin nötig.

### Shutdown ist jetzt ein *Future*, kein geteiltes Flag

L16 nutzte `Arc<AtomicBool>` — geteilter Zustand, den ein anderer Thread
*kippt* (deshalb `Atomic` + `Ordering`). Jetzt besitzt `serve` das `shutdown`-
Future **allein**; niemand sonst hat einen Zugriffspfad darauf. Es wird nicht
„geändert", es **wird fertig** (`Ready`). *Wodurch* — Ctrl-C, Timer, ein
`oneshot`-Sender — kapselt die jeweilige Future; die Synchronisation steckt
darin, nicht in unserem Code. Die ganze Fehlerklasse „Flag falsch synchronisiert,
Signal geht verloren" fällt damit weg. In `main` ist das Signal
`tokio::signal::ctrl_c()`, per DI reingereicht:

```rust
let shutdown = async { let _ = tokio::signal::ctrl_c().await; };
serve(&listener, shutdown, Duration::from_secs(30), registry).await?;
```

## 4b — voll-graceful via `JoinSet`

Der minimale Loop aus 4a *detached* die Verbindungs-Tasks (`tokio::spawn`,
Handle weggeworfen) → beim `break` kehrt `serve` sofort zurück, laufende
Antworten würden beim Prozess-Ende abgeschnitten. Das ist der async-Zwilling der
L16-Lücke vor dem `join()`-Loop.

`tokio::task::JoinSet` ist der async-Ersatz für `Vec<JoinHandle>` + `join()`:

```rust
let mut tasks = JoinSet::new();
// im accept-Zweig:  tasks.spawn(async move { ... });
// nach dem loop:
tasks.join_all().await;   // wartet auf ALLE laufenden Tasks, DANN zurück
```

`join_all().await` gibt den Thread frei, während es wartet (kein Blockieren) —
`serve` kehrt erst zurück, wenn die letzte in-flight-Verbindung fertig ist.
Bewiesen über **Zeitmessung** (`serve_waits_for_inflight_connection`): Client
ruft `/api/slow` (500 ms), Shutdown nach 100 ms; detached → Rückkehr ~100 ms
(rot), `JoinSet` → ~500 ms (grün). „200 empfangen" allein würde *fälschlich*
grün, weil ein detached Task seine Antwort noch liefert solange der Testprozess
lebt — nur das *Timing* unterscheidet detached von graceful.

Nebeneffekt: ein gedroppter `JoinSet` **abortet** alle enthaltenen Tasks — bei
einem frühen `?`-Ausstieg verwaist so keine Verbindung.

## 4c — per-Verbindung-Timeout (Slow-Loris-Abwehr)

`join_all().await` wartet **unbegrenzt** — eine Verbindung, die ewig im
`read().await` hängt (Slow-Loris), würde den Shutdown für immer blockieren.
🔒 Das ist eine echte DoS-Fläche. In L16 half `set_read_timeout`; das existiert
auf `tokio::net::TcpStream` nicht. Der async-Ersatz ist `tokio::time::timeout`:

```rust
match tokio::time::timeout(timeout, handle_connection(stream)).await {
    Ok(Ok(())) => {}
    Ok(Err(e)) => tracing::warn!(error = %e, "connection failed"),
    Err(_elapsed) => tracing::warn!("connection timed out"), // Frist ab → Future gedroppt
}
```

Läuft die Frist ab, wird das `handle_connection`-Future **gedroppt** → der
`stream` schließt → der hängende `read` endet. Der Timeout ist ein
**DI-Parameter** (wie `shutdown`): der Test injiziert `100ms`, `main`
`30s`. Ein hartkodierter Wert wäre nicht in <1 s testbar — und `main` mit `100ms`
würde jede reale Verbindung töten. Bewiesen über `serve_one`
(`serve_one_drops_a_silent_connection_after_timeout`): ein stiller Client wird
nach der Frist fallengelassen statt ewig zu hängen.

## 4d — Registry: eine Verbindung gezielt abbrechen

Ziel: *eine bestimmte* Verbindung von außen beenden, ohne die anderen zu
stoppen — das Task-Gegenstück zum `thread.kill()`, das Rust bewusst **nicht**
hat. `tasks.spawn(...)` gibt einen `AbortHandle`; der landet in einer Registry:

```rust
pub type Registry = Arc<Mutex<HashMap<u64, AbortHandle>>>;

let id = next_id; next_id += 1;
let reg = Arc::clone(&registry);
let handle = tasks.spawn(async move {
    handle_with_timeout(stream, timeout).await;
    reg.lock().unwrap().remove(&id);          // self-deregister beim Ende
});
registry.lock().unwrap().insert(id, handle);  // AbortHandle merken
```

Von außen: `registry.lock().unwrap().get(&id).abort()` → nur *dieser* Task
endet. Vier Punkte:

- **`AbortHandle` statt `JoinHandle`**: `JoinSet` behält den `JoinHandle` (für
  graceful-shutdown aus 4b), die Registry braucht nur den leichten `AbortHandle`
  zum Killen. Beides koexistiert sauber.
- **Self-deregister** (`remove` am Task-Ende): sonst wächst die Map unbegrenzt —
  🔒 ein Angreifer, der Verbindungen auf-/zumacht, sprengt sonst den Speicher.
- **`std::sync::Mutex`, nicht `tokio::sync::Mutex`**: erlaubt, *weil* wir den Lock
  nie über ein `.await` halten — locken, eine Map-Operation, Guard fällt sofort.
  (Über ein `.await` gehaltener Lock würde Tasks verklemmen → dann bräuchte es
  den async-Mutex.)
- **`next_id` braucht kein Atomic**: nur der eine Loop vergibt IDs; geteilt ist
  nur die *Map*, nicht der Zähler.

Bewiesen (`can_abort_a_specific_connection`): Client A hängt (registriert),
Client B kriegt trotzdem `200` (Server + andere laufen weiter), dann gezielt
`handle.abort()` → nur A endet (`is_finished()`).

## Ehrliche Grenze: `abort()` ist *kooperativ*, kein `kill -9`

Diese Frage kam beim Bauen auf und ist wichtig genug für eine eigene Notiz:
**kann ich mit `abort()` einen Task stoppen, der gerade etwas Schädliches tut —
z. B. ein `rm -rf` aufruft?** Antwort: **Nein.**

`abort()` setzt nur ein Flag „bitte beim nächsten `.await` aufhören". Der Task
wird an einem **Yield-Punkt** abgebrochen. Läuft er gerade in synchronem Code
*ohne* `.await` (ein blockierender `Command::status()`, oder unser
`std::thread::sleep` in `/api/slow`), gibt es keinen Yield-Punkt → der Task läuft
stur weiter, bis der Block fertig ist. `abort`, `AtomicBool`-Flag,
`CancellationToken` — **alle kooperativ**: sie brauchen Code, der freiwillig
nachschaut. Auch Threads haben bewusst kein `kill()` — mitten-drin-Abbrechen
hinterließe kaputte Locks, halb geschriebene Dateien (UB). Das kennen wir aus L15.

Was man **stattdessen** tut, nach Wirksamkeit:

1. **Verhindern statt abbrechen** (die eigentliche Antwort): die Fähigkeit,
   Schaden anzurichten, gar nicht erst in den Request-Pfad lassen. Kein
   Shell-Aufruf aus ungeprüftem Input; Input-Validierung an der Grenze;
   Allowlists. Ein gut designter Handler *kann* nichts Schädliches.
2. **Kooperative Yield-Punkte einbauen**, wenn es *deine* lange Arbeit ist:
   `if token.is_cancelled() { return; }` in der Schleife; `tokio_util`s
   `CancellationToken` ist das saubere Werkzeug (der Task *weiß*, dass er
   abbricht, und kann aufräumen).
3. **Isolation** als einzige *echte* präemptive Grenze: riskante Arbeit als
   eigener Kindprozess (`tokio::process::Command` → `child.kill()` per OS-Signal)
   oder Sandbox/Container mit CPU-/Zeit-/FS-Limits. Das ist das `kill -9`, das es
   *innerhalb* eines Prozesses nicht gibt.

Für unser Projekt bleibt die Registry sinnvoll — für den **legitimen** Fall: eine
Verbindung, die in `read().await` hängt (idle/Slow-Loris), lässt sich prompt
kicken, weil sie an einem echten Yield-Punkt wartet. Das ist der Löwenanteil
realer „kick this connection"-Fälle.

> Genau hier wird **Memory Safety ≠ Application Security** konkret: Rust
> verhindert Speicherfehler, aber ob dein Server sich selbst löschen kann,
> entscheidet dein *Design*, nicht die Sprache. Dieses Lernprojekt ist kein
> auditierter, produktionssicherer Server.

## `#[tokio::test]` — single- vs. multi-thread

Standard-`#[tokio::test]` läuft **single-threaded**. Ein Test, der `serve`
spawnt *und* mit einem blockierenden `std`-Client im Testkörper wartet, würde
deadlocken: der blockierende Client hält den einzigen Runtime-Thread, der
`serve`-Task wird nie gepollt. Fix: `#[tokio::test(flavor = "multi_thread")]`.
Clients laufen bewusst als `std::thread` mit blockierendem `std::net::TcpStream`
(nicht `tokio`), damit sie keinen Runtime-Worker belegen. `std::future::pending::
<()>()` ist ein nie-fertig-Future — ein endloser Server ohne Shutdown, für die
Verbindungs-Tests.

## DRY: `tests/common/mod.rs`

Der `get_health`-Client-Helfer war in beiden Integrationstest-Dateien dupliziert.
Jede Datei direkt in `tests/` ist ein **eigenes Crate** — sie können sich nicht
gegenseitig importieren. Das Idiom: `tests/common/mod.rs` (ein *Unterordner* mit
`mod.rs`, nicht `common.rs` — sonst liefe es als eigenes „0 tests"-Crate), in
jede Datei per `mod common;` + `use common::get_health;` eingehängt. Geteilter
*Code*, aber keine Laufzeit-Kopplung: `get_health` ist eine reine Funktion.

## Getestet

- `serve_returns_when_shutdown_future_completes` — `serve` kehrt zurück, wenn das
  Shutdown-Future fertig wird.
- `serve_waits_for_inflight_connection` — `serve` wartet die laufende Verbindung
  aus (`elapsed >= 450 ms`, Client bekommt volle Antwort) — 4b.
- `serve_one_drops_a_silent_connection_after_timeout` — stiller Client wird nach
  der Frist fallengelassen statt zu hängen — 4c.
- `can_abort_a_specific_connection` — eine Verbindung gezielt abbrechen, während
  Server + andere weiterlaufen — 4d.
- `serve_one_*`, `serve_handles_multiple_connections`,
  `slow_connection_does_not_block_others` — async-Ports der L14–16-Tests.
