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

Ein laufender HTTP-Server auf Basis von **`axum`**. `main` startet
`axum::serve(listener, router())` mit `.with_graceful_shutdown(ctrl_c)`. Die
Routen (`/api/health`, `/api/item/{id}`, `POST /api/item`, `/api/slow`) sind
axum-Handler mit `IntoResponse`, Path-Extractor und Methoden-Routing; `404`/`405`
liefert axum automatisch. Ein per-Request-Timeout hängt als
`tower_http::TimeoutLayer` am Router (Slow-Loris-Abwehr, gibt `408`). Die
Konfiguration (`ServerConfig`, Host/Port-Validierung, `ConfigError`) bleibt
framework-unabhängig. Routen werden in-process über `tower`s `oneshot` getestet
(kein TCP, kein Runtime-Deadlock). CI (GitHub Actions) prüft `fmt`, `clippy
-D warnings`, Build und Tests.

JSON läuft über `serde`: Domänentypen in `src/models.rs`, getrennt von der
HTTP-Schicht. Ein-und Ausgabe sind bewusst getrennte Typen (`CreateItem` ohne
`id` — die ist serverkontrolliert). Ehrliche Grenze: `serde` prüft nur Struktur,
nicht Semantik (leerer/überlanger `name` kommt durch) — inhaltliche Validierung
ist eine spätere Schicht.

Volles **CRUD** über einen In-Memory-Store: `AppState` mit
`Arc<RwLock<HashMap<String, Item>>>` (`src/store.rs`), geteilt via axums
`State`-Extractor. `POST` (`201`), `GET /{id}` (`200`/`404`), `GET` (Liste),
`PUT /{id}` (`200`/`404`), `DELETE /{id}` (`204`/`404`). IDs sind server-vergebene
UUIDs (v4, nicht durchzählbar). `std::sync::RwLock` reicht, weil der Guard nie
über ein `.await` lebt. Noch im RAM (Neustart = Datenverlust); die Store-Methoden
sind so geschnitten, dass sie in Phase C zu einem `Repository`-Trait werden.

Eine erste **fachliche Regel**: `name` ist eindeutig. `insert` gibt
`Result<Item, StoreError>` (getippter Fehler via `thiserror`); ein doppelter Name
wird im Handler auf `409 Conflict` gemappt. Prüfung und Insert teilen sich einen
`write`-Guard (kein TOCTOU-Race zwischen zwei parallelen Creates). Das ist die
Blaupause für das DB→HTTP-Fehler-Mapping in Phase C.

Der handgebaute Stack aus Lektion 14–17 (`serve`/`select!`/`JoinSet`/Registry,
eigenes Request/Response-Parsing) wurde nach der axum-Migration entfernt — er
bleibt über die Tags `lesson-14` … `lesson-17` und `docs/14`–`docs/17` erhalten.
Er war die Grundlage, um zu *verstehen*, was `axum::serve` intern tut.

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
16. Graceful Shutdown + per-Verbindung-Logging (`Arc<AtomicBool>`, `JoinHandle`/`join`, `set_read_timeout`, `tracing`-Span)
17. Async mit `tokio` (`select!`, `pin!`, `JoinSet`, `tokio::time::timeout`, Registry mit `AbortHandle`, `ctrl_c`)
18. HTTP-APIs mit `axum` (`Router`, Handler + `IntoResponse`, `Path`-Extractor, `axum::serve` + graceful shutdown, `tower_http::TimeoutLayer`, `oneshot`-Tests, CI-Härtung)
19. `serde` + JSON (`Serialize`/`Deserialize`, `Json<T>` als Extractor und Response, Input-≠-Output-Typen, `201 Created`, Struktur- vs. Semantik-Validierung)
20. In-Memory-Store + CRUD (`AppState`, `Arc<RwLock<HashMap>>`, `State`-Extractor, `uuid` v4, `200`/`201`/`204`/`404`, `std`- vs. `tokio`-`RwLock`)
21. `409 Conflict` + erster fachlicher Fehler (Eindeutigkeit von `name`, `insert -> Result<Item, StoreError>`, `thiserror`, Fehler→HTTP-Mapping, TOCTOU unter einem Guard)

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

### Phase B — REST / CRUD

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
