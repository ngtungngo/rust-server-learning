# 20 – In-Memory-Store: shared State + CRUD

## Ziel

Bisher warf `create_item` das erzeugte Item weg und `get_item` erfand die ID aus
der URL — kein Zustand. Jetzt kommt ein **Store**, den sich alle Handler teilen:
`AppState` mit `Arc<RwLock<HashMap<String, Item>>>`, eingehängt über axums
`State`-Extractor. Damit volles **CRUD** (Create, Read, List, Update, Delete) mit
echten, server-vergebenen IDs. Erste Persistenz — noch im Speicher, aber mit der
Semantik, die später die DB erbt.

Etappen, getaggt:
1. Store-Gerüst + `create`/`get` (echtes Speichern/Lesen, `404`) — `lesson-20-store`
2. `list`/`delete`/`update` — `lesson-20-crud`

## Der geteilte Zustand: `AppState`

```rust
#[derive(Clone)]
pub struct AppState {
    items: Arc<RwLock<HashMap<String, Item>>>,
}
```

Drei Schichten, jede mit Grund:
- **`Arc`** — geteilter Besitz über Tasks. Jeder Handler-Aufruf bekommt einen
  *Clone* des `AppState`, aber `Arc::clone` kopiert nur den Zeiger (Zähler +1),
  **nicht** die Daten. Alle Clones zeigen auf *denselben* Store.
- **`RwLock`** — geregelter Zugriff: viele gleichzeitige Leser ODER ein
  Schreiber. ⚡ Für einen überwiegend gelesenen Store besser als `Mutex` (dort
  serialisiert sich auch jedes Lesen).
- **`HashMap<String, Item>`** — der Store: ID → Item.

**`#[derive(Clone)]`** braucht axums `State`-Extractor. **`items` ist privat** —
alle Zugriffe laufen über Methoden von `AppState`; die Lock-Disziplin bleibt in
`store.rs` gekapselt, kein Handler fasst den Lock direkt an. Das ist der Keim des
späteren `Repository`-Traits (Phase C).

## `std::sync::RwLock` — warum nicht der tokio-Lock?

Regel: **lebt der Guard über einen `.await`-Punkt?** Hier nie — jede Methode
lockt, macht *eine* synchrone Map-Operation, der Guard fällt am Statement-Ende:

```rust
self.items.write().unwrap().insert(id, item.clone()); // lock → insert → guard fällt
```

Kein `.await` dazwischen → `std::sync::RwLock` ist korrekt und billiger als
`tokio::sync::RwLock`. Ausführliche Begründung samt Deadlock-Szenario und
Diagrammen: [`concepts/rwlock-std-vs-tokio.md`](concepts/rwlock-std-vs-tokio.md).

## `State`-Extractor + `.with_state`

```rust
pub fn router(timeout: Duration, state: AppState) -> Router {
    Router::new()
        .route("/api/item/{id}", get(get_item).delete(delete_item).put(update_item))
        .route("/api/item", post(create_item).get(list_items))
        // ...
        .with_state(state)
}

async fn get_item(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse { ... }
```

`.with_state(state)` hängt den Zustand an den Router; jeder Handler, der ihn
braucht, fordert ihn via `State(state): State<AppState>` an. **Methoden-Chaining**
auf einem Pfad (`get(...).delete(...).put(...)`, `post(...).get(...)`) bedient
mehrere HTTP-Methoden an derselben Route — die REST-Ressourcen-Sicht.

⚠️ **Extractor-Reihenfolge:** `Json` konsumiert den Request-Body und muss
**zuletzt** stehen. `State`/`Path` sind billig und kommen davor. Falsche
Reihenfolge → Compile-Fehler (in `create_item`, `update_item`).

## Die CRUD-Operationen

| HTTP | Handler | Store-Methode | Erfolg | Fehlt |
|---|---|---|---|---|
| `POST /api/item` | `create_item` | `insert(name) -> Item` | `201 Created` | — |
| `GET /api/item/{id}` | `get_item` | `get(id) -> Option<Item>` | `200` + Item | `404` |
| `GET /api/item` | `list_items` | `list() -> Vec<Item>` | `200` + Array | — |
| `PUT /api/item/{id}` | `update_item` | `update(id, name) -> Option<Item>` | `200` + Item | `404` |
| `DELETE /api/item/{id}` | `delete_item` | `delete(id) -> bool` | `204` | `404` |

Kernmuster:
- **`Option`/`bool` → Statuscode:** `get`/`update` geben `Option<Item>` (`Some`
  → 200, `None` → 404); `delete` gibt `bool` (`true` → 204, `false` → 404). Die
  „nicht da → 404"-Semantik ist bei `get`/`update`/`delete` konsistent.
- **`.cloned()` / `.clone()`:** Items werden aus der Map *herauskopiert*, damit
  kein Guard/Referenz den Lock überlebt (`get`, `list`, `update`, `insert`).
- **`get_mut(id)?`** in `update`: `?` auf `Option` — `None` → sofort raus → 404;
  sonst in-place-Mutation unter dem write-Lock.
- **`StatusCode` allein ist `IntoResponse`** — `delete_item` gibt nackte
  Statuscodes (kein Body); `204 No Content` ist genau das.

## Server-vergebene IDs: `uuid` v4

```rust
let id = Uuid::new_v4().to_string();
```
Der Server vergibt die ID, nicht der Client (L19-Prinzip). 🔒 **v4 = zufällig,
nicht durchzählbar** — anders als `1, 2, 3, …`, wo ein Angreifer alle IDs
enumerieren könnte (IDOR-Schutz). PUT legt bewusst *nicht* an, wenn die ID fehlt
(kein Upsert) — sonst könnte der Client die ID diktieren.

## Testtechnik: shared State beweisen

`app.oneshot(...)` konsumiert den Router. Für POST-dann-GET brauchen zwei Router
denselben Store — geht via `state.clone()` (klont nur den `Arc`):

```rust
let state = AppState::new();
// POST über router(.., state.clone()) ...
// GET  über router(.., state.clone()) → findet das eben angelegte Item
```
Wäre der State nicht geteilt, wäre der Store nach dem POST leer. Zusätzlich prüft
**jeder mutierende Test den Store direkt** (`state.get(...)` nach delete/update) —
nicht nur den Statuscode. „Status 204" allein würde auch ein No-Op liefern; der
`get`-Check beweist die echte Änderung. (Grüne Tests ≠ korrekt.)

Nicht-deterministische Werte (die zufällige UUID) werden nicht wörtlich
assertet, sondern per `serde_json::Value` geparst und die `id` dynamisch
weiterverwendet.

## Ehrliche Grenzen

- 🔒/⚡ **`list()` gibt *alles* zurück** — bei großem Store ein DoS-/Performance-
  Problem. Ein echter Server paginiert (`?limit=&offset=`). Bewusst weggelassen.
- **Alles im RAM** — Neustart = Datenverlust. Persistenz kommt in Phase C
  (`sqlx`/DB). Der Store ist bewusst so gebaut, dass die Methoden (`insert`/`get`/
  `list`/`update`/`delete`) später zu einem `Repository`-Trait werden, das DB
  *und* In-Memory implementieren — Ports-and-Adapters.
- **Keine inhaltliche Validierung** — leerer/überlanger `name` kommt durch
  (serde prüft nur Struktur). Eigene Schicht = L22.

## Getestet

- `create_then_get_roundtrip` — POST dann GET über geteilten State (beweist
  `Arc`-Sharing).
- `get_unknown_item_returns_404`, `delete_unknown_item_returns_404`,
  `update_unknown_item_returns_404` — die „nicht da → 404"-Fälle.
- `delete_existing_item_returns_204`, `update_existing_item_returns_200` — inkl.
  Store-Änderungs-Assert.
- `list_returns_all_items` — Anzahl + Mitgliedschaft, reihenfolge-unabhängig.
