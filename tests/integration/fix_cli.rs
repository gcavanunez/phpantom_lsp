//! Integration tests for the `fix` CLI module.
//!
//! Tests exercise the unused-import fixer with a real `Backend` to
//! verify end-to-end correctness: parse → detect unused → build edits
//! → apply edits → verify output.

use crate::common::create_test_backend;
use phpantom_lsp::Backend;

/// Helper: parse a PHP file into the backend and run the unused-import
/// fixer, returning the fixed content.
fn fix_unused_imports(backend: &Backend, uri: &str, content: &str) -> String {
    backend.update_ast(uri, content);

    let mut diagnostics = Vec::new();
    backend.collect_unused_import_diagnostics(uri, content, &mut diagnostics);

    if diagnostics.is_empty() {
        return content.to_string();
    }

    use std::collections::HashSet;
    use tower_lsp::lsp_types::*;

    let removed_import_lines: HashSet<usize> = diagnostics
        .iter()
        .map(|d| d.range.start.line as usize)
        .collect();

    let mut edits: Vec<TextEdit> = diagnostics
        .iter()
        .map(|d| {
            phpantom_lsp::code_actions::build_line_deletion_edit(
                content,
                &d.range,
                &removed_import_lines,
            )
        })
        .collect();

    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));

    apply_text_edits(content, &edits)
}

/// Apply reverse-sorted text edits to content.
fn apply_text_edits(content: &str, edits: &[tower_lsp::lsp_types::TextEdit]) -> String {
    let mut result = content.to_string();

    for edit in edits {
        let start = lsp_position_to_byte_offset(&result, edit.range.start);
        let end = lsp_position_to_byte_offset(&result, edit.range.end);

        if start <= end && end <= result.len() {
            result.replace_range(start..end, &edit.new_text);
        }
    }

    result
}

/// Convert LSP Position to byte offset.
fn lsp_position_to_byte_offset(content: &str, pos: tower_lsp::lsp_types::Position) -> usize {
    let mut offset = 0;
    for (i, line) in content.lines().enumerate() {
        if i == pos.line as usize {
            let mut utf16_units = 0u32;
            for (byte_idx, ch) in line.char_indices() {
                if utf16_units >= pos.character {
                    return offset + byte_idx;
                }
                utf16_units += ch.len_utf16() as u32;
            }
            return offset + line.len();
        }
        offset += line.len() + 1;
    }
    content.len()
}

// ── Single unused import ────────────────────────────────────────────────────

#[test]
fn removes_single_unused_import() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\User;

class Foo {}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        !result.contains("use App\\Models\\User"),
        "Unused import should be removed. Got:\n{result}"
    );
    assert!(
        result.contains("class Foo {}"),
        "Class declaration should remain"
    );
}

// ── Multiple unused imports ─────────────────────────────────────────────────

#[test]
fn removes_multiple_unused_imports() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\User;
use App\Models\Post;
use App\Models\Comment;

class Foo {}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        !result.contains("use App\\Models\\User"),
        "User import should be removed"
    );
    assert!(
        !result.contains("use App\\Models\\Post"),
        "Post import should be removed"
    );
    assert!(
        !result.contains("use App\\Models\\Comment"),
        "Comment import should be removed"
    );
    assert!(
        result.contains("class Foo {}"),
        "Class declaration should remain"
    );
}

// ── Used import is preserved ────────────────────────────────────────────────

#[test]
fn preserves_used_import() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\User;

class Foo {
    public function bar(): User {
        return new User();
    }
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        result.contains("use App\\Models\\User"),
        "Used import should be preserved. Got:\n{result}"
    );
}

// ── Mix of used and unused imports ──────────────────────────────────────────

#[test]
fn removes_only_unused_from_mixed_imports() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\User;
use App\Models\Post;
use App\Models\Comment;

class Foo {
    public function bar(): User {
        return new User();
    }
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        result.contains("use App\\Models\\User"),
        "Used import (User) should be preserved"
    );
    assert!(
        !result.contains("use App\\Models\\Post"),
        "Unused import (Post) should be removed"
    );
    assert!(
        !result.contains("use App\\Models\\Comment"),
        "Unused import (Comment) should be removed"
    );
}

// ── No imports at all ───────────────────────────────────────────────────────

#[test]
fn no_imports_returns_unchanged() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

class Foo {
    public function bar(): void {}
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);
    assert_eq!(result, content);
}

// ── All imports used ────────────────────────────────────────────────────────

#[test]
fn all_imports_used_returns_unchanged() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\User;
use App\Models\Post;

class Foo {
    public function bar(): User {
        return new User();
    }
    public function baz(): Post {
        return new Post();
    }
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);
    assert_eq!(result, content);
}

// ── Group import with one unused member ─────────────────────────────────────

#[test]
fn removes_unused_member_from_group_import() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\{User, Post};

class Foo {
    public function bar(): User {
        return new User();
    }
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        result.contains("User"),
        "Used member (User) should be preserved"
    );
    assert!(
        !result.contains("Post"),
        "Unused member (Post) should be removed from group"
    );
}

// ── Group import with all members unused ────────────────────────────────────

#[test]
fn removes_entire_group_import_when_all_unused() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\{User, Post};

class Foo {}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        !result.contains("use App\\Models"),
        "Entire group import should be removed. Got:\n{result}"
    );
    assert!(
        result.contains("class Foo {}"),
        "Class declaration should remain"
    );
}

// ── Blank line collapsing ───────────────────────────────────────────────────

#[test]
fn collapses_blank_lines_after_removing_all_imports() {
    let backend = create_test_backend();
    let content = "<?php\n\nnamespace App;\n\nuse App\\Models\\User;\n\nclass Foo {}\n";

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    // Should not have double blank lines where the import was.
    assert!(
        !result.contains("\n\n\n"),
        "Should not leave triple newlines. Got:\n{result}"
    );
}

// ── Static method reference keeps import ────────────────────────────────────

#[test]
fn preserves_import_used_in_static_call() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Utils\Helper;

class Foo {
    public function bar(): void {
        Helper::doSomething();
    }
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        result.contains("use App\\Utils\\Helper"),
        "Import used in static call should be preserved"
    );
}

// ── Import used in type hint ────────────────────────────────────────────────

#[test]
fn preserves_import_used_in_parameter_type_hint() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\User;

class Foo {
    public function bar(User $user): void {}
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        result.contains("use App\\Models\\User"),
        "Import used as parameter type hint should be preserved"
    );
}

// ── Import used in docblock ─────────────────────────────────────────────────

#[test]
fn preserves_import_referenced_in_phpdoc_return_tag() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\User;

class Foo {
    /** @return User */
    public function bar() {}
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        result.contains("use App\\Models\\User"),
        "Import referenced in @return should be preserved"
    );
}

// ── Braced namespace ────────────────────────────────────────────────────────

#[test]
fn removes_unused_import_in_braced_namespace() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App {
    use App\Models\User;
    use App\Models\Post;

    class Foo {
        public function bar(): User {
            return new User();
        }
    }
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        result.contains("use App\\Models\\User"),
        "Used import should be preserved in braced namespace"
    );
    assert!(
        !result.contains("use App\\Models\\Post"),
        "Unused import should be removed from braced namespace"
    );
}

// ── Trait use statement is not removed ───────────────────────────────────────

#[test]
fn does_not_remove_trait_use_statements() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Traits\HasName;

class Foo {
    use HasName;
}
"#;

    let result = fix_unused_imports(&backend, "file:///test.php", content);

    assert!(
        result.contains("use App\\Traits\\HasName"),
        "Namespace-level import for trait should be preserved (used by trait-use inside class)"
    );
}

// ── Idempotency ─────────────────────────────────────────────────────────────

#[test]
fn fix_is_idempotent() {
    let backend = create_test_backend();
    let content = r#"<?php

namespace App;

use App\Models\User;
use App\Models\Post;

class Foo {
    public function bar(): User {
        return new User();
    }
}
"#;

    let first_pass = fix_unused_imports(&backend, "file:///test.php", content);

    // Re-parse with the fixed content and fix again.
    let second_pass = fix_unused_imports(&backend, "file:///test.php", &first_pass);

    assert_eq!(
        first_pass, second_pass,
        "Running fix twice should produce the same result"
    );
}