# 11 – Fehler-Boilerplate mit `thiserror`

## Ziel

Die handgeschriebenen `impl Display` und `impl Error` (Lektion 8–10) durch die
Crate `thiserror` ersetzen — ohne dass sich das Verhalten ändert.

## Erste externe Dependency

`Cargo.toml`:

```toml
[dependencies]
thiserror = "2"
```

`"2"` ist eine SemVer-Range: ≥ 2.0.0, < 3.0.0 — Cargo nimmt die neueste 2.x.
`Cargo.lock` hält die exakt aufgelöste Version fest und wird mitcommittet.

## Vorher / nachher

Vorher: ~15 Zeilen Handarbeit (`impl Display` mit `match`, `impl Error` mit
`source()`). Nachher: Attribute am Enum.

```rust
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum ConfigError {
    #[error("Host must not be empty")]
    EmptyHost,
    #[error("'{value}' is not a valid port number")]
    InvalidPort { value: String, source: std::num::ParseIntError },
    #[error("Port 0 is not allowed")]
    PortZero,
}
```

## Was jedes Attribut ersetzt

| Attribut | Ersetzt den Handcode aus |
|---|---|
| `#[derive(Error)]` | `impl std::error::Error for ConfigError {}` (Lektion 9) |
| `#[error("...")]` pro Variante | den ganzen `impl Display`-`match` (Lektion 8) |
| Feld heißt `source` | `fn source()` — `thiserror` erkennt `source` automatisch (Lektion 10) |

`#[error("'{value}' ...")]` interpoliert das benannte Feld `value` mit Display —
exakt das alte `write!(f, "'{value}' ...")`.

## Der TDD-Beweis: Tests bleiben unverändert

Es wird **kein neuer Test** geschrieben. Die 9 bestehenden Tests sind die
Spezifikation. Der Refactor ist genau dann korrekt, wenn `cargo test` grün
bleibt, **ohne** dass eine Testzeile geändert wird (`git diff tests/` leer).
Klassisches Refactoring: Verhalten bleibt, Implementierung ändert sich, Tests
beweisen es.

## Nicht benutzt: `#[from]`

`#[from]` würde zusätzlich `impl From<ParseIntError>` generieren, sodass `?` den
Fehler automatisch konvertiert. Hier ungeeignet: `#[from]` kennt nur den Fehler,
nicht den `input`-String — das `value`-Feld ließe sich nicht setzen. Deshalb
bleibt `parse_port` bei `.map_err(|e| ... { value, source: e })`.

## Verstehen, nicht nur benutzen

Der Wert der Lektion: Das Makro ist keine Blackbox. Für jede Attribut-Zeile ist
bekannt, welchen Rust-Code sie ersetzt — weil er in Lektion 8–10 von Hand
geschrieben wurde.
