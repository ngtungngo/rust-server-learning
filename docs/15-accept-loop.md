# 15 – Accept-Loop + Thread pro Verbindung

## Ziel

`serve_one` bediente genau eine Verbindung und kehrte zurück. Jetzt ein
Server, der dauerhaft läuft und mehrere Verbindungen bedient — auch
gleichzeitig. Zwei Grenzen wurden nacheinander testgetrieben aufgelöst.

## Der Weg in drei Stufen (jede von einem roten Test erzwungen)

1. **`serve_one`** (Lektion 14): eine Verbindung, dann Rückkehr.
2. **Loop** (`serve` mit `incoming()`): mehrere Verbindungen *nacheinander*.
3. **Thread pro Verbindung**: mehrere *gleichzeitig*.

Kein Code auf Verdacht: erst der Test, der die jeweilige Grenze zeigt, dann
die kleinste Erweiterung, die ihn grün macht.

## `handle_connection` extrahieren

Die Pro-Verbindung-Logik (lesen, parsen, handeln, schreiben) wandert aus
`serve_one` in eine eigene Funktion:

```rust
fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> { ... }
```

`stream` **by value**, nicht `&mut`: Wenn `handle_connection` endet, wird
`stream` gedroppt → der Socket schließt → der Client bekommt EOF und sein
`read_to_string` kehrt zurück. Ownership erledigt das Ressourcen-Management
(RAII); niemand muss ans Schließen denken.

`serve_one` bleibt als dünner Einmal-Bediener für die terminierbaren Tests
aus Lektion 14 erhalten.

## Der Accept-Loop

```rust
pub fn serve(listener: &TcpListener) -> std::io::Result<()> {
    for stream in listener.incoming() {
        let stream = stream?;
        std::thread::spawn(move || {
            if let Err(e) = handle_connection(stream) {
                tracing::warn!(error = %e, "connection failed");
            }
        });
    }
    Ok(())
}
```

- **`incoming()`** ist ein Iterator; sein `next()` blockiert bis eine
  Verbindung kommt und endet nie → der `for`-Loop läuft für immer. Ein Server
  lebt, bis man ihn beendet.
- **`stream?`** auf Accept-Ebene: ein kaputter Listener ist ernst, darf `serve`
  verlassen.
- **`if let Err` statt `?`** *innerhalb* der Verbindung: ein Fehler bei *einer*
  Verbindung wird geloggt, der Loop macht weiter. 🔒 Fehler-Isolation — ein
  bösartiger oder kaputter Client darf den Dienst für alle anderen nicht töten.
  Im Thread ist `?` ohnehin unmöglich: der Thread kann `serve` keinen Fehler
  zurückgeben.

## Warum überhaupt Threads? Der Slow-Loris-Test

Ein Test mit vielen *schnellen* parallelen Requests würde **auch sequenziell**
grün — das OS nimmt Verbindungen im Backlog an, der Loop arbeitet sie blitz-
schnell nacheinander ab. Das beweist nichts über echte Parallelität und gäbe
falsche Sicherheit.

Der Unterschied wird nur an einer **hängenden** Verbindung sichtbar:

- Client A verbindet, schickt **nichts** → `serve` steckt in `handle_connection`s
  `stream.read(...)` fest.
- **Sequenziell**: der Loop kommt nie zum nächsten `accept`, Client B wird nie
  bedient → Timeout. ROT.
- **Mit Threads**: A hängt in *seinem* Thread, der Loop akzeptiert sofort
  weiter, B wird bedient. GRÜN.

Testtechnik: `let _slow` (benannt, hält den Socket offen — `let _` würde ihn
sofort droppen), `set_read_timeout` (macht „blockiert" zu einem sauberen `Err`
statt den Test einzufrieren), `sleep` erzwingt die Reihenfolge (zeitabhängig,
also die fragile Sorte Test — bewusst in Kauf genommen).

## `move`, `Send` — und was *nicht* gebraucht wird

- **`move`** ist Pflicht: die Closure übernimmt `stream`. Der Thread kann länger
  leben als die Loop-Iteration; nur-Borgen wäre use-after-free, den Rust
  verbietet. Der Compiler erzwingt `move`.
- **`Send`**: `thread::spawn` verlangt, dass alles Ge-`move`-te zwischen Threads
  bewegbar ist. `TcpStream` ist `Send`, deshalb kompiliert es. Etwas
  Nicht-`Send`es (z. B. `Rc`) würde hier vom Compiler abgelehnt.
- **Kein `Arc`/`Mutex`**: jeder Thread besitzt seinen eigenen `stream`, es gibt
  nichts Geteiltes. Shared State (und damit `Arc<Mutex<…>>`) kommt erst, wenn
  Threads sich echten gemeinsamen Zustand teilen — Phase B, die CRUD-`HashMap`.

## Security / Performance

- 🔒 Der Slow-Loris-Test zeigt die *Lösung* für eine hängende Verbindung, aber
  unbegrenztes `thread::spawn` ist **selbst** ein DoS-Vektor: jede Verbindung =
  ein OS-Thread. Ein Angreifer öffnet zehntausende hängende Verbindungen →
  Thread- und Speichererschöpfung. Ein echter Server braucht ein Limit
  (Thread-Pool, Backpressure) oder ein anderes Modell.
- ⚡ Thread-per-Connection skaliert nicht: ein Thread kostet Stack (grob ~8 MiB
  reserviert) und Kontextwechsel. Bei zehntausenden Verbindungen ist das teuer.
  Genau diese Grenze motiviert **async** (Lektion 17): viele Verbindungen auf
  wenigen Threads, ohne pro Verbindung einen zu blockieren.

## Getestet

- `serve_handles_multiple_connections` — drei Requests nacheinander, alle `200`
  (Loop löst die Einmal-Grenze).
- `slow_connection_does_not_block_others` — hängende Verbindung A, Client B
  bekommt trotzdem prompt `200` (Threads lösen die Blockier-Grenze).
- `serve_one_serves_only_the_first_connection` bleibt: dokumentiert, *warum*
  `serve_one` weiterhin existiert (terminierbar für die Einmal-Tests).
