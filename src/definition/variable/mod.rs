/// Variable definition resolution.
///
/// This module handles go-to-definition for `$variable` references,
/// jumping from a variable usage to its most recent assignment or
/// declaration site.
///
/// The primary path parses the file into an AST and walks the enclosing
/// scope to find the variable's definition site with byte-accurate
/// offsets.  This correctly handles:
///   - Array destructuring: `[$a, $b] = explode(',', $str)`
///   - List destructuring:  `list($a, $b) = func()`
///   - Multi-line parameter lists
///   - Nested scopes (closures, arrow functions)
///
/// Supported definition sites (searched bottom-up from cursor):
///   - **Assignment**: `$var = …` (but not `==` / `===`)
///   - **Parameter**: `Type $var` in a function/method signature
///   - **Foreach**: `as $var` / `=> $var`
///   - **Catch**: `catch (…Exception $var)`
///   - **Static / global**: `static $var` / `global $var`
///   - **Array destructuring**: `[$a, $b] = …` / `list($a, $b) = …`
///
/// When the cursor is already at the definition site (e.g. on a
/// parameter), the module falls through to type-hint resolution:
/// it extracts the type hint and jumps to the first class-like type
/// in it (e.g. `HtmlString` in `HtmlString|string $content`).
///
/// When the AST parse fails (malformed PHP, parser panic), the function
/// returns `None` rather than falling back to text heuristics.
///
/// ## Submodules
///
/// - [`var_definition`]: AST walk that finds variable definition sites
///   (assignments, parameters, foreach, catch, static/global,
///   destructuring).
/// - [`type_hint`]: AST walk that extracts the type hint string at a
///   variable's definition site (parameter, property, closure/arrow
///   function parameter).
use tower_lsp::lsp_types::*;

use crate::Backend;
use crate::composer;
use crate::parser::with_parsed_program;
use crate::util::offset_to_position;

mod type_hint;
mod var_definition;

use type_hint::find_type_hint_at_definition;
use var_definition::find_variable_definition_in_program;

// ═══════════════════════════════════════════════════════════════════════
// AST-based variable definition search result
// ═══════════════════════════════════════════════════════════════════════

/// Result of searching for a variable definition in the AST.
#[derive(Default)]
pub(super) enum VarDefSearchResult {
    /// No definition site found for this variable in the current scope.
    #[default]
    NotFound,
    /// The cursor is already sitting on the definition site (e.g. on a
    /// parameter declaration).  The caller should fall through to
    /// type-hint resolution.
    AtDefinition,
    /// Found a prior definition at the given byte offset.
    /// `offset` is the start of the `$var` token, `end_offset` is the end.
    FoundAt { offset: u32, end_offset: u32 },
}

impl Backend {
    // ──────────────────────────────────────────────────────────────────────
    // Variable go-to-definition helpers
    // ──────────────────────────────────────────────────────────────────────

    /// Returns `true` when the cursor is sitting on a `$variable` token.
    ///
    /// `extract_word_at_position` strips `$`, so we peek at the character
    /// immediately before the word to see if it is `$`.
    pub(super) fn cursor_is_on_variable(content: &str, position: Position, _word: &str) -> bool {
        let lines: Vec<&str> = content.lines().collect();
        let line_idx = position.line as usize;
        if line_idx >= lines.len() {
            return false;
        }
        let line = lines[line_idx];
        let chars: Vec<char> = line.chars().collect();
        let col = (position.character as usize).min(chars.len());

        // Find where `word` starts on this line (same logic as
        // extract_word_at_position: walk left from cursor).
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_' || c == '\\';
        let mut start = col;
        if start < chars.len() && is_word_char(chars[start]) {
            // on a word char
        } else if start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        } else {
            return false;
        }
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

        // The character just before the word must be `$`.
        if start == 0 {
            return false;
        }
        if chars[start - 1] != '$' {
            return false;
        }

        // If the `$` is preceded by `::`, this is a static property access
        // (e.g. `Config::$defaultLocale`), not a local variable.
        if start >= 3 && chars[start - 2] == ':' && chars[start - 3] == ':' {
            return false;
        }

        true
    }

    /// Find the most recent assignment or declaration of `$var_name` before
    /// `position` and return its location.
    ///
    /// Parses the file into an AST and walks the enclosing scope to find
    /// the definition site with exact byte offsets.  Returns `None` when
    /// the AST parse fails or no definition is found.
    pub(super) fn resolve_variable_definition(
        content: &str,
        uri: &str,
        position: Position,
        var_name: &str,
    ) -> Option<Location> {
        Self::resolve_variable_definition_ast(content, uri, position, var_name)?
    }

    /// AST-based variable definition resolution.
    ///
    /// Returns:
    /// - `Some(Some(location))` — found a prior definition, jump there
    /// - `Some(None)` — cursor is at a definition site (fall through to type-hint)
    ///   OR no definition found in the AST (don't fall back to text)
    /// - `None` — AST parse failed, caller should try the text-based fallback
    fn resolve_variable_definition_ast(
        content: &str,
        uri: &str,
        position: Position,
        var_name: &str,
    ) -> Option<Option<Location>> {
        let cursor_offset = Self::position_to_offset(content, position);

        let result = with_parsed_program(
            content,
            "resolve_variable_definition_ast",
            |program, content| {
                find_variable_definition_in_program(program, content, var_name, cursor_offset)
            },
        );

        match result {
            VarDefSearchResult::NotFound => {
                // The AST parse succeeded but found no definition — return
                // Some(None) so the caller knows not to fall back to text.
                Some(None)
            }
            VarDefSearchResult::AtDefinition => {
                // Cursor is at the definition — return Some(None) so the
                // caller falls through to type-hint resolution.
                Some(None)
            }
            VarDefSearchResult::FoundAt { offset, end_offset } => {
                let target_uri = Url::parse(uri).ok()?;
                let start_pos = offset_to_position(content, offset as usize);
                let end_pos = offset_to_position(content, end_offset as usize);
                Some(Some(Location {
                    uri: target_uri,
                    range: Range {
                        start: start_pos,
                        end: end_pos,
                    },
                }))
            }
        }
    }

    /// Find a whole-word occurrence of `var_name` in `line`, skipping
    /// partial matches like `$item` inside `$items`.
    fn find_whole_var(line: &str, var_name: &str) -> Option<usize> {
        let is_ident_char = |c: char| c.is_alphanumeric() || c == '_';
        let mut start = 0;
        while let Some(pos) = line[start..].find(var_name) {
            let abs = start + pos;
            let after = abs + var_name.len();
            let boundary_ok =
                after >= line.len() || !line[after..].starts_with(|c: char| is_ident_char(c));
            if boundary_ok {
                return Some(abs);
            }
            start = abs + 1;
        }
        None
    }

    // ─── Type-Hint Resolution at Variable Definition ────────────────────

    /// When the cursor is on a variable that is already at its definition
    /// site (parameter, property, promoted property), extract the type hint
    /// and jump to the first class-like type in it.
    ///
    /// For example, given `public readonly HtmlString|string $content,`
    /// this returns the location of the `HtmlString` class definition.
    pub(super) fn resolve_type_hint_at_variable(
        &self,
        uri: &str,
        content: &str,
        position: Position,
        var_name: &str,
    ) -> Option<Location> {
        // Try AST-based type-hint extraction first.
        if let Some(result) =
            self.resolve_type_hint_at_variable_ast(uri, content, position, var_name)
        {
            return Some(result);
        }

        // Fall back to text-based extraction.
        self.resolve_type_hint_at_variable_text(uri, content, position, var_name)
    }

    /// AST-based type-hint resolution: extract the type hint from the AST
    /// node where the variable is defined (parameter, catch, property).
    fn resolve_type_hint_at_variable_ast(
        &self,
        uri: &str,
        content: &str,
        position: Position,
        var_name: &str,
    ) -> Option<Location> {
        let cursor_offset = Self::position_to_offset(content, position);

        let type_hint_str: Option<String> = with_parsed_program(
            content,
            "resolve_type_hint_at_variable_ast",
            |program, _| find_type_hint_at_definition(program, var_name, cursor_offset),
        );

        let type_hint = type_hint_str?;
        self.resolve_type_hint_string_to_location(uri, content, &type_hint)
    }

    /// Text-based type-hint resolution (original implementation).
    fn resolve_type_hint_at_variable_text(
        &self,
        uri: &str,
        content: &str,
        position: Position,
        var_name: &str,
    ) -> Option<Location> {
        let lines: Vec<&str> = content.lines().collect();
        let line_idx = position.line as usize;
        if line_idx >= lines.len() {
            return None;
        }
        let line = lines[line_idx];

        let var_pos = Self::find_whole_var(line, var_name)?;

        let before_raw = line[..var_pos].trim_end();

        let before = match before_raw.rfind('(') {
            Some(pos) => before_raw[pos + 1..].trim_start(),
            None => before_raw,
        };

        let type_hint = match before.rsplit_once(char::is_whitespace) {
            Some((_, t)) => t,
            None => before,
        };
        if type_hint.is_empty() {
            return None;
        }

        self.resolve_type_hint_string_to_location(uri, content, type_hint)
    }

    /// Given a type-hint string (e.g. `HtmlString|string`, `?Foo`),
    /// resolve it to the definition location of the first class-like type.
    fn resolve_type_hint_string_to_location(
        &self,
        uri: &str,
        content: &str,
        type_hint: &str,
    ) -> Option<Location> {
        let scalars = [
            "string", "int", "float", "bool", "array", "callable", "iterable", "object", "mixed",
            "void", "never", "null", "false", "true", "self", "static", "parent",
        ];

        let class_name = type_hint
            .split(['|', '&'])
            .map(|t| t.trim_start_matches('?'))
            .find(|t| !t.is_empty() && !scalars.contains(&t.to_lowercase().as_str()))?;

        let ctx = self.file_context(uri);

        let fqn = Self::resolve_to_fqn(class_name, &ctx.use_map, &ctx.namespace);

        let mut candidates = vec![fqn];
        if class_name.contains('\\') && !candidates.contains(&class_name.to_string()) {
            candidates.push(class_name.to_string());
        }

        // Try same-file first.
        for fqn in &candidates {
            if let Some(location) = self.find_definition_in_ast_map(fqn, content, uri) {
                return Some(location);
            }
        }

        // Try PSR-4 resolution.
        // resolve_class_in_file parses, caches, and uses keyword_offset
        // (AST-based), falling back to text search only when the parser
        // fails.
        let workspace_root = self
            .workspace_root
            .lock()
            .ok()
            .and_then(|guard| guard.clone());

        if let Some(workspace_root) = workspace_root
            && let Ok(mappings) = self.psr4_mappings.lock()
        {
            for fqn in &candidates {
                if let Some(file_path) =
                    composer::resolve_class_path(&mappings, &workspace_root, fqn)
                    && let Some(location) = self.resolve_class_in_file(&file_path, fqn)
                {
                    return Some(location);
                }
            }
        }

        None
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests;
