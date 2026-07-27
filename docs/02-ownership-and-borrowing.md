# 02 – Ownership und Borrowing

## Ziel

Verstehen, warum Rust ohne Garbage Collector Speicher sicher verwalten kann.

## Merksätze

- Ein `String` hat genau einen Besitzer.
- Eine Übergabe ohne `&` kann den Besitz an eine Funktion verschieben.
- `&str` oder `&String` ist eine unveränderliche Ausleihe: Die Funktion darf
  lesen, aber der Besitzer bleibt derselbe.
- `&mut String` ist eine veränderliche Ausleihe: Die Funktion darf ändern,
  aber nur exklusiv.
- `mut` an einer Variable erlaubt dem Besitzer Änderungen.

## Java-Vergleich

Java-Referenzen können meist frei geteilt werden; die Garbage Collection räumt
später auf. Rust prüft Besitz und Ausleihen schon beim Kompilieren. Dadurch
verhindert Rust viele Null-, Datenrennen- und Lebensdauerfehler früh.
