# Rust Server Learning

Ein praxisorientiertes Rust-Lernprojekt für Web- und Serverentwicklung,
testgetrieben in kleinen Lektionen aufgebaut. Jede Lektion hat einen Test, eine
Erinnerungsstütze in `docs/` und einen Git-Tag (`lesson-NN`).

## Starten

```bash
cargo run                 # startet das Programm
cargo test                # alle Tests
RUST_LOG=debug cargo run  # mit ausführlichem Logging
```

## Stand

Ein laufender HTTP-Server über echtes TCP. `serve` akzeptiert in einer
Endlosschleife (`incoming()`) und bedient jede Verbindung in einem eigenen
Thread — mehrere Verbindungen gleichzeitig, eine hängende blockiert die anderen
nicht. `serve_one` bleibt als terminierbarer Einmal-Bediener für Tests.
Nächster Schritt: Graceful Shutdown und, motiviert durch die Thread-Grenze,
async (siehe Roadmap).

## Bearbeitete Lektionen

Details je Lektion in [`docs/`](docs/README.md).

1. Projekt-Setup (Cargo, Git)
2. Ownership und Borrowing
3. Structs und Methoden
4. Fehlerbehandlung mit `Result` und Tests
5. Server-Konfiguration (`ServerConfig`)
6. Bind-Adresse
7. `self` versus `&self`
8. Fehlertyp als Enum (`ConfigError`)
9. Das `std::error::Error`-Trait
10. Fehler-Ketten mit `source()`
11. Fehler-Boilerplate mit `thiserror`
12. Strukturiertes Logging mit `tracing`
13. Request/Response-Typen + reiner Handler (Routing, `/api/item/:id`, Builder)
14. TCP-Schale (`serve_one`, Request-Parsing, HTTP-Serialisierung, `curl`-testbar)
15. Accept-Loop + Thread pro Verbindung (`serve`, `incoming()`, `thread::spawn`, `move`/`Send`)

## Zielbild

Ein produktionsnaher **CRUD-Server**: REST-API (JSON), Persistenz in SQL
(`sqlx`) und später NoSQL (MongoDB) hinter derselben Repository-Abstraktion,
geschützt per **OAuth2 / OIDC mit Keycloak** als IDP. Der Weg dorthin durchläuft
die Rust-Kernkonzepte, an denen sich „kann ich Rust" entscheidet: Ownership
unter async-Druck, Traits als Architektur, Fehler über Schichten, Typsystem für
Korrektheit.

## Roadmap (geplant, noch nicht umgesetzt)

Testbarkeit und Design zuerst, Netzwerk danach. Reiner Kern (Logik) getrennt
von der I/O-Schale — das Ports-and-Adapters-Muster. Jede Stufe motiviert durch
das Limit der vorigen.

### Phase A — HTTP-Server von Grund auf

16. **Graceful Shutdown (einfach, sync) + per-Verbindung-Logging** (`tracing`-
    Span pro Verbindung).
17. **Asynchron mit `tokio`** — dieselbe Struktur async; eleganter Graceful
    Shutdown via `tokio::select!`.
18. **HTTP-APIs mit `axum`** — Router und Handler als echte Abstraktion.

### Phase B — REST / CRUD

19. **`serde` + JSON** — Request-Body deserialisieren, Response serialisieren.
20. **CRUD mit In-Memory-Store** (`Arc<RwLock<HashMap>>`) — CRUD-Semantik und
    shared State lernen, bevor eine DB dazukommt.
21. **Ressourcen-Routing + Statuscodes** (`201 Created`, `204 No Content`,
    `409 Conflict`, `404`).
22. **Input-Validierung als eigene Schicht** (🔒 nie ungeprüfte Daten in die
    Domäne).

### Phase C — Persistenz

23. **SQL mit `sqlx`** — Connection-Pool, erste async Query.
24. **Migrations + Repository-Pattern** — Trait `Repository`, implementiert von
    DB *und* In-Memory (für Tests). Ports-and-Adapters in Reinform.
25. **Fehler-Mapping DB → HTTP** (baut auf `thiserror` auf).
26. **NoSQL-Variante (MongoDB)** hinter *demselben* Repository-Trait — zeigt,
    warum die Abstraktion trägt.

### Phase D — Authentifizierung (OAuth2 / OIDC)

27. **JWT verstehen + validieren** (`jsonwebtoken`) — Signaturprüfung.
28. **Keycloak als IDP** — JWKS-Endpoint, Token gegen Realm prüfen
    (`iss`/`aud`/`exp`). 🔒 echte Security-Tiefe.
29. **axum-Middleware / Extractor** — geschützte Routes, Rollen aus dem Token.

### Phase E — Produktionsreife

30. Config aus Env/Datei, Docker, Integrationstests gegen echte DB
    (testcontainers), Observability. (Wird bei Ankunft verfeinert.)

## Weiterführende Ideen (jenseits des Web-Backends)

Wofür Rust noch benutzt wird — als Horizont für spätere Projekte. Fast alles
baut auf denselben Grundlagen (Ownership, Traits, Fehler, async); der
Domänenwechsel bedeutet meist neue Crates, keine neue Sprache. Manches
(Embedded, Game-ECS) ist ein eigenes Universum mit größerem Sprung.

- **CLI-Tools** — Rusts Paradedisziplin (`ripgrep`, `fd`, `bat`). Single-Binary,
  keine Runtime. Crate: `clap`. Kleiner Sprung von hier.
- **WebAssembly** — Rust → WASM im Browser, nahe an React-Erfahrung: schwere
  Logik in Rust, eingebunden ins Frontend. Crates: `wasm-bindgen`, `wasm-pack`;
  Frameworks `leptos`/`yew` (React-artig).
- **Native Extensions** — Rust als schneller Kern für andere Sprachen: Python
  (`pyo3`, z. B. `polars`), Node (`napi-rs`). Trifft das Performance-Interesse.
- **Netzwerk / Infrastruktur** — Proxies, Load-Balancer, Suchengines
  (`tantivy`), Log-Pipelines (`vector`). Direkt anschlussfähig ans Server-Wissen.
- **Security-Tooling** — sichere Protokoll-/Format-Parser (Memory Safety
  entschärft einen klassischen C-Angriffsvektor), Scanner, Krypto-Werkzeuge.
- **Embedded / IoT** — Rust auf Mikrocontrollern ohne OS (`no_std`); Memory
  Safety ohne GC. Steil, braucht Hardware und anderes Mindset.
- **Game Dev / Simulation** — `bevy` (ECS-Engine); das Entity-Component-System
  ist ein bewusst un-OOP Design-Pattern.
- **Data / ML-Infrastruktur** — nicht Modelltraining (bleibt Python), aber die
  schnelle Infrastruktur: `polars`, Tokenizer (`tokenizers`), Inferenz-Serving.
