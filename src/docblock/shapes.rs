//! Array shape and object shape parsing.
//!
//! This submodule handles parsing PHPStan/Psalm array shape and object
//! shape type strings into their constituent entries, and looking up
//! value types by key.
//!
//! All parsing is delegated to `PhpType::parse()` (which uses
//! `mago_type_syntax` internally), eliminating ~250 lines of
//! hand-rolled depth-tracking parsers.

use crate::php_type::PhpType;
use crate::types::ArrayShapeEntry;

/// Convert a `PhpType` shape entry list into `ArrayShapeEntry` values.
///
/// Positional entries (where `ShapeEntry.key` is `None`) are assigned
/// auto-incrementing numeric keys (`"0"`, `"1"`, …) to match the
/// behaviour callers expect.
fn shape_entries_to_array_entries(entries: &[crate::php_type::ShapeEntry]) -> Vec<ArrayShapeEntry> {
    let mut result = Vec::with_capacity(entries.len());
    let mut implicit_index: u32 = 0;

    for entry in entries {
        let key = match &entry.key {
            Some(k) => k.clone(),
            None => {
                let k = implicit_index.to_string();
                implicit_index += 1;
                k
            }
        };

        result.push(ArrayShapeEntry {
            key,
            value_type: entry.value_type.to_string(),
            optional: entry.optional,
        });
    }

    result
}

/// Unwrap nullable and extract an array shape from a `PhpType`.
///
/// Returns the shape entries if the (possibly nullable) type is an
/// array shape, or `None` otherwise.
fn unwrap_array_shape(ty: &PhpType) -> Option<&[crate::php_type::ShapeEntry]> {
    match ty {
        PhpType::ArrayShape(entries) => Some(entries),
        PhpType::Nullable(inner) => unwrap_array_shape(inner),
        _ => None,
    }
}

/// Unwrap nullable/intersection and extract an object shape from a `PhpType`.
///
/// Returns the shape entries if the (possibly nullable or intersected)
/// type contains an object shape, or `None` otherwise.
fn unwrap_object_shape(ty: &PhpType) -> Option<&[crate::php_type::ShapeEntry]> {
    match ty {
        PhpType::ObjectShape(entries) => Some(entries),
        PhpType::Nullable(inner) => unwrap_object_shape(inner),
        // `object{foo: int, bar: string}&\stdClass` parses as an
        // intersection; check each member.
        PhpType::Intersection(members) => members.iter().find_map(|m| unwrap_object_shape(m)),
        _ => None,
    }
}

/// Parse a PHPStan/Psalm array shape type string into its constituent
/// entries.
///
/// Handles both named and positional (implicit-key) entries, optional
/// keys (with `?` suffix), and nested types.
///
/// # Examples
///
/// - `"array{name: string, age: int}"` → two entries
/// - `"array{name: string, age?: int}"` → "age" is optional
/// - `"array{string, int}"` → positional keys "0", "1"
/// - `"array{user: User, items: list<Item>}"` → nested generics preserved
///
/// Returns `None` if the type is not an array shape.
pub fn parse_array_shape(type_str: &str) -> Option<Vec<ArrayShapeEntry>> {
    let parsed = PhpType::parse(type_str);
    let entries = unwrap_array_shape(&parsed)?;
    Some(shape_entries_to_array_entries(entries))
}

/// Look up the value type for a specific key in an array shape type string.
///
/// Given a type like `"array{name: string, user: User}"` and key `"user"`,
/// returns `Some("User")`.
///
/// Returns `None` if the type is not an array shape or the key is not found.
pub fn extract_array_shape_value_type(type_str: &str, key: &str) -> Option<String> {
    let entries = parse_array_shape(type_str)?;
    entries
        .into_iter()
        .find(|e| e.key == key)
        .map(|e| e.value_type)
}

/// Parse a PHPStan object shape type string into its constituent entries.
///
/// Object shapes describe an anonymous object with typed properties:
///
/// # Examples
///
/// - `"object{foo: int, bar: string}"` → two entries
/// - `"object{foo: int, bar?: string}"` → "bar" is optional
/// - `"object{'foo': int, \"bar\": string}"` → quoted property names
/// - `"object{foo: int, bar: string}&\stdClass"` → intersection ignored here
///
/// The returned entries reuse [`ArrayShapeEntry`] since the structure is
/// identical (key name, value type, optional flag).
///
/// Returns `None` if the type is not an object shape.
pub fn parse_object_shape(type_str: &str) -> Option<Vec<ArrayShapeEntry>> {
    let parsed = PhpType::parse(type_str);
    let entries = unwrap_object_shape(&parsed)?;
    Some(shape_entries_to_array_entries(entries))
}

/// Return `true` if `type_str` is an object shape type (e.g. `object{name: string}`).
pub fn is_object_shape(type_str: &str) -> bool {
    PhpType::parse(type_str).is_object_shape()
}

/// Look up the value type for a specific property in an object shape.
///
/// Given a type like `"object{name: string, user: User}"` and key `"user"`,
/// returns `Some("User")`.
///
/// Returns `None` if the type is not an object shape or the property
/// is not found.
pub fn extract_object_shape_property_type(type_str: &str, prop: &str) -> Option<String> {
    let entries = parse_object_shape(type_str)?;
    entries
        .into_iter()
        .find(|e| e.key == prop)
        .map(|e| e.value_type)
}
