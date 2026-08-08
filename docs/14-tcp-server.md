# 14 – TCP-Schale (`serve_one`)

## Ziel

Um den reinen `handle` aus Lektion 13 eine dünne I/O-Schale legen, die echte
Bytes von einem TCP-Socket liest, parst, `handle` ruft und die Antwort
zurückschreibt. Der erste Code, der wirklich auf einem Port lauscht.

## Ports and Adapters

- **Reiner Kern** `handle(&Request) -> Response` — kennt keinen Socket, blieb
  unverändert.
- **I/O-Schale** `serve_one(&TcpListener)` — übersetzt Bytes ↔ Typen.

Die bedienende Logik hängt nicht davon ab, woher der Listener kommt — er wird
gereicht (Dependency Injection). Deshalb nimmt `serve_one` einen `&TcpListener`
als Parameter: der Test bindet `:0`, `main` bindet über `ServerConfig`.

## Die vier Schritte

```rust
pub fn serve_one(listener: &TcpListener) -> std::io::Result<()> {
    let (mut stream, addr) = listener.accept()?;     // 1. warten
    let mut buffer = [0u8; 1024];
    let idx = stream.read(&mut buffer)?;             // 2. lesen
    let raw = String::from_utf8_lossy(&buffer[..idx]);
    let response = match parse_request(&raw) {       // 3. parsen
        Some(req) => handle(&req),
        None => Response::text(StatusCode::BadRequest, "bad request"),  // 400
    };
    stream.write_all(to_http(&response).as_bytes())?; // 4. schreiben
    Ok(())
}
```

Genau EINE Verbindung, dann Rückkehr — macht den Test terminierbar. Der
Accept-Loop kommt in Lektion 15. Folge: `cargo run` beantwortet einen Request
und endet.

## TCP-Fallstrick: erst lesen, dann schließen

`Connection reset by peer` (RST) entstand, weil der Server schrieb und den
Socket schloss, ohne die Client-Bytes gelesen zu haben. Ein Socket, der mit
ungelesenen Daten geschlossen wird, sendet RST. **Ein Server muss die Anfrage
lesen, bevor er die Verbindung schließt** — auch wenn er den Inhalt nicht
braucht.

## Parsen ohne `unwrap`: `?` auf `Option`

```rust
fn parse_request(raw: &str) -> Option<Request> {
    let first_line = raw.lines().next()?;
    let mut parts = first_line.split(' ');
    let method = Method::from_str(parts.next()?).ok()?;
    let path = parts.next()?;
    Some(Request::new(method, path.to_owned()))
}
```

- `?` funktioniert auch auf `Option` (nicht nur `Result`): `None` → sofortiger
  Rücksprung mit `None`. Jeder Müll-Input landet bei `None` → `400`. Kein
  `unwrap`, kein Panic bei bösartigem Input (DoS-Schutz).
- `Method` implementiert `std::str::FromStr` → `"GET".parse::<Method>()` geht.
  Trait-Methoden geben `Result`; `.ok()` überbrückt zu `Option`.

## Response → HTTP-Bytes

```
HTTP/1.1 200 OK\r\n
Content-Type: text/plain\r\n
Content-Length: 2\r\n
\r\n
ok
```

- Zeilenende `\r\n` (CRLF), Leerzeile zwischen Headern und Body, `Content-Length`
  = `body.len()`.
- Statuszeile braucht `code()` UND `reason()` (beide `match self`).
- Falle: `{:?}` (Debug) beim Content-Type erzeugt Anführungszeichen → `{}`
  (Display) nehmen.

## Integrationstest: echtes TCP mit Threads

- `TcpListener::bind("127.0.0.1:0")` → OS wählt freien Port, kein Konflikt.
- Client in eigenem `thread::spawn`, sonst Deadlock (`serve_one` blockiert bei
  `accept()`, Client bei `connect()`).
- `stream.shutdown(Shutdown::Write)` signalisiert EOF, sonst hängt
  `read_to_string`.

Netzwerk-Tests mit Threads sind fragiler als Unit-Tests — deshalb ist der reine
`handle` separat unit-getestet, hier nur die Schale integrativ.

## `main` verdrahtet die Kette

```rust
let config = ServerConfig::new("localhost", "8080")?;
let listener = TcpListener::bind(config.bind_address())?;
serve_one(&listener)?;
```

Hier lebt `ServerConfig` (Lektion 5): `bind_address()` liefert den String für
`TcpListener::bind`. Die Abstraktion war nicht tot — sie gehört an den Rand
(`main`), nicht in `serve_one`.

## Manuell verifiziert

`cargo run`, dann `curl -v localhost:8080/api/health` → `200 OK`, Body `ok`.
`/api/item/42` → JSON, `/nope` → `404`. Server pro Lauf neu starten (eine
Verbindung).

## Security / Performance

- 🔒 Fester 1024-Byte-Buffer = erste DoS-Schranke; fail closed (400/404 statt
  Panic).
- ⚡ `from_utf8_lossy` gibt `Cow<str>` — kopiert nur bei ungültigem UTF-8.
