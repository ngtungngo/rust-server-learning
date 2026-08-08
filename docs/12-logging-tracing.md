# 12 – Logging mit `tracing`

## Ziel

Strukturiertes Logging integrieren und die Log-Ausgabe testen.

## Architektur: Facade vs. Implementation (wie slf4j / log4j)

Zwei strikt getrennte Rollen:

- **Facade** `tracing` — die Library (`lib.rs`) ruft `warn!`, `debug!`, `info!`.
  Sie weiß **nicht**, wohin geloggt wird.
- **Subscriber** `tracing-subscriber` — die App (`main.rs`) entscheidet
  einmalig, *wohin* und *welches Level*.

**Goldene Regel: Libraries loggen gegen die Facade, initialisieren NIE einen
Subscriber.** Nur `main` (oder der Test) setzt einen. Zwei Subscriber würden
sich in die Quere kommen.

## Dependencies

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
tracing-test = { version = "0.2", features = ["no-env-filter"] }
```

- `tracing` ist Laufzeit-Abhängigkeit (die Library loggt im Produktivcode).
- `tracing-test` steht unter `[dev-dependencies]` — nur beim Testen gebraucht,
  nicht im ausgelieferten Binary (analog Mavens `<scope>test</scope>`).

## Subscriber-Init in `main` (nicht auf Datei-Ebene!)

Statements gehören in einen Funktionskörper. Der Subscriber wird als erstes in
`main` initialisiert, bevor das erste Log-Event kommt:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    // ...
}
```

`RUST_LOG=debug cargo run` zeigt DEBUG+INFO, ohne `RUST_LOG` nur INFO (Fallback).

## Strukturierte Felder — der Kernvorteil gegenüber `log`

Felder getrennt von der Nachricht:

```rust
tracing::warn!(input = %input, "invalid port number");
tracing::debug!(port, "parsed port successfully");
```

- `port` ist Kurzform für `port = port` (Feldname = Variablenname).
- `%input` zeichnet via `Display` auf, `?input` via `Debug` (nötig, weil
  `tracing` sonst nicht weiß, wie es einen `&str` aufzeichnen soll).
- Ausgabe: `parsed port successfully port=8080` — nach `port=8080` kann später
  gefiltert werden, statt Text zu parsen. Das macht `tracing` server-tauglich.

Jeder Fehlerpfad loggt eine eigene, unterscheidbare Meldung: Parse-Fehler
(`"invalid port number"`) vs. Null-Port (`"port must not be zero"`).

## Log-Ausgabe testen mit `tracing-test`

```rust
use tracing_test::traced_test;

#[traced_test]
#[test]
fn logs_a_warning_on_empty_host() {
    let _ = ServerConfig::new("   ", "8080");
    assert!(logs_contain("empty host"));
}
```

`#[traced_test]` installiert für die Testdauer einen Capture-Subscriber;
`logs_contain(substr)` prüft, ob ein Log den Substring enthält.

### Falle: Integrationstests brauchen `no-env-filter`

Jede Datei in `tests/` ist ein **eigener Crate** (nutzt die Library von außen).
`tracing-test` filtert standardmäßig auf den Test-Crate — Library-Logs (Target
`rust_server_learning`) fallen raus und `logs_contain` findet nichts. Lösung:
das Feature `no-env-filter` einschalten (siehe Cargo.toml oben).

## Debug-Vorgehen bei rotem `logs_contain`

Zuerst die Testausgabe lesen:

- Steht der `WARN`/`INFO`-Log da? → **Text-Mismatch** (gesuchter Substring
  stimmt nicht mit dem Log-Text überein).
- Kein Log sichtbar? → **Pfad- oder Filter-Problem** (falscher Codepfad geübt,
  oder `no-env-filter` fehlt).

## Ausblick

`tracing`-Spans (`#[instrument]`) erhalten Kontext über `.await`-Punkte hinweg —
der eigentliche Grund, warum `tracing` im async/tokio-Server Standard ist. Das
wird relevant, sobald der echte Server dazukommt.
