# 13 – Request/Response-Typen + reiner Handler

## Ziel

Den HTTP-Kern als reine Logik bauen — Typen und ein Handler `Request →
Response`, komplett ohne Netzwerk. Voll unit-testbar. Ports-and-Adapters: der
reine Kern zuerst, die I/O-Schale (Lektion 14) später.

## Modul-Struktur

```
src/http/
  mod.rs       Fassade: mod-Deklarationen + pub use
  types.rs     Daten: Method, StatusCode, ContentType, Request, Response
  handle.rs   Verhalten: handle(&Request) -> Response
```

`mod.rs` re-exportiert mit `pub use types::{...}`, sodass der Benutzer
`http::Request` schreibt und die interne Dateistruktur nicht kennen muss
(Facade-Pattern). `handle.rs` greift via `use super::types::{...}` auf die
Geschwister-Typen zu.

## Design: Daten getrennt vom Verhalten

- `types.rs` = datenorientiert (die Domäne als Typen; DDD-Einschlag).
- `handle.rs` = funktional (reine Funktion, keine Seiteneffekte).
- Enums für geschlossene Mengen (`Method`, `StatusCode`, `ContentType`) —
  „make illegal states unrepresentable". `path` bleibt `String` (offene Menge).

Namen am Ökosystem-Standard (`http`-Crate): `StatusCode`, `Method` — kurz, kein
redundantes Präfix. Später (Lektion 18, `axum`) heißen die Typen genauso.

## StatusCode: Nummer abgeleitet, nicht gespeichert

```rust
impl StatusCode {
    pub fn code(&self) -> u16 {
        match self {
            StatusCode::OK => 200,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::UnsupportedMediaType => 415,
            // ...
        }
    }
}
```

Single Source of Truth: Variante und Code können nicht auseinanderlaufen. Der
exhaustive `match` erzwingt für jede neue Variante einen Arm. Zero-cost: der
Compiler optimiert das `match` zur Konstante.

## Routing auf (Methode, Pfad)

```rust
match (&request.method, request.path.as_str()) {
    (Method::Get, "/api/health") => Response::text(StatusCode::OK, "ok"),
    (_, "/api/health")           => /* 405: Pfad ok, Methode falsch */,
    _                            => /* 404: Pfad unbekannt */,
}
```

Reihenfolge entscheidet (Rust nimmt den ersten passenden Arm): spezifisch vor
allgemein. Drei semantisch getrennte Fälle: 200 / 405 (Methode falsch) / 404
(Pfad unbekannt). Ein naives einzelnes `_` würde 405 fälschlich als 404 melden.

## Dynamische Pfad-Segmente (`/api/item/:id`)

Ein `match`-Arm matcht nur Literale — `:id` ist keins. Extraktion von Hand mit
`strip_prefix` vor dem `match`:

```rust
if let Some(id) = request.path.strip_prefix("/api/item/") {
    return handle_item(&request.method, id);
}
```

Das ist genau die Arbeit, die `axum` später abnimmt (`/api/item/:id` als Route
mit automatischer Extraktion).

## Builder-Pattern mit `mut self`

Reicher `Request` (5 Felder) → schlanker `new` + verkettbare Setter:

```rust
pub fn with_content_type(mut self, ct: ContentType) -> Self {
    self.content_type = Some(ct);
    self
}
```

`mut self` (nicht `&mut self`): konsumiert self, gibt es modifiziert zurück →
Verkettung `Request::new(...).with_content_type(...)`. Zero-cost, kein Klonen.

## `impl Into<String>` für flexible Konstruktoren

```rust
pub fn json(status: StatusCode, body: impl Into<String>) -> Self {
    Self { ..., body: body.into() }
}
```

Nimmt `&str` UND `String` (z. B. aus `format!`). Löst die `&str`/`String`-
Reibung an der Wurzel — kein `&format!(...)` oder manuelles `.to_owned()` nötig.

## Content-Type-Validierung (Vorgriff auf Phase B)

POST prüft den Request-Content-Type, bevor er den Body verarbeiten würde:

```rust
match request.content_type {
    Some(ContentType::ApplicationJson) => /* 501 Not Implemented (Body noch nicht geparst) */,
    _ => Response::text(StatusCode::UnsupportedMediaType, "expected application/json"),
}
```

🔒 „Fail closed": unbekannten Input kontrolliert ablehnen (`415`), bevor man ihn
verarbeitet. Ehrlich: `501` statt `201`, weil der Body noch nicht verarbeitet
wird — echtes Body-Parsing kommt mit `serde` in Phase B.

## Wichtigstes Learning: grüne Tests ≠ korrekt

Zwei stille Bugs, die alle Tests bestanden:

1. `UnsupportedMediaType` gab `409` statt `415` — Tests prüften nur die
   *Variante* (`assert_eq!(status_code, ...)`), nie `code()`.
2. `{{...}}` in einem **Literal** (statt `format!`) erzeugte kaputtes JSON
   (`{{"error":...}}`) — kein Test prüfte den `body` der Fehler-Zweige.

`{{`/`}}` gehört NUR in `format!` (dort Platzhalter-Escaping). In normalen
Literalen bleibt `{` einfach `{`. Handgebautes JSON ist fehleranfällig und die
Fehler sind unsichtbar — genau die Motivation für `serde` (Phase B).

Konsequenz: nur was getestet wird, ist abgesichert. Body und `code()` in Tests
mit abdecken.
