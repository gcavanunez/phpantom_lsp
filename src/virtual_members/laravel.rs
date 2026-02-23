//! Laravel Eloquent Model virtual member provider.
//!
//! Synthesizes virtual members for classes that extend
//! `Illuminate\Database\Eloquent\Model`.  This is the highest-priority
//! virtual member provider: its contributions beat `@method` /
//! `@property` tags (PHPDocProvider) and `@mixin` members
//! (MixinProvider).
//!
//! Currently implements:
//!
//! - **Relationship properties.** Methods returning a known Eloquent
//!   relationship type (e.g. `HasOne`, `HasMany`, `BelongsTo`) produce
//!   a virtual property with the same name.  The property type is
//!   inferred from the relationship's generic parameters (Larastan-style
//!   `@return HasMany<Post, $this>` annotations) or, as a fallback,
//!   from the first `::class` argument in the method body text.

use crate::docblock::types::parse_generic_args;
use crate::types::{ClassInfo, MAX_INHERITANCE_DEPTH, PropertyInfo, Visibility};

use super::{VirtualMemberProvider, VirtualMembers};

/// The fully-qualified name of the Eloquent base model.
const ELOQUENT_MODEL_FQN: &str = "Illuminate\\Database\\Eloquent\\Model";

/// Known Eloquent relationship class short names that yield a single
/// (nullable) related model instance when accessed as a property.
const SINGULAR_RELATIONSHIPS: &[&str] = &["HasOne", "MorphOne", "BelongsTo"];

/// Known Eloquent relationship class short names that yield a
/// `Collection<TRelated>` when accessed as a property.
const COLLECTION_RELATIONSHIPS: &[&str] = &[
    "HasMany",
    "MorphMany",
    "BelongsToMany",
    "HasManyThrough",
    "MorphToMany",
];

/// The `MorphTo` relationship resolves to the generic `Model` base class
/// because the concrete related type is determined at runtime.
const MORPH_TO: &str = "MorphTo";

/// Virtual member provider for Laravel Eloquent models.
///
/// When a class extends `Illuminate\Database\Eloquent\Model` (directly
/// or through an intermediate parent), this provider scans its methods
/// for Eloquent relationship return types and synthesizes virtual
/// properties for each one.
///
/// For example, a method `posts()` returning `HasMany<Post, $this>`
/// produces a virtual property `$posts` with type
/// `\Illuminate\Database\Eloquent\Collection<Post>`.
pub struct LaravelModelProvider;

/// Determine whether `class_name` is the Eloquent Model base class.
///
/// Checks against the FQN with and without a leading backslash, and
/// also against the short name `Model` (which may appear in stubs or
/// in same-file test setups).
fn is_eloquent_model(class_name: &str) -> bool {
    let stripped = class_name.strip_prefix('\\').unwrap_or(class_name);
    stripped == ELOQUENT_MODEL_FQN
}

/// Walk the parent chain of `class` looking for
/// `Illuminate\Database\Eloquent\Model`.
///
/// Returns `true` if the class itself is `Model` or any ancestor is.
fn extends_eloquent_model(
    class: &ClassInfo,
    class_loader: &dyn Fn(&str) -> Option<ClassInfo>,
) -> bool {
    if is_eloquent_model(&class.name) {
        return true;
    }

    let mut current = class.clone();
    let mut depth = 0u32;
    while let Some(ref parent_name) = current.parent_class {
        depth += 1;
        if depth > MAX_INHERITANCE_DEPTH {
            break;
        }
        if is_eloquent_model(parent_name) {
            return true;
        }
        match class_loader(parent_name) {
            Some(parent) => {
                if is_eloquent_model(&parent.name) {
                    return true;
                }
                current = parent;
            }
            None => break,
        }
    }

    false
}

/// The category of a relationship return type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelationshipKind {
    /// HasOne, MorphOne, BelongsTo — singular nullable model.
    Singular,
    /// HasMany, MorphMany, BelongsToMany, HasManyThrough, MorphToMany — Collection.
    Collection,
    /// MorphTo — generic Model.
    MorphTo,
}

/// Try to classify a return type string as a known Eloquent relationship.
///
/// Accepts both short names (`HasMany`) and fully-qualified names
/// (`\Illuminate\Database\Eloquent\Relations\HasMany`).  Generic
/// parameters are stripped before matching.
fn classify_relationship(return_type: &str) -> Option<RelationshipKind> {
    let (base, _) = parse_generic_args(return_type);
    let short_name = extract_short_name(base);

    if SINGULAR_RELATIONSHIPS.contains(&short_name) {
        return Some(RelationshipKind::Singular);
    }
    if COLLECTION_RELATIONSHIPS.contains(&short_name) {
        return Some(RelationshipKind::Collection);
    }
    if short_name == MORPH_TO {
        return Some(RelationshipKind::MorphTo);
    }

    None
}

/// Extract the short (unqualified) class name from a potentially
/// fully-qualified name.
///
/// `"\\Illuminate\\Database\\Eloquent\\Relations\\HasMany"` → `"HasMany"`
/// `"HasMany"` → `"HasMany"`
fn extract_short_name(fqn: &str) -> &str {
    fqn.rsplit('\\').next().unwrap_or(fqn)
}

/// Extract the `TRelated` type from a relationship return type's
/// generic parameters.
///
/// Given `"HasMany<Post, $this>"`, returns `Some("Post")`.
/// Given `"HasOne<\\App\\Models\\Post, $this>"`, returns
/// `Some("\\App\\Models\\Post")`.
///
/// Returns `None` if no generic parameters are present.
fn extract_related_type(return_type: &str) -> Option<String> {
    let (_, args) = parse_generic_args(return_type);
    let first = args.first()?;
    let trimmed = first.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Build the virtual property type for a relationship.
///
/// - Singular relationships → the related type as-is (nullable).
/// - Collection relationships → `\Illuminate\Database\Eloquent\Collection<TRelated>`.
/// - MorphTo → `\Illuminate\Database\Eloquent\Model`.
fn build_property_type(kind: RelationshipKind, related_type: Option<&str>) -> Option<String> {
    match kind {
        RelationshipKind::Singular => related_type.map(|t| t.to_string()),
        RelationshipKind::Collection => {
            let inner = related_type.unwrap_or("\\Illuminate\\Database\\Eloquent\\Model");
            Some(format!(
                "\\Illuminate\\Database\\Eloquent\\Collection<{inner}>"
            ))
        }
        RelationshipKind::MorphTo => Some("\\Illuminate\\Database\\Eloquent\\Model".to_string()),
    }
}

impl VirtualMemberProvider for LaravelModelProvider {
    /// Returns `true` if the class extends `Illuminate\Database\Eloquent\Model`.
    fn applies_to(
        &self,
        class: &ClassInfo,
        class_loader: &dyn Fn(&str) -> Option<ClassInfo>,
    ) -> bool {
        extends_eloquent_model(class, class_loader)
    }

    /// Scan the class's methods for Eloquent relationship return types
    /// and synthesize a virtual property for each one.
    fn provide(
        &self,
        class: &ClassInfo,
        _class_loader: &dyn Fn(&str) -> Option<ClassInfo>,
    ) -> VirtualMembers {
        let mut properties = Vec::new();

        for method in &class.methods {
            let return_type = match method.return_type.as_deref() {
                Some(rt) => rt,
                None => continue,
            };

            let kind = match classify_relationship(return_type) {
                Some(k) => k,
                None => continue,
            };

            let related_type = extract_related_type(return_type);
            let type_hint = build_property_type(kind, related_type.as_deref());

            if type_hint.is_some() {
                properties.push(PropertyInfo {
                    name: method.name.clone(),
                    type_hint,
                    is_static: false,
                    visibility: Visibility::Public,
                    is_deprecated: false,
                });
            }
        }

        VirtualMembers {
            methods: Vec::new(),
            properties,
            constants: Vec::new(),
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClassLikeKind, MethodInfo};
    use std::collections::HashMap;

    /// Helper: create a minimal `ClassInfo` with the given name.
    fn make_class(name: &str) -> ClassInfo {
        ClassInfo {
            kind: ClassLikeKind::Class,
            name: name.to_string(),
            methods: Vec::new(),
            properties: Vec::new(),
            constants: Vec::new(),
            start_offset: 0,
            end_offset: 0,
            parent_class: None,
            interfaces: Vec::new(),
            used_traits: Vec::new(),
            mixins: Vec::new(),
            is_final: false,
            is_abstract: false,
            is_deprecated: false,
            template_params: Vec::new(),
            template_param_bounds: HashMap::new(),
            extends_generics: Vec::new(),
            implements_generics: Vec::new(),
            use_generics: Vec::new(),
            type_aliases: HashMap::new(),
            trait_precedences: Vec::new(),
            trait_aliases: Vec::new(),
            class_docblock: None,
        }
    }

    /// Helper: create a `MethodInfo` with a given return type.
    fn make_method(name: &str, return_type: Option<&str>) -> MethodInfo {
        MethodInfo {
            name: name.to_string(),
            parameters: Vec::new(),
            return_type: return_type.map(|s| s.to_string()),
            is_static: false,
            visibility: Visibility::Public,
            conditional_return: None,
            is_deprecated: false,
            template_params: Vec::new(),
            template_bindings: Vec::new(),
        }
    }

    fn no_loader(_name: &str) -> Option<ClassInfo> {
        None
    }

    // ── is_eloquent_model ───────────────────────────────────────────────

    #[test]
    fn recognises_fqn() {
        assert!(is_eloquent_model("Illuminate\\Database\\Eloquent\\Model"));
    }

    #[test]
    fn recognises_fqn_with_leading_backslash() {
        assert!(is_eloquent_model("\\Illuminate\\Database\\Eloquent\\Model"));
    }

    #[test]
    fn rejects_unrelated_class() {
        assert!(!is_eloquent_model("App\\Models\\User"));
    }

    // ── extends_eloquent_model ──────────────────────────────────────────

    #[test]
    fn direct_child_of_model() {
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());

        let model = make_class("Illuminate\\Database\\Eloquent\\Model");
        let loader = |name: &str| -> Option<ClassInfo> {
            if name == "Illuminate\\Database\\Eloquent\\Model" {
                Some(model.clone())
            } else {
                None
            }
        };

        assert!(extends_eloquent_model(&user, &loader));
    }

    #[test]
    fn indirect_child_of_model() {
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("App\\Models\\BaseModel".to_string());

        let mut base_model = make_class("App\\Models\\BaseModel");
        base_model.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());

        let model = make_class("Illuminate\\Database\\Eloquent\\Model");

        let loader = |name: &str| -> Option<ClassInfo> {
            match name {
                "App\\Models\\BaseModel" => Some(base_model.clone()),
                "Illuminate\\Database\\Eloquent\\Model" => Some(model.clone()),
                _ => None,
            }
        };

        assert!(extends_eloquent_model(&user, &loader));
    }

    #[test]
    fn not_a_model() {
        let service = make_class("App\\Services\\UserService");
        assert!(!extends_eloquent_model(&service, &no_loader));
    }

    // ── classify_relationship ───────────────────────────────────────────

    #[test]
    fn classify_has_one() {
        assert_eq!(
            classify_relationship("HasOne<Profile, $this>"),
            Some(RelationshipKind::Singular)
        );
    }

    #[test]
    fn classify_has_many() {
        assert_eq!(
            classify_relationship("HasMany<Post, $this>"),
            Some(RelationshipKind::Collection)
        );
    }

    #[test]
    fn classify_belongs_to() {
        assert_eq!(
            classify_relationship("BelongsTo<User, $this>"),
            Some(RelationshipKind::Singular)
        );
    }

    #[test]
    fn classify_belongs_to_many() {
        assert_eq!(
            classify_relationship("BelongsToMany<Role, $this>"),
            Some(RelationshipKind::Collection)
        );
    }

    #[test]
    fn classify_morph_one() {
        assert_eq!(
            classify_relationship("MorphOne<Image, $this>"),
            Some(RelationshipKind::Singular)
        );
    }

    #[test]
    fn classify_morph_many() {
        assert_eq!(
            classify_relationship("MorphMany<Comment, $this>"),
            Some(RelationshipKind::Collection)
        );
    }

    #[test]
    fn classify_morph_to() {
        assert_eq!(
            classify_relationship("MorphTo"),
            Some(RelationshipKind::MorphTo)
        );
    }

    #[test]
    fn classify_morph_to_many() {
        assert_eq!(
            classify_relationship("MorphToMany<Tag, $this>"),
            Some(RelationshipKind::Collection)
        );
    }

    #[test]
    fn classify_has_many_through() {
        assert_eq!(
            classify_relationship("HasManyThrough<Post, Country>"),
            Some(RelationshipKind::Collection)
        );
    }

    #[test]
    fn classify_fqn_relationship() {
        assert_eq!(
            classify_relationship(
                "\\Illuminate\\Database\\Eloquent\\Relations\\HasMany<Post, $this>"
            ),
            Some(RelationshipKind::Collection)
        );
    }

    #[test]
    fn classify_non_relationship() {
        assert_eq!(classify_relationship("string"), None);
        assert_eq!(classify_relationship("Collection<User>"), None);
    }

    #[test]
    fn classify_bare_name_without_generics() {
        assert_eq!(
            classify_relationship("HasMany"),
            Some(RelationshipKind::Collection)
        );
    }

    // ── extract_related_type ────────────────────────────────────────────

    #[test]
    fn extracts_first_generic_arg() {
        assert_eq!(
            extract_related_type("HasMany<Post, $this>"),
            Some("Post".to_string())
        );
    }

    #[test]
    fn extracts_fqn_related_type() {
        assert_eq!(
            extract_related_type("HasOne<\\App\\Models\\Profile, $this>"),
            Some("\\App\\Models\\Profile".to_string())
        );
    }

    #[test]
    fn returns_none_without_generics() {
        assert_eq!(extract_related_type("HasMany"), None);
    }

    // ── build_property_type ─────────────────────────────────────────────

    #[test]
    fn singular_with_related() {
        assert_eq!(
            build_property_type(RelationshipKind::Singular, Some("Profile")),
            Some("Profile".to_string())
        );
    }

    #[test]
    fn singular_without_related() {
        assert_eq!(build_property_type(RelationshipKind::Singular, None), None);
    }

    #[test]
    fn collection_with_related() {
        assert_eq!(
            build_property_type(RelationshipKind::Collection, Some("Post")),
            Some("\\Illuminate\\Database\\Eloquent\\Collection<Post>".to_string())
        );
    }

    #[test]
    fn collection_without_related_uses_model() {
        assert_eq!(
            build_property_type(RelationshipKind::Collection, None),
            Some(
                "\\Illuminate\\Database\\Eloquent\\Collection<\\Illuminate\\Database\\Eloquent\\Model>"
                    .to_string()
            )
        );
    }

    #[test]
    fn morph_to_always_returns_model() {
        assert_eq!(
            build_property_type(RelationshipKind::MorphTo, None),
            Some("\\Illuminate\\Database\\Eloquent\\Model".to_string())
        );
    }

    // ── applies_to ──────────────────────────────────────────────────────

    #[test]
    fn applies_to_model_subclass() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());

        let model = make_class("Illuminate\\Database\\Eloquent\\Model");
        let loader = |name: &str| -> Option<ClassInfo> {
            if name == "Illuminate\\Database\\Eloquent\\Model" {
                Some(model.clone())
            } else {
                None
            }
        };

        assert!(provider.applies_to(&user, &loader));
    }

    #[test]
    fn does_not_apply_to_non_model() {
        let provider = LaravelModelProvider;
        let service = make_class("App\\Services\\UserService");
        assert!(!provider.applies_to(&service, &no_loader));
    }

    // ── provide: relationship properties ────────────────────────────────

    #[test]
    fn synthesizes_has_many_property() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods
            .push(make_method("posts", Some("HasMany<Post, $this>")));

        let result = provider.provide(&user, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(result.properties[0].name, "posts");
        assert_eq!(
            result.properties[0].type_hint.as_deref(),
            Some("\\Illuminate\\Database\\Eloquent\\Collection<Post>")
        );
        assert_eq!(result.properties[0].visibility, Visibility::Public);
        assert!(!result.properties[0].is_static);
    }

    #[test]
    fn synthesizes_has_one_property() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods
            .push(make_method("profile", Some("HasOne<Profile, $this>")));

        let result = provider.provide(&user, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(result.properties[0].name, "profile");
        assert_eq!(result.properties[0].type_hint.as_deref(), Some("Profile"));
    }

    #[test]
    fn synthesizes_belongs_to_property() {
        let provider = LaravelModelProvider;
        let mut post = make_class("App\\Models\\Post");
        post.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        post.methods
            .push(make_method("author", Some("BelongsTo<User, $this>")));

        let result = provider.provide(&post, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(result.properties[0].name, "author");
        assert_eq!(result.properties[0].type_hint.as_deref(), Some("User"));
    }

    #[test]
    fn synthesizes_morph_to_property() {
        let provider = LaravelModelProvider;
        let mut comment = make_class("App\\Models\\Comment");
        comment.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        comment
            .methods
            .push(make_method("commentable", Some("MorphTo")));

        let result = provider.provide(&comment, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(result.properties[0].name, "commentable");
        assert_eq!(
            result.properties[0].type_hint.as_deref(),
            Some("\\Illuminate\\Database\\Eloquent\\Model")
        );
    }

    #[test]
    fn synthesizes_belongs_to_many_property() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods
            .push(make_method("roles", Some("BelongsToMany<Role, $this>")));

        let result = provider.provide(&user, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(result.properties[0].name, "roles");
        assert_eq!(
            result.properties[0].type_hint.as_deref(),
            Some("\\Illuminate\\Database\\Eloquent\\Collection<Role>")
        );
    }

    #[test]
    fn synthesizes_multiple_relationship_properties() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods
            .push(make_method("posts", Some("HasMany<Post, $this>")));
        user.methods
            .push(make_method("profile", Some("HasOne<Profile, $this>")));
        user.methods
            .push(make_method("roles", Some("BelongsToMany<Role, $this>")));

        let result = provider.provide(&user, &no_loader);
        assert_eq!(result.properties.len(), 3);

        let names: Vec<&str> = result.properties.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"posts"));
        assert!(names.contains(&"profile"));
        assert!(names.contains(&"roles"));
    }

    #[test]
    fn skips_non_relationship_methods() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods
            .push(make_method("getFullName", Some("string")));
        user.methods.push(make_method("save", Some("bool")));
        user.methods.push(make_method("toArray", Some("array")));

        let result = provider.provide(&user, &no_loader);
        assert!(result.properties.is_empty());
    }

    #[test]
    fn skips_methods_without_return_type() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods.push(make_method("posts", None));

        let result = provider.provide(&user, &no_loader);
        assert!(result.properties.is_empty());
    }

    #[test]
    fn handles_fqn_relationship_return_types() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods.push(make_method(
            "posts",
            Some("\\Illuminate\\Database\\Eloquent\\Relations\\HasMany<Post, $this>"),
        ));

        let result = provider.provide(&user, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(result.properties[0].name, "posts");
        assert_eq!(
            result.properties[0].type_hint.as_deref(),
            Some("\\Illuminate\\Database\\Eloquent\\Collection<Post>")
        );
    }

    #[test]
    fn relationship_without_generics_and_singular_produces_nothing() {
        // A singular relationship without generics has no TRelated,
        // so we cannot determine the property type.
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods.push(make_method("profile", Some("HasOne")));

        let result = provider.provide(&user, &no_loader);
        assert!(
            result.properties.is_empty(),
            "Singular relationship without generics should not produce a property"
        );
    }

    #[test]
    fn collection_relationship_without_generics_uses_model_fallback() {
        // A collection relationship without generics defaults to
        // Collection<Model>.
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods.push(make_method("posts", Some("HasMany")));

        let result = provider.provide(&user, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(
            result.properties[0].type_hint.as_deref(),
            Some(
                "\\Illuminate\\Database\\Eloquent\\Collection<\\Illuminate\\Database\\Eloquent\\Model>"
            )
        );
    }

    #[test]
    fn no_virtual_methods_or_constants() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods
            .push(make_method("posts", Some("HasMany<Post, $this>")));

        let result = provider.provide(&user, &no_loader);
        assert!(result.methods.is_empty());
        assert!(result.constants.is_empty());
    }

    #[test]
    fn provides_fqn_related_type_in_collection() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods.push(make_method(
            "posts",
            Some("HasMany<\\App\\Models\\Post, $this>"),
        ));

        let result = provider.provide(&user, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(
            result.properties[0].type_hint.as_deref(),
            Some("\\Illuminate\\Database\\Eloquent\\Collection<\\App\\Models\\Post>")
        );
    }

    #[test]
    fn provides_fqn_related_type_singular() {
        let provider = LaravelModelProvider;
        let mut user = make_class("App\\Models\\User");
        user.parent_class = Some("Illuminate\\Database\\Eloquent\\Model".to_string());
        user.methods.push(make_method(
            "profile",
            Some("HasOne<\\App\\Models\\Profile, $this>"),
        ));

        let result = provider.provide(&user, &no_loader);
        assert_eq!(result.properties.len(), 1);
        assert_eq!(
            result.properties[0].type_hint.as_deref(),
            Some("\\App\\Models\\Profile")
        );
    }

    // ── extract_short_name ──────────────────────────────────────────────

    #[test]
    fn short_name_from_fqn() {
        assert_eq!(
            extract_short_name("\\Illuminate\\Database\\Eloquent\\Relations\\HasMany"),
            "HasMany"
        );
    }

    #[test]
    fn short_name_already_short() {
        assert_eq!(extract_short_name("HasMany"), "HasMany");
    }

    #[test]
    fn short_name_no_backslash_prefix() {
        assert_eq!(
            extract_short_name("Illuminate\\Database\\Eloquent\\Relations\\HasOne"),
            "HasOne"
        );
    }
}
