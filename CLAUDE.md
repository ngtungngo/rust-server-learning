---
obsidian-vault: "obsidian-vault"
obsidian-project-path: "02-Knowledge/learnings"
git-md-policy: ALWAYS
---

# CLAUDE.md — rust-server-learning

> Nur projektspezifische Regeln. Globale Workflow-, Task- und Kern-Prinzipien
> stehen in `~/.claude/CLAUDE.md` und werden hier **nicht** wiederholt.

## Was das ist

Ein persönliches Rust-Lernprojekt von Tung (Web-/Serverentwicklung), testgetrieben
in kleinen Lektionen. Zweck ist Lernen, nicht Produktion: kleinste sinnvolle
Schritte schlagen Vollständigkeit.

## Sprache (überschreibt geerbte Regeln)

- **Lern-Docs (`docs/`) und Vault-Notizen: Deutsch.** Sie sind Tungs
  Erinnerungsstütze. Die "English only"-Regel aus `~/workspace/CLAUDE.md`
  (GIAM) gilt hier **nicht**.
- **Code-Bezeichner, Fehlermeldungen, Commit-Messages: Englisch.**
- Kommentare im Code nur, wo sie einen Gedanken tragen — die Erklärung
  gehört in die passende `docs/NN-*.md`, nicht in den Code.

## TDD-Ablauf (verbindlich pro Lektion)

1. Erwartetes Verhalten als Test in `tests/` formulieren (Dateiname sagt, was
   getestet wird, z. B. `port_parsing_test.rs`).
2. Test fehlschlagen lassen.
3. Kleinstmögliche Implementierung in `src/`.
4. `cargo test` — alles grün.
5. Lektion committen; eine kurze `docs/NN-thema.md` als Erinnerungsstütze
   schreiben.

## Befehle

```bash
cargo test    # alle Tests
cargo run     # Programm starten
cargo clippy  # Lints, bevor eine Lektion als fertig gilt
```

## Aktueller Stand (ehrlich halten, nicht kaschieren)

- `std`-only, keine externen Crates, kein async, keine echte Netzwerk-I/O.
- Der "Server" ist bisher reine Konfigurationslogik (`ServerConfig`, Bind-Adresse).
- Die Liste im `README.md` (Tokio, Axum) ist die **geplante** Richtung, nicht
  der Ist-Zustand. Beim Dokumentieren den tatsächlichen Stand beschreiben.

## Vault-Anbindung (zweites Gehirn)

Nach einer substanziellen Lektion den **Sprach-Wissensstand** pflegen:

- Notiz: `02-Knowledge/learnings/2026-08-08-rust-wissensstand-server-projekt.md`
  im Vault `obsidian-vault` (per `obsidian`-CLI aktualisieren, nicht neu anlegen).
- Schreibstil folgt `[[05-Meta/ai-schreibregeln]]`: kurze Sätze, konkrete
  Details, Wissenslücken offen benennen statt glattziehen.
- Nicht verwechseln: `rust-test-*`-Notizen betreffen den *Schreibstil über*
  Rust, diese Notiz betrifft das *Sprachwissen in* Rust.
