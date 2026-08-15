use rust_server_learning::validation::{ItemName, ValidationError};

#[test]
fn accepts_a_valid_name() {
    let name = ItemName::parse("  widget  ".to_string()).unwrap();
    assert_eq!(name.into_inner(), "widget")
}

#[test]
fn rejects_empty_name() {
    assert_eq!(ItemName::parse("".to_string()), Err(ValidationError::Empty));
}

#[test]
fn rejects_whitespace_only_name() {
    assert_eq!(
        ItemName::parse(" ".to_string()),
        Err(ValidationError::Empty)
    )
}

#[test]
fn rejects_too_long_name() {
    assert_eq!(
        ItemName::parse("a".repeat(101)),
        Err(ValidationError::TooLong { actual: 101 })
    )
}
