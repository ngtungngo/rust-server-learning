# 01 – Projekt-Setup

## Ziel

Ein ausführbares Rust-Projekt mit Cargo, Git und GitHub einrichten.

## Wichtige Dateien

- `Cargo.toml`: Projektbeschreibung und Abhängigkeiten.
- `src/main.rs`: Startpunkt des ausführbaren Programms.
- `Cargo.lock`: festgehaltene Versionen der Abhängigkeiten.
- `target/`: erzeugte Build-Dateien; wird nicht in Git eingecheckt.

## Wichtige Befehle

- `cargo run`: kompilieren und starten.
- `cargo test`: alle Tests ausführen.
- `cargo fmt`: Code formatieren.
- `cargo clippy`: zusätzliche Code-Hinweise prüfen.

Cargo ist vergleichbar mit Maven oder Gradle, aber auch mit npm: Es baut,
testet und verwaltet Abhängigkeiten.
