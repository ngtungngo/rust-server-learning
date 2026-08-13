# 19 – `serde` + JSON: Serialize/Deserialize, Json<T>, Input-≠-Output

## Ziel

JSON typsicher statt von Hand. Bisher baute `get_item` seinen JSON-String mit
`format!` und setzte den Content-Type-Header manuell; `create_item` prüfte nur
den Content-Type und las den Body gar nicht. `serde` (das De-facto-Standard-Crate
für Datenkonvertierung) + axums `Json<T>` lösen beides ab. Erster Punkt der
Phase B (REST/CRUD).

Zwei Etappen, getaggt:
1. Response serialisieren: `get_item` → `Json<Item>` — `lesson-19-serialize`
2. Request deserialisieren: `create_item` → `Json<CreateItem>` — `lesson-19-deserialize`

## `serde` in einem Satz

`serde` = **ser**ialize/**de**serialize. Zwei Traits, per `#[derive(...)]`
abgeleitet:
- **`Serialize`**: Rust-Struct → JSON (Rückgabe an den Client)
- **`Deserialize`**: JSON → Rust-Struct (Body vom Client lesen)

`serde` definiert die Traits, `serde_json` macht die JSON-Arbeit. axum ist darauf
gebaut: `Json<T>` ist beides — Response *und* Extractor.

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Domänentypen getrennt von der HTTP-Schicht

Neue Datei `src/models.rs` — die Datentypen leben *nicht* bei den Handlern,
sondern als eigene Domäne (Ports-and-Adapters: Daten unabhängig vom Transport).
Ab Phase C (DB) brauchen sie sowohl Handler als auch Repository.

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Item {
    pub id: String,
}

#[derive(Deserialize)]
pub struct CreateItem {
    pub name: String,
}
```

## Input ≠ Output — zwei Typen, kein Zufall

`Item` (Ausgabe, `Serialize`) und `CreateItem` (Eingabe, `Deserialize`) sind
bewusst getrennt:

- **Der Client schickt kein `id`** — die vergibt der Server (später aus der DB).
  🔒 Würde `create_item` ein `Item { id }` erwarten, könnte der Client die ID
  diktieren — IDs sind serverkontrolliert, das ist eine echte Sicherheitsgrenze.
- **Input-Typ ≠ Output-Typ** ist das saubere Muster: „was reinkommt" vs. „was
  rausgeht". In echten APIs divergieren die stark (Passwort rein, nie raus).

## Serialisieren: `get_item` → `Json<Item>`

```rust
async fn get_item(Path(id): Path<String>) -> Json<Item> {
    Json(Item { id })
}
```

`Json<Item>` ist `IntoResponse`: serialisiert `Item` via `serde_json` **und**
setzt `Content-Type: application/json` automatisch. Der manuelle Header-Tupel und
das `format!(r#"{{"id":"{id}"}}"#)` fallen weg — kein Escaping-Risiko, kein
Tippfehler im JSON möglich. `Item { id }` ist field-init-shorthand für
`Item { id: id }`. Standardstatus ist `200`.

Das Verhalten bleibt identisch (`{"id":"42"}`) — der **bestehende Test** ist die
Charakterisierung, die das beweist: Refactor unter grünem Test, Verhalten erhalten.

## Deserialisieren: `create_item` → `Json<CreateItem>`

```rust
async fn create_item(Json(input): Json<CreateItem>) -> impl IntoResponse {
    let item = Item { id: input.name }; // placeholder until the store (L20)
    (StatusCode::CREATED, Json(item))
}
```

- **`Json<CreateItem>` als Extractor**: axum liest den Body und deserialisiert ihn
  zu `CreateItem`. 🔒 Grenze-Validierung geschenkt: fehlt der Header
  `application/json`, gibt axum automatisch `415 Unsupported Media Type`; ist das
  JSON ungültig oder ein Feld fehlt, `400`/`422`. Das ersetzt die alte manuelle
  `HeaderMap`-Content-Type-Prüfung komplett — der behaltene `415`-Test beweist,
  dass `Json` diese Prüfung übernommen hat.
- **`(StatusCode::CREATED, Json(item))`**: Tupel aus Status + Body. `201 Created`
  ist der korrekte REST-Status für „Ressource angelegt" (nicht `200`).

## Ehrliche Grenze: Struktur ≠ Semantik

`Json`/`serde` validiert nur die **Struktur** — „ist es valides JSON mit den
richtigen Feldern und Typen?". *Nicht* die **Semantik**: `{"name":""}` (leer) oder
ein 10-MB-`name` kommen sauber durch. 🔒 Die inhaltliche Prüfung (leerer Name →
`400`, Längenlimit gegen DoS) ist eine eigene Schicht — genau Lektion 22
(„Input-Validierung als eigene Schicht"). Bewusst noch nicht hier gebaut.

Zweite Grenze, ehrlich: `create_item` **persistiert nichts** — es gibt keinen
Store. Das `Item` wird erzeugt und zurückgegeben, aber nicht gespeichert; die
`id` ist ein Platzhalter (`= input.name`). Echte ID-Vergabe + Persistenz kommen
mit dem In-Memory-Store in Lektion 20.

## Getestet

- `get_item_returns_id_as_json` (unverändert) — beweist, dass der `Json<Item>`-
  Refactor das Verhalten `{"id":"42"}` erhält.
- `create_item_returns_201_with_json_body` — POST mit `{"name":"widget"}` →
  `201` + Body `{"id":"widget"}`.
- `create_item_without_json_returns_415` (behalten) — POST ohne
  `application/json` → axums `Json`-Extractor lehnt mit `415` ab.
