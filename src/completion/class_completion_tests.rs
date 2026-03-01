use super::*;
use crate::types::ClassLikeKind;

// ── detect_stub_class_kind ──────────────────────────────────────

#[test]
fn test_detect_class_in_single_class_file() {
    let source = "<?php\nclass DateTime {\n}\n";
    let result = detect_stub_class_kind("DateTime", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Class, false, false)),
        "should detect a plain class"
    );
}

#[test]
fn test_detect_interface_in_single_file() {
    let source = "<?php\ninterface JsonSerializable\n{\n}\n";
    let result = detect_stub_class_kind("JsonSerializable", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Interface, false, false)),
        "should detect an interface"
    );
}

#[test]
fn test_detect_abstract_class() {
    let source = "<?php\nabstract class SplHeap implements Iterator, Countable\n{\n}\n";
    let result = detect_stub_class_kind("SplHeap", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Class, true, false)),
        "should detect an abstract class"
    );
}

#[test]
fn test_detect_final_class() {
    let source = "<?php\nfinal class Closure {\n}\n";
    let result = detect_stub_class_kind("Closure", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Class, false, true)),
        "should detect a final class"
    );
}

#[test]
fn test_detect_readonly_class() {
    let source = "<?php\nreadonly class Value {\n}\n";
    let result = detect_stub_class_kind("Value", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Class, false, false)),
        "readonly class is neither abstract nor final"
    );
}

#[test]
fn test_detect_final_readonly_class() {
    let source = "<?php\nfinal readonly class Immutable {\n}\n";
    let result = detect_stub_class_kind("Immutable", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Class, false, true)),
        "should detect final through readonly"
    );
}

#[test]
fn test_detect_abstract_readonly_class() {
    let source = "<?php\nabstract readonly class Base {\n}\n";
    let result = detect_stub_class_kind("Base", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Class, true, false)),
        "should detect abstract through readonly"
    );
}

#[test]
fn test_detect_trait() {
    let source = "<?php\ntrait Stringable {\n}\n";
    let result = detect_stub_class_kind("Stringable", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Trait, false, false)),
        "should detect a trait"
    );
}

#[test]
fn test_detect_enum() {
    let source = "<?php\nenum Suit {\n}\n";
    let result = detect_stub_class_kind("Suit", source);
    assert_eq!(
        result,
        Some((ClassLikeKind::Enum, false, false)),
        "should detect an enum"
    );
}

#[test]
fn test_detect_class_in_multi_class_file() {
    // Simulates SPL_c1.php which has many classes and a few interfaces.
    let source = concat!(
        "<?php\n",
        "class SplFileInfo implements Stringable\n{\n}\n",
        "class DirectoryIterator extends SplFileInfo implements SeekableIterator\n{\n}\n",
        "class FilesystemIterator extends DirectoryIterator\n{\n}\n",
        "abstract class SplHeap implements Iterator, Countable\n{\n}\n",
        "interface SplObserver\n{\n}\n",
        "interface SplSubject\n{\n}\n",
        "class SplObjectStorage implements Countable\n{\n}\n",
    );

    assert_eq!(
        detect_stub_class_kind("DirectoryIterator", source),
        Some((ClassLikeKind::Class, false, false)),
        "should find DirectoryIterator as a class in a multi-class file"
    );
    assert_eq!(
        detect_stub_class_kind("SplHeap", source),
        Some((ClassLikeKind::Class, true, false)),
        "should find SplHeap as an abstract class"
    );
    assert_eq!(
        detect_stub_class_kind("SplObserver", source),
        Some((ClassLikeKind::Interface, false, false)),
        "should find SplObserver as an interface"
    );
    assert_eq!(
        detect_stub_class_kind("SplObjectStorage", source),
        Some((ClassLikeKind::Class, false, false)),
        "should find SplObjectStorage as a class"
    );
}

#[test]
fn test_detect_does_not_match_substring() {
    // "Iterator" appears as a substring in "DirectoryIterator" and
    // "FilesystemIterator".  The word boundary check must prevent a
    // false match.
    let source = concat!(
        "<?php\n",
        "interface Iterator\n{\n}\n",
        "class DirectoryIterator extends SplFileInfo\n{\n}\n",
    );

    assert_eq!(
        detect_stub_class_kind("Iterator", source),
        Some((ClassLikeKind::Interface, false, false)),
        "should match the standalone 'Iterator' interface, not the substring in DirectoryIterator"
    );
}

#[test]
fn test_detect_does_not_match_superstring() {
    // Searching for "Directory" should NOT match "DirectoryIterator".
    let source = "<?php\nclass DirectoryIterator extends SplFileInfo\n{\n}\n";
    assert_eq!(
        detect_stub_class_kind("Directory", source),
        None,
        "should not match 'Directory' inside 'DirectoryIterator'"
    );
}

#[test]
fn test_detect_skips_name_in_comments() {
    // The class name appears in a docblock comment, not a declaration.
    let source = concat!(
        "<?php\n",
        "/**\n",
        " * @see DirectoryIterator\n",
        " */\n",
        "class DirectoryIterator extends SplFileInfo\n{\n}\n",
    );
    assert_eq!(
        detect_stub_class_kind("DirectoryIterator", source),
        Some((ClassLikeKind::Class, false, false)),
        "should skip the comment mention and find the actual class declaration"
    );
}

#[test]
fn test_detect_skips_extends_mention() {
    // "SplFileInfo" appears after `extends`, not as a declaration keyword.
    let source = concat!(
        "<?php\n",
        "class DirectoryIterator extends SplFileInfo\n{\n}\n",
    );
    assert_eq!(
        detect_stub_class_kind("SplFileInfo", source),
        None,
        "should not match SplFileInfo in 'extends SplFileInfo' (no declaration keyword before it)"
    );
}

#[test]
fn test_detect_with_fqn_key() {
    // The stub_index key might be a FQN like "Ds\\Set".
    // detect_stub_class_kind should extract the short name "Set".
    let source = concat!(
        "<?php\n",
        "namespace Ds;\n",
        "class Set implements Collection\n{\n}\n",
    );
    assert_eq!(
        detect_stub_class_kind("Ds\\Set", source),
        Some((ClassLikeKind::Class, false, false)),
        "should handle FQN keys by extracting the short name"
    );
}

#[test]
fn test_detect_not_found() {
    let source = "<?php\nclass Foo {\n}\n";
    assert_eq!(
        detect_stub_class_kind("Bar", source),
        None,
        "should return None when the class is not in the source"
    );
}

#[test]
fn test_detect_class_with_extends_and_implements() {
    let source = "<?php\nclass SplFixedArray implements Iterator, ArrayAccess, Countable, IteratorAggregate, JsonSerializable\n{\n}\n";
    assert_eq!(
        detect_stub_class_kind("SplFixedArray", source),
        Some((ClassLikeKind::Class, false, false)),
        "should detect a class with multiple implements"
    );
}

// ── ClassNameContext::matches_kind_flags ─────────────────────────

#[test]
fn test_extends_class_rejects_interface() {
    assert!(
        !ClassNameContext::ExtendsClass.matches_kind_flags(ClassLikeKind::Interface, false, false),
        "ExtendsClass should reject interfaces"
    );
}

#[test]
fn test_extends_class_rejects_final() {
    assert!(
        !ClassNameContext::ExtendsClass.matches_kind_flags(ClassLikeKind::Class, false, true),
        "ExtendsClass should reject final classes"
    );
}

#[test]
fn test_extends_class_accepts_abstract() {
    assert!(
        ClassNameContext::ExtendsClass.matches_kind_flags(ClassLikeKind::Class, true, false),
        "ExtendsClass should accept abstract classes"
    );
}

#[test]
fn test_implements_accepts_interface() {
    assert!(
        ClassNameContext::Implements.matches_kind_flags(ClassLikeKind::Interface, false, false),
        "Implements should accept interfaces"
    );
}

#[test]
fn test_implements_rejects_class() {
    assert!(
        !ClassNameContext::Implements.matches_kind_flags(ClassLikeKind::Class, false, false),
        "Implements should reject classes"
    );
}

#[test]
fn test_trait_use_accepts_trait() {
    assert!(
        ClassNameContext::TraitUse.matches_kind_flags(ClassLikeKind::Trait, false, false),
        "TraitUse should accept traits"
    );
}

#[test]
fn test_trait_use_rejects_class() {
    assert!(
        !ClassNameContext::TraitUse.matches_kind_flags(ClassLikeKind::Class, false, false),
        "TraitUse should reject classes"
    );
}

#[test]
fn test_instanceof_rejects_trait() {
    assert!(
        !ClassNameContext::Instanceof.matches_kind_flags(ClassLikeKind::Trait, false, false),
        "Instanceof should reject traits"
    );
}

#[test]
fn test_instanceof_accepts_enum() {
    assert!(
        ClassNameContext::Instanceof.matches_kind_flags(ClassLikeKind::Enum, false, false),
        "Instanceof should accept enums"
    );
}

#[test]
fn test_new_rejects_abstract() {
    assert!(
        !ClassNameContext::New.matches_kind_flags(ClassLikeKind::Class, true, false),
        "New should reject abstract classes"
    );
}

#[test]
fn test_new_rejects_interface() {
    assert!(
        !ClassNameContext::New.matches_kind_flags(ClassLikeKind::Interface, false, false),
        "New should reject interfaces"
    );
}

// ── UseImport / UseFunction / UseConst detection ────────────────

#[test]
fn test_detect_use_import_context() {
    let content = "<?php\nuse App";
    let pos = Position {
        line: 1,
        character: 7,
    };
    assert_eq!(
        detect_class_name_context(content, pos),
        ClassNameContext::UseImport,
        "Top-level `use` should produce UseImport"
    );
}

#[test]
fn test_detect_use_function_context() {
    let content = "<?php\nuse function array";
    let pos = Position {
        line: 1,
        character: 19,
    };
    assert_eq!(
        detect_class_name_context(content, pos),
        ClassNameContext::UseFunction,
        "`use function` should produce UseFunction"
    );
}

#[test]
fn test_detect_use_const_context() {
    let content = "<?php\nuse const PHP";
    let pos = Position {
        line: 1,
        character: 14,
    };
    assert_eq!(
        detect_class_name_context(content, pos),
        ClassNameContext::UseConst,
        "`use const` should produce UseConst"
    );
}

#[test]
fn test_detect_use_inside_class_body_is_trait_use() {
    let content = "<?php\nclass Foo {\n    use Some";
    let pos = Position {
        line: 2,
        character: 12,
    };
    assert_eq!(
        detect_class_name_context(content, pos),
        ClassNameContext::TraitUse,
        "`use` inside class body should remain TraitUse"
    );
}

#[test]
fn test_use_import_is_class_only() {
    assert!(
        ClassNameContext::UseImport.is_class_only(),
        "UseImport should be class-only (no constants or functions)"
    );
}

#[test]
fn test_use_function_is_not_class_only() {
    assert!(
        !ClassNameContext::UseFunction.is_class_only(),
        "UseFunction should NOT be class-only (handler shows functions)"
    );
}

#[test]
fn test_use_const_is_not_class_only() {
    assert!(
        !ClassNameContext::UseConst.is_class_only(),
        "UseConst should NOT be class-only (handler shows constants)"
    );
}

#[test]
fn test_use_import_accepts_all_kinds() {
    assert!(ClassNameContext::UseImport.matches_kind_flags(ClassLikeKind::Class, false, false));
    assert!(ClassNameContext::UseImport.matches_kind_flags(ClassLikeKind::Interface, false, false));
    assert!(ClassNameContext::UseImport.matches_kind_flags(ClassLikeKind::Trait, false, false));
    assert!(ClassNameContext::UseImport.matches_kind_flags(ClassLikeKind::Enum, false, false));
}

#[test]
fn test_detect_use_function_with_fqn_partial() {
    let content = "<?php\nuse function App\\Helpers\\format";
    let pos = Position {
        line: 1,
        character: 35,
    };
    assert_eq!(
        detect_class_name_context(content, pos),
        ClassNameContext::UseFunction,
        "`use function` with namespace-qualified partial should produce UseFunction"
    );
}

#[test]
fn test_detect_use_const_with_fqn_partial() {
    let content = "<?php\nuse const App\\Config\\DB";
    let pos = Position {
        line: 1,
        character: 26,
    };
    assert_eq!(
        detect_class_name_context(content, pos),
        ClassNameContext::UseConst,
        "`use const` with namespace-qualified partial should produce UseConst"
    );
}

// ── NamespaceDeclaration detection ──────────────────────────────

#[test]
fn test_detect_namespace_declaration_context() {
    let content = "<?php\nnamespace App";
    let pos = Position {
        line: 1,
        character: 13,
    };
    assert_eq!(
        detect_class_name_context(content, pos),
        ClassNameContext::NamespaceDeclaration,
        "Top-level `namespace` should produce NamespaceDeclaration"
    );
}

#[test]
fn test_detect_namespace_declaration_with_partial_fqn() {
    let content = "<?php\nnamespace App\\Models";
    let pos = Position {
        line: 1,
        character: 22,
    };
    assert_eq!(
        detect_class_name_context(content, pos),
        ClassNameContext::NamespaceDeclaration,
        "`namespace App\\Models` should produce NamespaceDeclaration"
    );
}

#[test]
fn test_namespace_inside_class_body_is_not_declaration() {
    let content = "<?php\nclass Foo {\n    public function bar() {\n        namespace\n";
    let pos = Position {
        line: 3,
        character: 17,
    };
    assert_ne!(
        detect_class_name_context(content, pos),
        ClassNameContext::NamespaceDeclaration,
        "`namespace` inside class body (brace depth >= 1) should not be NamespaceDeclaration"
    );
}

// ── class_completion_texts edge cases ───────────────────────────

#[test]
fn test_class_completion_texts_fqn_same_namespace_simplifies() {
    let ns = Some("Demo".to_string());
    let (label, insert, _filter, use_import) =
        class_completion_texts("Box", "Demo\\Box", true, true, &ns, "demo\\");
    assert_eq!(label, "Box", "Label should be the relative name");
    assert_eq!(insert, "Box", "Insert text should be the relative name");
    assert!(
        use_import.is_none(),
        "No use import needed for same namespace"
    );
}

#[test]
fn test_class_completion_texts_fqn_different_namespace_keeps_fqn() {
    let ns = Some("Demo".to_string());
    let (label, insert, _filter, use_import) =
        class_completion_texts("Foo", "Other\\Foo", true, true, &ns, "other\\");
    assert_eq!(label, "Other\\Foo", "Label should be the full FQN");
    assert_eq!(
        insert, "\\Other\\Foo",
        "Insert should have leading backslash"
    );
    assert!(use_import.is_none(), "FQN mode never produces a use import");
}

#[test]
fn test_class_completion_texts_non_fqn_always_short_name() {
    let ns: Option<String> = None;
    let (label, insert, _filter, use_import) = class_completion_texts(
        "Dechunk",
        "http\\Encoding\\Dechunk",
        false,
        false,
        &ns,
        "dec",
    );
    assert_eq!(
        label, "Dechunk",
        "Non-FQN mode should always use the short name"
    );
    assert_eq!(insert, "Dechunk");
    assert_eq!(
        use_import.as_deref(),
        Some("http\\Encoding\\Dechunk"),
        "Non-FQN mode should import the full FQN"
    );
}

#[test]
fn test_class_completion_texts_fqn_nested_same_namespace() {
    let ns = Some("Demo".to_string());
    let (label, insert, _filter, use_import) =
        class_completion_texts("Thing", "Demo\\Sub\\Thing", true, true, &ns, "demo\\");
    assert_eq!(
        label, "Sub\\Thing",
        "Nested same-namespace class should use relative path"
    );
    assert_eq!(insert, "Sub\\Thing");
    assert!(use_import.is_none(), "No use import for same namespace");
}

#[test]
fn test_class_completion_texts_leading_backslash_single_segment_same_ns() {
    // Typing `\Demo` (no trailing backslash) in namespace `Demo`.
    // `is_fqn = true` because `has_leading_backslash` is true.
    // `prefix_lower = "demo"` (the normalised, lower-cased prefix).
    let ns = Some("Demo".to_string());
    let (label, insert, _filter, use_import) =
        class_completion_texts("Box", "Demo\\Box", true, true, &ns, "demo");
    assert_eq!(
        label, "Box",
        "Same-namespace class should simplify to short name"
    );
    assert_eq!(
        insert, "Box",
        "Insert text should be 'Box', not '\\Box' or '\\Demo\\Box'"
    );
    assert!(
        use_import.is_none(),
        "No use import needed for same namespace"
    );
}

#[test]
fn test_class_completion_texts_leading_backslash_single_segment_diff_ns() {
    // Typing `\Other` in namespace `Demo` — different namespace.
    let ns = Some("Demo".to_string());
    let (label, insert, _filter, use_import) =
        class_completion_texts("Foo", "Other\\Foo", true, true, &ns, "other");
    assert_eq!(label, "Other\\Foo", "Label should be the full FQN");
    assert_eq!(
        insert, "\\Other\\Foo",
        "Insert should have leading backslash for different namespace"
    );
    assert!(use_import.is_none(), "FQN mode never produces a use import");
}
