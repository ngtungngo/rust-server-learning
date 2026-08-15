# 21 – Ressourcen-Statuscodes: `409 Conflict` + erster fachlicher Fehler

## Ziel

Die meisten Statuscodes standen schon (`200`/`201`/`204`/`404`, Methoden-Chaining).
Das Neue an dieser Lektion ist **`409 Conflict`** — und der zwingt etwas heraus,
das der Store bisher nicht hatte: eine **fachliche Regel, die verletzt werden
kann**. Bisher konnte `insert` nie scheitern (gab immer ein `Item`). Jetzt gilt:
**`name` ist eindeutig.** Ein zweiter `POST` mit schon existierendem Namen → `409`
statt `201`.

Das ist die Blaupause für Phase C: dort verletzt eine DB einen `UNIQUE`-Constraint
und der Fehler muss auf HTTP gemappt werden (L25). Wir üben das Muster hier im
Kleinen, ohne DB.

## `insert` bekommt seine erste Fehlerbedingung

Die Signatur wechselt von `Item` zu `Result<Item, StoreError>`:

```rust
pub fn insert(&self, name: String) -> Result<Item, StoreError> {
    let mut items = self.items.write().unwrap();
    if items.values().any(|item| item.name == name) {
        return Err(StoreError::DuplicateName(name)); // Regel an der Grenze
    }
    let id = Uuid::new_v4().to_string();
    let item = Item { id: id.clone(), name };
    items.insert(id, item.clone());
    Ok(item)
}
```

Sobald eine Operation scheitern *kann*, ist `Result` der ehrliche Rückgabetyp —
nicht ein `Option` oder ein stiller No-Op. Der Aufrufer *muss* den Fehlerfall
behandeln (Rust erzwingt das über `#[must_use]` auf `Result`).

## Getippter Fehler mit `thiserror`

Kein generischer `Error` (Projekt-Regel) — ein eigenes Enum, wie schon
`ConfigError`:

```rust
#[derive(Debug, PartialEq, Error)]
pub enum StoreError {
    #[error("an item with name '{0}' already exists")]
    DuplicateName(String),
}
```

`#[error(...)]` erzeugt die `Display`-Impl (intern ein `write!` — siehe L11). Das
Enum lebt in `store.rs`, weil der Konflikt eine **Store-Domänen**-Aussage ist,
nicht HTTP. Die Übersetzung „welcher Fehler → welcher Statuscode" macht erst der
Handler. Diese Trennung trägt später Phase C: `StoreError` wächst um DB-Fälle,
ohne dass HTTP-Wissen in den Store sickert.

## Fehler → HTTP im Handler

```rust
async fn create_item(
    State(state): State<AppState>,
    Json(input): Json<CreateItem>,
) -> impl IntoResponse {
    match state.insert(input.name) {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(), // 201
        Err(err) => (StatusCode::CONFLICT, err.to_string()).into_response(), // 409
    }
}
```

Gleiches Muster wie `get`/`update` (dort `Option` → Statuscode), hier `Result` →
Statuscode. Der Handler ist der Ort, an dem Domänen-Ergebnisse auf das
HTTP-Vokabular abgebildet werden.

## Ein Guard über Prüfung UND Insert (TOCTOU)

Bewusst stehen **Prüfung und Insert unter demselben `write()`-Guard**:

```rust
let mut items = self.items.write().unwrap(); // ein Guard
if items.values().any(...) { return Err(...); }  // Time Of Check
items.insert(id, item.clone());                   // Time Of Use
```

Würde man erst lesen, den Lock freigeben, dann schreiben, entstünde ein
**TOCTOU-Race** (Time-Of-Check-To-Time-Of-Use): zwei parallele Requests mit
demselben Namen könnten *beide* die Prüfung passieren, bevor einer schreibt — und
beide legen an. Ein Guard über beide Schritte macht Prüfung+Insert atomar und
schließt das Fenster. (🔒 Diese Klasse von Races ist ein klassischer
Sicherheits-Fehler weit über diesen Store hinaus — z. B. „prüfe Berechtigung,
dann öffne Datei".)

## Security & Performance (Querschnitt)

- 🔒 **Grenz-Entscheidung:** Der Server weist inkonsistente Daten an der
  API-Grenze ab (`409`), statt still zu überschreiben. Konflikt sichtbar machen
  statt Daten verlieren.
- ⚡ **Trade-off:** Die Eindeutigkeitsprüfung ist ein linearer Scan
  (`values().any(...)`, O(n)) unter dem write-Lock. Bei einem RAM-Store mit
  wenigen Items egal; in der DB später ein **Index** auf der Spalte (O(log n) und
  vom DB-Constraint erzwungen, nicht von Anwendungscode).

## Ehrliche Grenzen

- Eindeutigkeit gilt nur **innerhalb dieses Prozesses**. Zwei Server-Instanzen
  auf demselben Datensatz hätten je ihre eigene `HashMap` — die Regel bräuchte
  einen gemeinsamen Store (DB-Constraint). Genau das kommt mit Phase C.
- `PUT` prüft (noch) **nicht** auf Namenskollision — ein Update könnte einen
  Namen doppeln. Bewusst klein gehalten; wäre der nächste konsequente Schritt.

## Getestet

- `create_duplicate_name_returns_409` — zweimal `POST` mit demselben Namen über
  geteilten State: erster → `201`, zweiter → `409`, und `state.list().len() == 1`
  (beweist: der Store hat *nicht* doppelt angelegt, nicht nur der Statuscode
  stimmt).
