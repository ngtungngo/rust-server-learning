# 04 – Result und Tests

## Ziel

Ungültige Konfiguration ausdrücklich behandeln und automatisch testen.

`parse_port` liefert einen Wert vom Typ `Result<u16, String>`:

- `Ok(port)`: eine gültige Portnummer.
- `Err(message)`: eine verständliche Fehlermeldung.

```rust
match parse_port("8080") {
    Ok(port) => println!("Server will use port {port}"),
    Err(message) => println!("Configuration error: {message}"),
}
```

`match` behandelt beide möglichen Varianten. Es ist dem Zweck nach ähnlich zu
Java `try/catch`, aber ein Fehler ist hier ein normaler, typisierter Rückgabewert
und keine geworfene Exception.

## Tests

Die Tests stehen in `tests/port_parsing_test.rs`. Sie prüfen:

- eine gültige Portnummer,
- nicht-numerische Eingabe,
- Port `0`.

`lib.rs` bedeutet **library**. Es ist nicht der Name für eine Testdatei:
Es enthält wiederverwendbare Programmlogik. `main.rs` startet das Programm und
benutzt diese Logik.
