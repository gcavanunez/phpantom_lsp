//! Type cleaning and classification utilities for PHPDoc types.
//!
//! This module was split into focused submodules for navigability:
//!
//! - [`super::type_strings`]: Foundational type string manipulation (constants,
//!   splitting, cleaning, stripping, scalar checks, self/static replacement)
//! - [`super::generics`]: Generic argument parsing and iterable element/key extraction
//! - [`super::shapes`]: Array shape and object shape parsing
//! - [`super::callable_types`]: Callable/Closure return type and parameter extraction,
//!   Generator TSend/TValue extraction
//!
//! All public and crate-visible items are re-exported here so that existing
//! `use crate::docblock::types::*` and `use super::types::*` call sites
//! continue to work without modification.

// ─── Re-exports: type_strings ───────────────────────────────────────────────

pub(crate) use super::type_strings::PHPDOC_TYPE_KEYWORDS;
#[cfg(test)]
pub(crate) use super::type_strings::SCALAR_TYPES;
pub use super::type_strings::split_intersection_depth0;
pub use super::type_strings::{base_class_name, clean_type, replace_self_in_type};
pub(crate) use super::type_strings::{
    is_scalar, split_generic_args, split_type_token, split_union_depth0, strip_generics,
    strip_nullable,
};

// ─── Re-exports: generics ───────────────────────────────────────────────────

pub use super::generics::{
    extract_generic_key_type, extract_generic_value_type, extract_iterable_element_type,
};
pub(crate) use super::generics::{find_matching_close, parse_generic_args};

// ─── Re-exports: shapes ─────────────────────────────────────────────────────

pub use super::shapes::{
    extract_array_shape_value_type, extract_object_shape_property_type, is_object_shape,
    parse_array_shape, parse_object_shape,
};

// ─── Re-exports: callable_types ─────────────────────────────────────────────

pub use super::callable_types::{
    extract_callable_param_types, extract_callable_return_type, extract_generator_send_type,
    extract_generator_value_type_raw,
};

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
