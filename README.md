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

Reine Konfigurations- und Fehler-Logik mit strukturiertem Logging. Noch kein
Netzwerk-Server — der ist der nächste Schritt (siehe Roadmap).

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

13. **Request/Response-Typen + reiner Handler.** `struct Request`, `struct
    Response`, `handle(&Request) -> Response` mit Routing (`GET /api/health` →
    200, sonst 404). Kein Socket — voll unit-testbar. Statuscodes als Typen.
14. **TCP-Schale (I/O-Adapter).** `serve_one()` nimmt eine Verbindung an, liest
    Bytes → `Request`, ruft `handle`, schreibt `Response`. Lese-/Schreibfehler
    als `Result`, malformed → `400`. Integrationstest auf freiem Port (`:0`),
    genau eine Verbindung, damit der Test sauber endet.
15. **Accept-Loop + ein Thread pro Verbindung** (`Arc`, `Send`/`Sync`).
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
