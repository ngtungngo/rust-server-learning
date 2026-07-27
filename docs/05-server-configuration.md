# 05 – Server-Konfiguration

## Ziel

Host und Port zu einem sicheren Konfigurationsobjekt bündeln.

`ServerConfig::new("localhost", "8080")` validiert beide Eingaben und liefert
entweder eine Konfiguration oder eine Fehlermeldung.

```rust
match ServerConfig::new("localhost", "8080") {
    Ok(config) => println!("{}:{}", config.host(), config.port()),
    Err(message) => println!("Configuration error: {message}"),
}
```

## Was neu ist

- `ServerConfig` ist ein `struct` mit privaten Feldern.
- `new` ist ein Konstruktor-ähnlicher Associated Function.
- `host()` und `port()` sind lesende Methoden (Getter).
- `?` in `new` gibt einen Fehler von `parse_port` direkt zurück.

## Ablauf im Programm

```mermaid
flowchart LR
    M[main.rs] --> C[ServerConfig::new]
    C --> H{Host leer?}
    H -- ja --> E[Err: Fehlermeldung]
    H -- nein --> P[parse_port]
    P --> R{Port gültig?}
    R -- ja --> O[Ok: ServerConfig]
    R -- nein --> E
```

`main.rs` entscheidet mit `match`, ob es eine gültige Konfiguration verwenden
oder eine Fehlermeldung ausgeben soll.

## Tests

`tests/server_config_test.rs` prüft:

- gültigen Host und Port,
- leeren Host,
- ungültigen Port.

Noch läuft kein Netzwerkserver. Wir haben bisher nur die Konfiguration gebaut,
die ein späterer Server benutzen wird.
