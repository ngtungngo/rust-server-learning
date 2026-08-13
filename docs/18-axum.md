# 18 – HTTP-APIs mit `axum`: Router, Handler, Layer, Migration

## Ziel

Der handgebaute Server-Stack aus Lektion 14–17 (`serve`, `select!`, `JoinSet`,
Registry, eigenes Request/Response-Parsing) wird durch **`axum`** ersetzt — das
idiomatische Rust-Web-Framework auf Basis von `tokio`, `tower` und `hyper`.
Bewusster Schritt: erst von Hand bauen, um zu *verstehen*, was ein Framework
intern tut; dann das Framework nutzen. Fast der gesamte L14–17-Code wird dadurch
abgelöst — er bleibt über die Tags `lesson-14` … `lesson-17` und `docs/14`–`17`
erhalten.

Etappen, jede getaggt:
1. axum-Dependency + erste Route `/api/health` — `lesson-18-health`
2. restliche Routen (Path-Extractor, Methoden-Routing, async sleep) — `lesson-18-routes`
3. `main` auf `axum::serve` + graceful shutdown — `lesson-18-serve`
4. per-Request-Timeout als `tower` `TimeoutLayer` — `lesson-18-timeout`
5. Cleanup (toten Stack löschen) + Doku — `lesson-18`

Nebenbei: CI (GitHub Actions) um `fmt --check`, `clippy -D warnings` und
Build-Cache gehärtet — `ci-hardening`.

## Was axum alles ersetzt

| handgebaut (L14–17) | axum |
|---|---|
| `parse_request` (Zeilen parsen) | intern via `hyper` |
| `to_http` (Response serialisieren) | `IntoResponse`-Trait |
| Routing (`match path`) | `Router::new().route(...)` |
| `serve` / `handle_connection` | `axum::serve(listener, router)` |
| graceful shutdown (`select!`/`JoinSet`) | `.with_graceful_shutdown(future)` |
| per-Verbindung-Timeout (`tokio::time::timeout`) | `tower_http::TimeoutLayer` |
| eigene `Method`/`StatusCode`/`Request`/`Response` | `axum::http::*` / Extractors |

Was **nicht** ersetzt wird: `ServerConfig`, `parse_port`, `ConfigError` — die
Konfigurations-/Validierungslogik ist framework-unabhängig und bleibt.

## Router und Handler

```rust
pub fn router(timeout: Duration) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/item/{id}", get(get_item))
        .route("/api/item", post(create_item))
        .route("/api/slow", get(slow))
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, timeout))
}

async fn health() -> &'static str { "ok" }
```

- **`router()` ist synchron, nicht `async`.** Es steckt nur die Route-Tabelle
  zusammen (reine Konstruktion, kein Warten). Ein `async fn router()` gäbe ein
  `Future<Output = Router>` zurück statt eines `Router` — dann schlägt
  `.oneshot(...)` fehl. `async` gehört nur an die **Handler**, die bei jedem
  Request laufen und `.await`en dürfen.
- **Handler + `IntoResponse`:** ein Handler gibt irgendetwas zurück, das axum in
  eine HTTP-Response verwandeln kann. `&'static str` → `200 OK`,
  `text/plain`. Ein Tupel `(StatusCode, [(header, value)], body)` setzt Status +
  Header explizit. Das ersetzt `to_http` — kein manuelles Formatieren mehr.
- **axum 0.8 nutzt `{id}`** in der Route (nicht `:id` wie 0.7).
- **`404`/`405` automatisch:** unbekannter Pfad → 404, existierender Pfad mit
  falscher Methode → 405. Die alten expliziten Arme aus `handle` entfallen.

## Extractors: typsichere Request-Teile

```rust
async fn get_item(Path(id): Path<String>) -> impl IntoResponse {
    let body = format!(r#"{{"id":"{id}"}}"#);
    (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], body)
}
```

`Path(id): Path<String>` zieht ein URL-Segment heraus. 🔒 Sicherheits-Bonus des
Typsystems: `Path<u32>` würde axum das Segment **validieren** lassen —
`/api/item/abc` gäbe automatisch `400`, der Handler liefe gar nicht erst. Hier
`String` gewählt (Verhaltensparität zum alten Handler, der jede ID akzeptierte).

`HeaderMap` als Extractor liest Header:
```rust
async fn create_item(headers: HeaderMap) -> impl IntoResponse {
    let is_json = headers.get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"));
    if is_json { (StatusCode::NOT_IMPLEMENTED, "...").into_response() }
    else { (StatusCode::UNSUPPORTED_MEDIA_TYPE, "...").into_response() }
}
```
- **`starts_with("application/json")`**, nicht `==`: Content-Type ist oft
  `application/json; charset=utf-8`. 🔒 Ein striktes `==` würde legitime Requests
  mit charset-Suffix fälschlich mit 415 ablehnen.
- **`.into_response()`** vereinheitlicht die zwei Zweige zu einem gemeinsamen
  `Response`-Typ (sonst hätten die Tupel unterschiedliche Typen).

## ⚡ async sleep statt blockierendem sleep

```rust
async fn slow() -> impl IntoResponse {
    tokio::time::sleep(Duration::from_millis(500)).await;
    "slow ok"
}
```
Der alte Handler nutzte `std::thread::sleep` — das **blockiert den Runtime-
Worker**. `tokio::time::sleep().await` gibt den Worker frei, während die 500 ms
laufen → andere Requests laufen weiter. Das ist zugleich Voraussetzung dafür,
dass der `TimeoutLayer` überhaupt greifen kann (nächster Abschnitt).

## `axum::serve` + graceful shutdown

```rust
axum::serve(listener, router(Duration::from_secs(30)))
    .with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; })
    .await?;
```
- **`axum::serve(listener, ...)`** nimmt den `TcpListener` **per Wert** (nicht
  `&listener` wie das handgebaute `serve`) und führt den Accept-Loop selbst.
- **`.with_graceful_shutdown(future)`** ist exakt L17/4b: nach dem Signal keine
  neuen Verbindungen, laufende werden ausgewartet. axum nutzt intern dieselbe
  Mechanik (`select!` auf accept + Signal, dann in-flight-Tasks abwarten), die in
  L17 von Hand gebaut wurde. Diese eine Zeile ersetzt drei Lektionen Code —
  verständlich, *weil* der handgebaute Weg vorher durchgearbeitet wurde.

## Middleware als Layer: der Timeout

```rust
.layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, timeout))
```
- **`.layer(...)`** wickelt den ganzen Router in Middleware — Querschnitts-
  Funktionalität (Timeout, später Logging, Auth, Rate-Limiting) an *einer* Stelle
  statt in jedem Handler. Das ist der `tower::Service`-Stack: Layer um Layer um
  den Handler. Bei mehreren Layern zählt die Reihenfolge (außen → innen).
- 🔒 Ersetzt L17/4c (per-Verbindung-Timeout, Slow-Loris-Abwehr) deklarativ. Gibt
  `408 Request Timeout` bei Ablauf. Der Statuscode ist mit
  `with_status_code(...)` **explizit** (die alte `TimeoutLayer::new` ist
  deprecated — und hätte im CI-`clippy -D warnings` die Pipeline rot gemacht).
- Der Timeout ist **Parameter** von `router()` (DI, wie `shutdown`/`timeout` in
  L17): `main` gibt 30 s, Tests geben kurze Werte → testbar.

## Testen: `oneshot` statt echtem Server

```rust
let app = router(Duration::from_secs(30));
let response = app
    .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
    .await.unwrap();
assert_eq!(response.status(), StatusCode::OK);
let body = response.into_body().collect().await.unwrap().to_bytes();
assert_eq!(&body[..], b"ok");
```
Ein `Router` **ist** ein `tower::Service` — `oneshot` schickt genau eine Anfrage
durch, ohne Netzwerk. Der ganze L14–17-Testaufwand (echter `TcpListener`,
Client-Thread, `multi_thread`-Runtime gegen Deadlock, Timing-Messung) entfällt.
Der Response-Body ist ein async Stream → `.collect().await...to_bytes()`. Der
Timeout-Test beweist: `/api/slow` (500 ms) unter 50-ms-Budget → `408`.

## Cleanup

Nach der Migration ist der handgebaute Stack toter Code (nur noch von alten
Tests referenziert, nicht von `main`). Entfernt: `src/server.rs`, `src/http/`,
`tests/http_test.rs`, `tests/server_test.rs`, `tests/server_shutdown_test.rs`,
`tests/common/`. Dazu das `io-util`-tokio-Feature (war nur für die
`AsyncReadExt`/`AsyncWriteExt` im gelöschten `handle_connection`). ⚡/🔒 kleinere
Dependency-Fläche = schnellere Builds + weniger Angriffsfläche. Alles bleibt über
git-Tags erhalten — löschen heißt hier nicht verlieren.

## Ehrliche Einordnung

- Dies ist der Punkt, an dem das Projekt von „HTTP selbst sprechen" zu „Framework
  nutzen" wechselt. Der handgebaute Code war kein Umweg: das Verständnis von
  `select!`/`pin!`/`JoinSet`/kooperativem Cancel erklärt jetzt, was axums
  Einzeiler intern tun — und wo ihre Grenzen liegen.
- `POST /api/item` gibt weiterhin nur `501`/`415` — echtes Deserialisieren des
  JSON-Bodys kommt in Lektion 19 (`serde`). Der JSON-String in `get_item` ist
  noch von Hand gebaut; auch das löst `serde` ab.

## Getestet

- `health_returns_200_ok`, `get_item_returns_id_as_json`, `slow_returns_ok`,
  `create_item_json_returns_501`, `create_item_without_json_returns_415` — Routen
  in-process via `oneshot`.
- `slow_route_times_out_with_408` — der `TimeoutLayer` bricht `/api/slow` unter
  kurzem Budget mit `408` ab.
