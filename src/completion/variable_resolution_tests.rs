use super::enrich_builder_type_in_scope;
use crate::test_fixtures::make_class;

use crate::types::ClassInfo;

fn make_model(name: &str) -> ClassInfo {
    let mut class = make_class(name);
    class.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
    class
}

fn model_loader(name: &str) -> Option<ClassInfo> {
    if name == "Illuminate\\Database\\Eloquent\\Model" {
        Some(make_class("Illuminate\\Database\\Eloquent\\Model"))
    } else if name == "App\\Models\\User" {
        Some(make_model("App\\Models\\User"))
    } else {
        None
    }
}

#[test]
fn enrich_scope_method_with_builder_type() {
    let model = make_model("App\\Models\\User");
    let result =
        enrich_builder_type_in_scope("Builder", "scopeActive", false, &model, &model_loader);
    assert_eq!(result, Some("Builder<App\\Models\\User>".to_string()));
}

#[test]
fn enrich_scope_method_with_fqn_builder() {
    let model = make_model("App\\Models\\User");
    let result = enrich_builder_type_in_scope(
        "Illuminate\\Database\\Eloquent\\Builder",
        "scopeActive",
        false,
        &model,
        &model_loader,
    );
    assert_eq!(
        result,
        Some("Illuminate\\Database\\Eloquent\\Builder<App\\Models\\User>".to_string())
    );
}

#[test]
fn enrich_skips_non_scope_method() {
    let model = make_model("App\\Models\\User");
    let result = enrich_builder_type_in_scope("Builder", "getName", false, &model, &model_loader);
    assert_eq!(result, None);
}

#[test]
fn enrich_skips_bare_scope_name() {
    let model = make_model("App\\Models\\User");
    let result = enrich_builder_type_in_scope("Builder", "scope", false, &model, &model_loader);
    assert_eq!(result, None);
}

#[test]
fn enrich_skips_non_model_class() {
    let plain = make_class("App\\Services\\SomeService");
    let result =
        enrich_builder_type_in_scope("Builder", "scopeActive", false, &plain, &model_loader);
    assert_eq!(result, None);
}

#[test]
fn enrich_skips_non_builder_type() {
    let model = make_model("App\\Models\\User");
    let result =
        enrich_builder_type_in_scope("Collection", "scopeActive", false, &model, &model_loader);
    assert_eq!(result, None);
}

#[test]
fn enrich_skips_builder_with_existing_generics() {
    let model = make_model("App\\Models\\User");
    let result =
        enrich_builder_type_in_scope("Builder<User>", "scopeActive", false, &model, &model_loader);
    assert_eq!(result, None);
}

#[test]
fn enrich_scope_multi_word_method_name() {
    let model = make_model("App\\Models\\User");
    let result =
        enrich_builder_type_in_scope("Builder", "scopeByAuthor", false, &model, &model_loader);
    assert_eq!(result, Some("Builder<App\\Models\\User>".to_string()));
}

#[test]
fn enrich_scope_with_leading_backslash_builder() {
    let model = make_model("App\\Models\\User");
    let result = enrich_builder_type_in_scope(
        "\\Illuminate\\Database\\Eloquent\\Builder",
        "scopeActive",
        false,
        &model,
        &model_loader,
    );
    assert_eq!(
        result,
        Some("\\Illuminate\\Database\\Eloquent\\Builder<App\\Models\\User>".to_string())
    );
}

// ── #[Scope] attribute tests ────────────────────────────────────────

#[test]
fn enrich_scope_attribute_method_with_builder_type() {
    let model = make_model("App\\Models\\User");
    let result = enrich_builder_type_in_scope("Builder", "active", true, &model, &model_loader);
    assert_eq!(result, Some("Builder<App\\Models\\User>".to_string()));
}

#[test]
fn enrich_scope_attribute_with_fqn_builder() {
    let model = make_model("App\\Models\\User");
    let result = enrich_builder_type_in_scope(
        "Illuminate\\Database\\Eloquent\\Builder",
        "active",
        true,
        &model,
        &model_loader,
    );
    assert_eq!(
        result,
        Some("Illuminate\\Database\\Eloquent\\Builder<App\\Models\\User>".to_string())
    );
}

#[test]
fn enrich_scope_attribute_skips_non_model_class() {
    let plain = make_class("App\\Services\\SomeService");
    let result = enrich_builder_type_in_scope("Builder", "active", true, &plain, &model_loader);
    assert_eq!(result, None);
}

#[test]
fn enrich_scope_attribute_skips_non_builder_type() {
    let model = make_model("App\\Models\\User");
    let result = enrich_builder_type_in_scope("Collection", "active", true, &model, &model_loader);
    assert_eq!(result, None);
}

#[test]
fn enrich_no_scope_attribute_and_no_convention_skips() {
    let model = make_model("App\\Models\\User");
    // Not a scopeX name and no attribute → should skip.
    let result = enrich_builder_type_in_scope("Builder", "active", false, &model, &model_loader);
    assert_eq!(result, None);
}
