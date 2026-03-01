use super::*;

// ── PHPDOC_TYPE_KEYWORDS ────────────────────────────────────────

#[test]
fn scalar_types_is_subset_of_phpdoc_type_keywords() {
    for entry in SCALAR_TYPES {
        assert!(
            PHPDOC_TYPE_KEYWORDS.contains(entry),
            "SCALAR_TYPES entry {:?} is missing from PHPDOC_TYPE_KEYWORDS",
            entry
        );
    }
}

#[test]
fn phpdoc_type_keywords_has_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for entry in PHPDOC_TYPE_KEYWORDS {
        assert!(
            seen.insert(entry),
            "PHPDOC_TYPE_KEYWORDS contains duplicate entry {:?}",
            entry
        );
    }
}

// ── replace_self_in_type ────────────────────────────────────────

#[test]
fn no_keywords_returns_unchanged() {
    assert_eq!(replace_self_in_type("string", "App\\User"), "string");
    assert_eq!(
        replace_self_in_type("Collection<int, string>", "App\\User"),
        "Collection<int, string>"
    );
}

#[test]
fn replaces_self() {
    assert_eq!(replace_self_in_type("self", "App\\User"), "App\\User");
}

#[test]
fn replaces_static() {
    assert_eq!(replace_self_in_type("static", "App\\User"), "App\\User");
}

#[test]
fn replaces_this() {
    assert_eq!(replace_self_in_type("$this", "App\\User"), "App\\User");
}

#[test]
fn replaces_in_union_type() {
    assert_eq!(
        replace_self_in_type("self|null", "App\\User"),
        "App\\User|null"
    );
    assert_eq!(
        replace_self_in_type("string|static|int", "App\\User"),
        "string|App\\User|int"
    );
    assert_eq!(
        replace_self_in_type("$this|null", "App\\User"),
        "App\\User|null"
    );
}

#[test]
fn replaces_nullable_keyword() {
    assert_eq!(replace_self_in_type("?self", "App\\User"), "?App\\User");
    assert_eq!(replace_self_in_type("?static", "App\\User"), "?App\\User");
    assert_eq!(replace_self_in_type("?$this", "App\\User"), "?App\\User");
}

#[test]
fn replaces_inside_generic_type() {
    assert_eq!(
        replace_self_in_type("Collection<self>", "App\\User"),
        "Collection<App\\User>"
    );
    assert_eq!(
        replace_self_in_type("array<int, static>", "App\\User"),
        "array<int, App\\User>"
    );
    assert_eq!(
        replace_self_in_type("Promise<$this>", "App\\User"),
        "Promise<App\\User>"
    );
}

#[test]
fn does_not_replace_partial_word_self() {
    // "self" appears in "selfService" but should not be replaced.
    assert_eq!(
        replace_self_in_type("selfService", "App\\User"),
        "selfService"
    );
    assert_eq!(replace_self_in_type("myself", "App\\User"), "myself");
}

#[test]
fn does_not_replace_partial_word_static() {
    assert_eq!(
        replace_self_in_type("staticMethod", "App\\User"),
        "staticMethod"
    );
    assert_eq!(replace_self_in_type("nonstatic", "App\\User"), "nonstatic");
}

#[test]
fn does_not_replace_partial_word_this() {
    // "$thisArg" should not be matched.
    assert_eq!(replace_self_in_type("$thisArg", "App\\User"), "$thisArg");
}

#[test]
fn replaces_multiple_occurrences() {
    assert_eq!(
        replace_self_in_type("self|static|$this", "App\\User"),
        "App\\User|App\\User|App\\User"
    );
}

#[test]
fn callable_signature_keywords() {
    assert_eq!(
        replace_self_in_type("callable($this, mixed): $this", "App\\User"),
        "callable(App\\User, mixed): App\\User"
    );
}

#[test]
fn underscore_boundary_prevents_replacement() {
    assert_eq!(replace_self_in_type("self_type", "App\\User"), "self_type");
    assert_eq!(
        replace_self_in_type("static_method", "App\\User"),
        "static_method"
    );
}

#[test]
fn intersection_type() {
    assert_eq!(
        replace_self_in_type("self&Countable", "App\\User"),
        "App\\User&Countable"
    );
}

#[test]
fn empty_string() {
    assert_eq!(replace_self_in_type("", "App\\User"), "");
}

#[test]
fn keyword_at_end_of_generic() {
    assert_eq!(
        replace_self_in_type("Map<string, self>", "App\\Foo"),
        "Map<string, App\\Foo>"
    );
}
