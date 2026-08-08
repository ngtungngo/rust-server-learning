# 10 – Fehler-Ketten mit `source()`

## Ziel

Den ursprünglichen Fehler (die Ursache) behalten, statt ihn wegzuwerfen, und
über `source()` zugänglich machen.

## `source()` ist Javas `getCause()`, nicht der Stacktrace

- `source()` = Ursachenkette: *welcher* Fehler löste diesen aus.
- **Nicht** der Stacktrace (*wo* im Code) — Rust-Fehler tragen per Default
  keinen automatischen Stacktrace. Fehler sind Werte, keine Objekte mit
  Laufzeit-Trace. Etwas Stacktrace-artiges gibt es nur opt-in
  (`std::backtrace::Backtrace`, `RUST_BACKTRACE=1`, oder `anyhow`).

## Die Variante trägt jetzt Wert UND Ursache (Struct-Variante)

```rust
InvalidPort { value: String, source: std::num::ParseIntError },
```

Struct-Variante (geschweifte Klammern, benannte Felder), nicht Tuple. `value`
für die Meldung, `source` für die Ursachenkette. Beim Bauen wird der echte
Fehler eingefangen, statt mit `|_|` verworfen:

```rust
.map_err(|e| ConfigError::InvalidPort { value: input.to_owned(), source: e })?
```

## `source()` implementieren

```rust
impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::InvalidPort { source, .. } => Some(source),
            _ => None,
        }
    }
}
```

Signatur Stück für Stück:
- `dyn Error` — irgendein Fehlertyp (Type Erasure), nicht `ParseIntError` konkret.
- `&(...)` — geliehen, die Ursache bleibt im `ConfigError`.
- `+ 'static` — der Fehler enthält keine geliehenen Referenzen; Pflichtangabe der Signatur.
- `Option` — `EmptyHost`/`PortZero` haben keine Ursache (`None`).

`Some(source)`: `&ParseIntError` wird automatisch zu `&(dyn Error + 'static)`
(unsized coercion), weil `ParseIntError: Error`.

## Trait-Methode braucht das Trait im Scope

`error.source()` im Test schlägt fehl mit „method not found", solange
`use std::error::Error;` fehlt — obwohl `ConfigError` das Trait implementiert.
Trait-Methoden sind nur sichtbar, wenn das Trait importiert ist. Der Import
bringt die *Methode* in Reichweite, nicht den Typ.

## Test-Stil hängt am Fehler-Design

Zwei Muster, bewusst beide als Referenz behalten:

```rust
// assert_eq! — exakt, wenn die Ursache herstellbar ist
let source = "abc".parse::<u16>().unwrap_err();
assert_eq!(parse_port("abc"),
    Err(ConfigError::InvalidPort { value: "abc".to_owned(), source }));

// matches! + Guard — nur Form + relevantes Feld, Ursache egal
assert!(matches!(result,
    Err(ConfigError::InvalidPort { value, .. }) if value == "abc"));
```

- `assert_eq!` geht nur, weil `ParseIntError` selbst `PartialEq` hat (Gleichheit
  an der Fehler-Art, nicht am Text).
- Wechselt `source` je auf `Box<dyn Error>`, funktioniert nur noch `matches!` /
  `to_string()` / `source().is_some()` — `dyn Error` hat kein `PartialEq`.
- Deshalb haben viele reale Fehlertypen **kein** `PartialEq`.

## Ausblick

`thiserror` automatisiert genau das: `#[source]` markiert das Ursachenfeld,
`Display` + `Error` + `source()` entstehen per Makro.
