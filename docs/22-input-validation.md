# 22 – Input-Validierung als eigene Schicht ("parse, don't validate")

## Ziel

Der offene Faden seit L19: `serde` prüft nur **Struktur**, nicht **Semantik**.
`{"name":""}`, `{"name":"   "}` oder ein 10-MB-`name` kommen strukturell korrekt
an und landen ungeprüft im Store. Diese Lektion schließt die Lücke mit einer
Validierungs-Schicht **an der HTTP-Grenze** — umgesetzt als eigener Typ, nicht als
verstreute `if`-Checks.

Etappen, getaggt:
1. Der Typ `ItemName` + Unit-Tests — `lesson-22-itemname`
2. Handler verdrahten (`400` an der Grenze) — `lesson-22`

## "Parse, don't validate": der Typ trägt die Garantie

```rust
#[derive(Debug, PartialEq)]
pub struct ItemName(String); // Feld privat!

impl ItemName {
    pub fn parse(input: String) -> Result<ItemName, ValidationError> {
        let name = input.trim();
        if name.is_empty() {
            return Err(ValidationError::Empty);
        }
        if name.len() > 100 {
            return Err(ValidationError::TooLong { actual: name.len() });
        }
        Ok(ItemName(name.to_string()))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}
```

Der Kern-Gedanke: **`parse` gibt den Wrapper zurück, nicht den rohen `String`.**
Weil das Feld privat ist und `parse` der einzige Konstruktor, gilt:

> Wer ein `ItemName` in der Hand hält, hält per Typsystem einen **geprüften** Namen.
> Ein ungültiger Name ist kein "Fehlerfall den man behandeln muss", sondern ein
> Wert, der gar nicht erst konstruiert werden kann.

Das ist der Unterschied zu "validate":
- **validate**: prüfen, dann die *rohen* Daten weiterreichen → die Information
  "ist geprüft" geht verloren, jede Schicht müsste neu prüfen.
- **parse**: prüfen *und in einen Typ überführen*, der die Prüfung bezeugt → die
  Garantie steckt ab jetzt im Typ ("make illegal states unrepresentable").

### `into_inner` — Namenskonvention als Vertrag

`into_inner` nimmt `self` (konsumiert), nicht `&self`. Rusts Präfixe verraten dem
Aufrufer die Kosten: `as_` (billige Referenz), `to_` (Klon, Original bleibt),
**`into_` (konsumiert, gibt Besitz weiter)**. Der Name ist kein Kosmetik-Detail —
er ist die Vertragsklausel.

## Getippter Fehler mit `thiserror`

```rust
#[derive(Debug, PartialEq, Error)]
pub enum ValidationError {
    #[error("name must not be empty")]
    Empty,
    #[error("name is too long: {actual} bytes (max 100)")]
    TooLong { actual: usize },
}
```

`TooLong` trägt `actual`, **nicht** `max`. Regel: Fehler-Felder tragen, was
*variiert und diagnostisch hilft* — nicht, was konstant und schon bekannt ist. Das
Limit (100) ist eine Konstante; sie gehört in den Meldungstext, nicht als Datenfeld.
Der abgelehnte Ist-Wert (`actual`) ist die eigentliche Diagnose-Info.

## Verdrahtung an der Grenze: `400` vor `409`

```rust
async fn create_item(State(state): State<AppState>, Json(input): Json<CreateItem>)
    -> impl IntoResponse
{
    let name = match ItemName::parse(input.name) {
        Ok(name) => name,
        Err(err) => return (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    };
    match state.insert(name.into_inner()) {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(err) => (StatusCode::CONFLICT, err.to_string()).into_response(),
    }
}
```

- **Guard-Clause mit `return`**: der Fehlerfall verlässt den Handler sofort — kein
  verschachteltes `match`.
- **Reihenfolge ist Absicht**: erst validieren (`400`), *dann* Store/Eindeutigkeit
  (`409`). Ungültiges erreicht die Domäne nie. 🔒 "Nie ungeprüfte Daten in die
  Domäne."
- **`update_item` validiert identisch** — ein häufiger Bug ist, nur den Create-Pfad
  zu prüfen und den Update-Pfad zu vergessen. Beide Pfade parsen jetzt.

## Security & Performance (Querschnitt)

- 🔒 **Validierung ≠ Injection-Abwehr.** Wir bauen bewusst *kein* Zeichen-
  Blacklisting ein. Injection entsteht nicht beim *Reinnehmen*, sondern beim
  *Rausgeben an einen Interpreter* (SQL, HTML, Shell) — und wird **an der Senke**
  gelöst: parametrisierte Queries (Phase C, `sqlx`), Output-Encoding beim Rendern.
  Ein `name`-Blacklisting wäre Security-Theater am falschen Ort und würde legitime
  Namen wie `O'Brien` beschädigen. Aktuell gibt es ohnehin keine Injection-Fläche
  (der `name` geht nur in eine `HashMap` und via `serde_json` korrekt escaped als
  JSON wieder raus).
- 🔒/⚡ **Das Längenlimit (100 Bytes) ist Ressourcenschutz**, kein Injection-Schutz:
  eine Obergrenze gegen Speicher-DoS durch riesige Eingaben. Bytes statt `chars`,
  weil Bytes das echte Speichermaß sind (ein 4-Byte-Emoji zählt als 4).

## Ehrliche Grenzen

- Validierung sitzt **im Handler** (an der HTTP-Grenze). `state.insert` nimmt weiter
  einen `String`, kein `ItemName` — theoretisch könnte anderer Code den Store mit
  ungeprüftem Namen füttern. Stärker wäre, `insert(name: ItemName)` zu fordern
  (Garantie bis in die Domäne). Bewusst klein gehalten; guter nächster Schritt.
- Nur zwei Regeln (nicht-leer, Längenlimit). Ein echter Server hätte oft mehr
  (erlaubte Zeichen, Normalisierung); das Muster (`parse` → getippter Fehler →
  `400`) skaliert aber unverändert.

## Getestet

- Unit (`tests/validation_test.rs`): `accepts_a_valid_name` (prüft via
  `into_inner()` zugleich das **Trimming** von `"  widget  "` → `"widget"`),
  `rejects_empty_name`, `rejects_whitespace_only_name`, `rejects_too_long_name`
  (`101` Bytes → `TooLong { actual: 101 }`).
- HTTP (`tests/item_crud_test.rs`): `create_empty_name_returns_400` (+ Store bleibt
  leer), `update_empty_name_returns_400` (+ alter Name unverändert — kein halber
  Update).
