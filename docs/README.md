# Rust Server Learning – Lernnotizen

Dieses Verzeichnis ist deine persönliche Erinnerungsstütze. Jede Lektion
beschreibt das Ziel, den wichtigsten Rust-Gedanken und die zugehörigen Dateien.

## Lektionen

1. [Projekt-Setup](01-project-setup.md)
2. [Ownership und Borrowing](02-ownership-and-borrowing.md)
3. [Structs und Methoden](03-structs-and-methods.md)
4. [Result und Tests](04-result-and-testing.md)
5. [Server-Konfiguration](05-server-configuration.md)

## Arbeitsweise

Wir arbeiten testgetrieben:

1. Erwartetes Verhalten als Test formulieren.
2. Test zunächst fehlschlagen lassen.
3. Kleinstmögliche Implementierung schreiben.
4. `cargo test` ausführen.
5. Lektion committen und mit einem Git-Tag markieren.

Die Tests liegen bewusst im Verzeichnis `tests/`. Die Dateinamen sagen, was
getestet wird, zum Beispiel `port_parsing_test.rs`.
