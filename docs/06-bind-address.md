# 06 – Bind-Adresse

## Ziel

Host und Port als eine Adresse für einen späteren Server ausdrücken.

```rust
let config = ServerConfig::new("localhost", "8080").unwrap();
assert_eq!(config.bind_address(), "localhost:8080");
```

## Test zuerst

Der Test in `tests/server_config_test.rs` wurde vor der Methode geschrieben.
Er schlug zunächst fehl, weil `bind_address()` noch nicht existierte. Danach
wurde die kleinste passende Implementierung ergänzt:

```rust
pub fn bind_address(&self) -> String {
    format!("{}:{}", self.host, self.port)
}
```

`format!` erzeugt einen neuen `String`. Es ist ähnlich zu `String.format` in
Java oder einem Template-String in TypeScript.

Die Methode leiht die Konfiguration mit `&self` nur aus: Sie liest Host und Port
und verändert nichts.
