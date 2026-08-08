# 08 – Fehlertyp als Enum

## Ziel

Fehler nicht mehr als `String` zurückgeben, sondern als eigenen Typ, den der
Aufrufer unterscheiden kann.

## Vorher / nachher

Vorher: `Result<u16, String>` — der Fehler war nur Text. Wer ihn behandeln
wollte, musste den Text vergleichen.

Nachher: ein `enum` mit benannten Varianten:

```rust
#[derive(Debug, PartialEq)]
pub enum ConfigError {
    EmptyHost,
    InvalidPort(String),
    PortZero,
}
```

`InvalidPort(String)` trägt den **rohen ungültigen Wert** (z. B. `"abc"`),
nicht die fertige Meldung. Das ist der Kern: Daten und Darstellung trennen.

## Zwei Formatierungs-Traits

Rust spaltet, was Java in `toString()` presst:

| Trait | Platzhalter | Für wen | Ableitbar? |
|---|---|---|---|
| `Debug` | `{:?}` | Entwickler, Tests | ja, `#[derive(Debug)]` |
| `Display` | `{}` | Endnutzer | nein, von Hand |

`Debug` liefert `assert_eq!` die Ausgabe im Fehlerfall. `Display` liefert die
menschenlesbare Meldung für `main`:

```rust
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::EmptyHost => write!(f, "Host must not be empty"),
            ConfigError::InvalidPort(value) => write!(f, "'{value}' is not a valid port number"),
            ConfigError::PortZero => write!(f, "Port 0 is not allowed"),
        }
    }
}
```

Der `match` ist exhaustiv: eine neue Variante zwingt den Compiler, hier eine
Meldung zu ergänzen. Die Anführungszeichen leben **nur** hier, nicht in der
Variante — sonst wäre die Formatierung wieder in die Daten geleckt.

## `?` und die Fehlerkonvertierung

`parse_port` liefert jetzt `Result<u16, ConfigError>`. Weil `new` denselben
Fehlertyp hat, muss `?` nichts konvertieren:

```rust
port: parse_port(port_input)?,
```

Hätte `parse_port` weiter `String` geliefert, hätte `?` ein
`impl From<String> for ConfigError` gebraucht — genau diesen
Konvertierungsmechanismus nutzt später die Crate `thiserror`.

## Warum Tests nur den Wert prüfen, nicht die Meldung

```rust
assert_eq!(result, Err(ConfigError::InvalidPort("abc".to_owned())));
```

Der Test vergleicht `"abc"`, nicht den ganzen Satz. So bleibt er stabil, wenn
sich der Meldungstext in `Display` ändert. Ein Test, der die volle Meldung
vergleicht, bricht bei jeder Formulierungsänderung — fragil.

`assert_eq!` braucht `PartialEq` + `Debug` auf **beiden** Seiten des `Result`:
darum leiten sowohl `ConfigError` als auch `ServerConfig` sie ab.

## Design-Muster

Idiomatisches Rust-Fehlermodell, Separation of Concerns:

- `enum` = *welcher* Fehler (Daten, unterscheidbar, `match`-bar)
- `Display` = *wie er für Menschen klingt* (Präsentation)
- `main` = *wohin* er ausgegeben wird (Ausgabeort)

Offen für Lektion 9: `impl std::error::Error for ConfigError {}` — das
Marker-Trait, das den Fehler ins allgemeine Ökosystem einklinkt
(`Box<dyn Error>`, `?` über Funktionsgrenzen).
