# 16 – Graceful Shutdown (sync) + per-Verbindung-Logging

## Ziel

`serve` lief endlos und ließ sich nur hart abwürgen. Jetzt: sauber stoppbar
und **graceful** — nach dem Stopp-Signal werden keine neuen Verbindungen mehr
angenommen, aber die *laufenden* zu Ende bedient, bevor `serve` zurückkehrt.
Dazu per-Verbindung-Logging über einen `tracing`-Span.

## Zwei Etappen (jede testgetrieben)

1. **Stoppbarer Loop** — `serve(&TcpListener, Arc<AtomicBool>)`. Der Test
   beweist: `serve` kehrt zurück, nachdem das Flag gesetzt wurde.
2. **Voll-graceful** — laufende Verbindungs-Threads werden vor der Rückkehr
   abgewartet. Der Test misst: `serve` kehrt erst *nach* dem in-flight-Handler
   zurück.

## Warum ein Flag allein nicht reicht: `accept()` blockiert

Die naive Idee „Loop prüft ein Flag" scheitert: `incoming()`/`accept()`
**blockiert**, bis eine Verbindung kommt — das Flag würde erst nach der
nächsten Verbindung geprüft. Lösung: `set_nonblocking(true)` macht `accept()`
sofort zurückkehrend.

```rust
pub fn serve(listener: &TcpListener, flag: Arc<AtomicBool>) -> std::io::Result<()> {
    listener.set_nonblocking(true)?;
    let mut handles = Vec::new();
    while !flag.load(Ordering::SeqCst) {          // Flag prüfen VOR jedem accept
        match listener.accept() {
            Ok((stream, _addr)) => {
                let handle = std::thread::spawn(move || {
                    if let Err(e) = handle_connection(stream) {
                        tracing::warn!(error = %e, "connection failed");
                    }
                });
                handles.push(handle);             // Handle behalten statt wegwerfen
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));  // nichts da → poll
            }
            Err(e) => return Err(e),
        }
    }
    for handle in handles {
        let _ = handle.join();                    // in-flight-Verbindungen abwarten
    }
    Ok(())
}
```

## Erstes echtes Shared State: `Arc<AtomicBool>`

Zwei Threads teilen *ein* Flag: der aufrufende Thread schreibt (`store`), der
Server-Thread liest (`load`). `Arc` (Atomic Reference Counted) erlaubt
geteilten *Besitz* über Thread-Grenzen — eine `&`-Referenz ginge nicht, weil
der Compiler nicht garantieren kann, dass das Flag den Thread überlebt.
`Arc::clone` gibt einen zweiten Handle auf denselben Wert (Zähler +1).

`AtomicBool` statt `Mutex<bool>`: für ein einzelnes Flag ist ein Lock Overkill;
atomares `store`/`load` ist lock-frei. `Ordering::SeqCst` ist die stärkste und
einfachste Sichtbarkeits-Garantie („alle Threads sehen dieselbe Reihenfolge") —
für ein Shutdown-Flag die richtige, unkomplizierte Wahl.

## Voll-graceful: `JoinHandle`s sammeln + `join()`

`thread::spawn` gibt ein `JoinHandle` zurück — bisher weggeworfen. Jetzt landet
jedes in einem `Vec`. Nach dem Loop wartet `join()` der Reihe nach auf jeden
Thread → laufende Anfragen werden fertig bedient, *dann* kehrt `serve` zurück.
`let _ = handle.join()` ignoriert bewusst das `Result`: ein panickter
Verbindungs-Thread soll den Shutdown nicht mitreißen.

**Getestet über Zeitmessung** (`server_shutdown_test.rs`): ein Client ruft den
langsamen Endpunkt (`/api/slow`, 500 ms), das Flag wird nach 100 ms gesetzt,
dann `Instant`-Messung bis `serve` zurückkehrt. Detached kehrte `serve` nach
~8 ms zurück (roter Beweis); graceful wartet die restlichen ~400 ms →
`elapsed >= 300 ms`. Zusätzlich: der Client erhält seine *vollständige* Antwort.

## Read-Timeout: gegen hängende Verbindungen

`join()` wartet **unbegrenzt** — ein Handler, der ewig in `stream.read(...)`
hängt (Slow-Loris), würde den Shutdown blockieren. Fix in `handle_connection`:

```rust
stream.set_read_timeout(Some(Duration::from_secs(5)))?;
```

Danach terminiert jeder `read` in ≤ 5 s (Antwort *oder* `WouldBlock`-Fehler) →
`join()` kann nicht ewig hängen. 🔒 Das ist primär eine Security-Maßnahme:
Standard-Abwehr gegen Slow-Loris (Verbindungen offen halten, nie senden, Threads
binden). Fehler landet über `?` im `if let Err`-Zweig von `serve` → Thread endet
sauber, kein Hänger.

## per-Verbindung-Logging: `tracing`-Span

Lose `info!`-Zeilen verschränken sich bei parallelen Verbindungen unlesbar. Ein
**Span** umschließt eine Verbindung und hängt jedem Log darin seinen Kontext an:

```rust
let peer = stream.peer_addr()?;
let span = tracing::info_span!("connection", %peer);
let _guard = span.enter();     // aktiv bis Funktionsende (Drop von _guard)
```

- `%peer` = `Display`-Form der Adresse als Span-Feld; jedes Event im Span erbt es.
- **`let _guard`, nicht `let _`**: `let _ = span.enter()` würde den Guard sofort
  droppen → Span augenblicklich zu, wirkungslos. Derselbe `_name`-vs-`_`-Punkt
  wie beim `let _slow` im Slow-Loris-Test.
- Die drei Debug-`info!` aus der Entwicklung entfernt; geblieben ist eine
  strukturierte Zeile `info!(method = ?req.method, path = %req.path, "request")`.

## Ehrliche Grenzen (nicht kaschiert)

- **In `main` wird das Flag nie gesetzt** — es gibt keinen Signal-Handler.
  Strg-C beendet den Prozess weiterhin hart; der Graceful-Mechanismus ist
  implementiert und getestet, aber dort noch nicht verdrahtet. Echtes
  Signal-Handling (z. B. `ctrlc`-Crate) wäre ein eigener Schritt.
- ⚡ **Polling kostet**: der 50-ms-`sleep` weckt den Loop 20×/s im Leerlauf und
  fügt bis zu 50 ms Shutdown-Latenz hinzu. Die ehrliche Grenze der sync-Lösung.
- Der 5-s-Read-Timeout ist willkürlich und schützt nur gegen *völlig* stille
  Clients — ein Angreifer, der alle 4 s ein Byte schickt, setzt ihn zurück. Ein
  echtes Gesamt-Timeout pro Anfrage ist mit blockierendem `std`-I/O umständlich.

Beide Umständlichkeiten — Polling statt Warten-auf-mehreres, kein sauberer
Abbruch eines blockierenden `read` — motivieren **async** (Lektion 17):
`tokio::select!` wartet gleichzeitig auf `accept` und Shutdown-Signal ohne
Polling, und Tasks sind kooperativ abbrechbar.

## Getestet

- `serve_returns_after_shutdown_signal` — `serve` terminiert nach dem Signal
  (`join()` des Server-Threads blockiert nicht ewig).
- `serve_waits_for_inflight_connection` — `serve` wartet die laufende Verbindung
  aus (`elapsed >= 300 ms`, Client bekommt volle Antwort).
- `slow_endpoint_returns_ok_after_delay` (in `http_test.rs`) — der langsame
  Endpunkt existiert und braucht ~500 ms.
