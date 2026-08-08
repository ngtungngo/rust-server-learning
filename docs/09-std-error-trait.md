# 09 – Das `std::error::Error`-Trait

## Ziel

`ConfigError` in das allgemeine Fehler-Ökosystem einklinken, damit `?` und
`Box<dyn Error>` funktionieren.

## Die eine Zeile

```rust
impl std::error::Error for ConfigError {}
```

Ein **leerer** Impl-Block. Das reicht, weil:

- `std::error::Error` verlangt `Display + Debug` als Supertraits — beide hat
  `ConfigError` seit Lektion 8.
- Alle Methoden des Traits (`source()` usw.) haben Default-Implementierungen.

`Error` ist ein Marker-/Capability-Trait: die Zeile erklärt „dieser Typ ist ein
richtiger Fehler", die Fähigkeiten kommen aus `Display`.

## Was die Zeile freischaltet

Die Standardbibliothek hat bereits:

```rust
impl<E: Error> From<E> for Box<dyn Error>
```

Sobald `ConfigError: Error` gilt, greift dieses `From` automatisch. Damit
funktionieren beide ohne eigenes `From`:

```rust
// im Test
let error: Box<dyn std::error::Error> =
    ServerConfig::new("   ", "8080").unwrap_err().into();

// in main
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new("localhost", "8080")?;
    println!("Server will listen on {}", config.bind_address());
    Ok(())
}
```

`?` konvertiert den Fehler implizit über `From` — genau die Kette, die die
Compiler-Meldung nennt.

## Design: `Box<dyn Error>` ist Type Erasure

`Box<dyn Error>` versteckt den konkreten Typ hinter einem Pointer (dynamic
dispatch). Der Aufrufer weiß danach nur „irgendein Fehler", nicht mehr *welcher*.

Faustregel:

- **Bibliotheks-API**: konkreten `enum` zurückgeben, damit der Aufrufer per
  `match` reagieren kann.
- **`main` / Anwendungscode**: `Box<dyn Error>` — der Fehler wird nur noch
  angezeigt, die Typinfo braucht niemand mehr.

## Falle: `?` in `main` gibt `Debug` aus, nicht `Display`

Wenn `main` mit `Err` endet, druckt die Runtime den Fehler über **`Debug`**:

```
Error: EmptyHost
```

Nicht die schöne `Display`-Meldung `Host must not be empty`. Das ist bewusst so
(`Termination`-Trait). Wer die `Display`-Meldung will, nutzt in `main` ein
explizites `match` oder eine Crate wie `anyhow`.

## Ausblick

`source()` wird interessant, sobald ein Fehler einen anderen umschließt (z. B.
den ursprünglichen `ParseIntError` in `InvalidPort` behalten). Dann bildet
`source()` eine Fehler-Kette. Das automatisiert `thiserror` mit `#[source]`.
