# 03 – Structs und Methoden

## Ziel

Daten und Verhalten ähnlich wie bei einer Java-Klasse bündeln.

```rust
struct User {
    name: String,
}

impl User {
    fn greet(&self) { /* lesen */ }
    fn rename(&mut self, new_name: &str) { /* ändern */ }
}
```

## Merksätze

- `struct` definiert Datenfelder.
- `impl` enthält zugehörige Methoden.
- `&self` leiht ein Objekt zum Lesen.
- `&mut self` leiht es exklusiv zum Ändern.
- `pub` macht eine Funktion oder einen Typ außerhalb des Moduls sichtbar.
