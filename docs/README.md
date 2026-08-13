# Rust Server Learning – Lernnotizen

Dieses Verzeichnis ist deine persönliche Erinnerungsstütze. Jede Lektion
beschreibt das Ziel, den wichtigsten Rust-Gedanken und die zugehörigen Dateien.

## Lektionen

1. [Projekt-Setup](01-project-setup.md)
2. [Ownership und Borrowing](02-ownership-and-borrowing.md)
3. [Structs und Methoden](03-structs-and-methods.md)
4. [Result und Tests](04-result-and-testing.md)
5. [Server-Konfiguration](05-server-configuration.md)
6. [Bind-Adresse](06-bind-address.md)
7. [`self` und `&self`](07-self-versus-borrowing.md)
8. [Fehlertyp als Enum](08-error-enum.md)
9. [Das `std::error::Error`-Trait](09-std-error-trait.md)
10. [Fehler-Ketten mit `source()`](10-error-source-chains.md)
11. [Fehler-Boilerplate mit `thiserror`](11-thiserror.md)
12. [Logging mit `tracing`](12-logging-tracing.md)
13. [Request/Response-Typen + reiner Handler](13-request-response-handler.md)
14. [TCP-Schale (`serve_one`)](14-tcp-server.md)
15. [Accept-Loop + Thread pro Verbindung](15-accept-loop.md)
16. [Graceful Shutdown + per-Verbindung-Logging](16-graceful-shutdown.md)
17. [Async mit `tokio`: select, JoinSet, Timeout, Registry](17-async-tokio.md)
18. [HTTP-APIs mit `axum`: Router, Handler, Layer, Migration](18-axum.md)
19. [`serde` + JSON: Serialize/Deserialize, Json<T>, Input-≠-Output](19-serde-json.md)

## Arbeitsweise

Wir arbeiten testgetrieben:

1. Erwartetes Verhalten als Test formulieren.
2. Test zunächst fehlschlagen lassen.
3. Kleinstmögliche Implementierung schreiben.
4. `cargo test` ausführen.
5. Lektion committen und mit einem Git-Tag markieren.

Die Tests liegen bewusst im Verzeichnis `tests/`. Die Dateinamen sagen, was
getestet wird, zum Beispiel `port_parsing_test.rs`.
