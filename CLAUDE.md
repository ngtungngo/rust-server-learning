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

## Security & Performance als Querschnitt

Tung lernt Rust auch wegen Security und Performance. Diese Themen sind **kein
eigenes Kapitel am Ende**, sondern Querschnitt: In jeder Lektion einen kurzen,
**konkreten** Kommentar an einer echten Design-Entscheidung, die gerade
getroffen wird.

- **Security**: an der Entscheidung verankern (z. B. Input-Validierung an der
  Grenze → `400`; Größenlimits gegen DoS; Path Traversal; Timeouts). Ehrlich
  bleiben: Memory Safety ≠ Application Security; dieses Lernprojekt ist **kein**
  produktionssicherer/auditierter Server.
- **Performance**: Trade-offs bewusst machen, **nicht** vorab mikro-optimieren
  (`String` vs `&str`/Allokation; Thread-per-connection vs. async; blockierend
  vs. non-blocking). Messen mit echtem Werkzeug (`criterion`), nicht per Bauch
  oder naivem `Instant::now()`.
- Kurz halten: ein bis zwei Sätze pro Thema und Lektion, immer an konkretem Code.

## Dateigröße & Modularisierung (verbindlich)

- **Keine Quelldatei über 250 Zeilen.** Wird eine Datei größer, ist das ein
  Signal, sie in Module aufzuteilen — nicht später, sondern sofort.
- Jederzeit refaktorieren erlaubt und erwünscht: sinnvoll modularisieren, sobald
  eine Datei mehrere Verantwortlichkeiten trägt (z. B. `error`, `config`,
  `server` in eigene Module/Dateien).
- Beim Coachen: wenn eine Datei der Grenze nahekommt, den Refactor als eigenen
  Schritt vorschlagen und begründen (welche Verantwortung wandert wohin, warum).

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
