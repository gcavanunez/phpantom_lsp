//! Docblock symbol extraction helpers for the symbol map.
//!
//! This module contains functions that scan PHPDoc comment blocks for
//! type references in supported tags (`@param`, `@return`, `@var`,
//! `@template`, `@method`, etc.) and emit [`SymbolSpan`] entries with
//! correct file-level byte offsets.

use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::docblock::types::{split_intersection_depth0, split_type_token, split_union_depth0};

use super::{SymbolKind, SymbolSpan};

// ─── Navigability filter ────────────────────────────────────────────────────

/// Non-navigable type names (scalars, pseudo-types, PHPStan utility types).
/// Types in this list are skipped when extracting docblock symbol spans.
const NON_NAVIGABLE: &[&str] = &[
    "int",
    "integer",
    "float",
    "double",
    "string",
    "bool",
    "boolean",
    "array",
    "object",
    "mixed",
    "void",
    "null",
    "true",
    "false",
    "never",
    "resource",
    "callable",
    "iterable",
    "static",
    "self",
    "parent",
    "class-string",
    "positive-int",
    "negative-int",
    "non-empty-string",
    "non-empty-array",
    "non-empty-list",
    "numeric-string",
    "numeric",
    "scalar",
    "list",
    "non-falsy-string",
    "literal-string",
    "callable-string",
    "array-key",
    "value-of",
    "key-of",
    "int-mask",
    "int-mask-of",
    "no-return",
    "empty",
    "number",
];

/// Returns `true` when a type name refers to a class/interface that the
/// user should be able to navigate to.
pub(crate) fn is_navigable_type(name: &str) -> bool {
    let base = name.split('<').next().unwrap_or(name);
    let base = base.split('{').next().unwrap_or(base);
    let lower = base.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return false;
    }
    !NON_NAVIGABLE.contains(&lower.as_str())
}

// ─── Span construction helpers ──────────────────────────────────────────────

/// Construct a `ClassReference` `SymbolSpan` from a raw identifier string.
///
/// Detects whether the name is fully-qualified (leading `\`) and sets
/// `is_fqn` accordingly.  The leading `\` is stripped from the stored
/// `name` in all cases.
pub(super) fn class_ref_span(start: u32, end: u32, raw_name: &str) -> SymbolSpan {
    let is_fqn = raw_name.starts_with('\\');
    let name = raw_name.strip_prefix('\\').unwrap_or(raw_name).to_string();
    SymbolSpan {
        start,
        end,
        kind: SymbolKind::ClassReference { name, is_fqn },
    }
}

// ─── Docblock text retrieval ────────────────────────────────────────────────

/// Like [`crate::docblock::get_docblock_text_for_node`] but also returns
/// the byte offset of the `/**` opening within the file.
pub fn get_docblock_text_with_offset<'a>(
    trivia: &'a [Trivia<'a>],
    content: &str,
    node: &impl HasSpan,
) -> Option<(&'a str, u32)> {
    let node_start = node.span().start.offset;
    let candidate_idx = trivia.partition_point(|t| t.span.start.offset < node_start);
    if candidate_idx == 0 {
        return None;
    }

    let content_bytes = content.as_bytes();
    let mut covered_from = node_start;

    for i in (0..candidate_idx).rev() {
        let t = &trivia[i];
        let t_end = t.span.end.offset;

        let gap = content_bytes
            .get(t_end as usize..covered_from as usize)
            .unwrap_or(&[]);
        if !gap.iter().all(u8::is_ascii_whitespace) {
            return None;
        }

        match t.kind {
            TriviaKind::DocBlockComment => {
                return Some((t.value, t.span.start.offset));
            }
            TriviaKind::WhiteSpace
            | TriviaKind::SingleLineComment
            | TriviaKind::MultiLineComment
            | TriviaKind::HashComment => {
                covered_from = t.span.start.offset;
            }
        }
    }

    None
}

// ─── Docblock tag scanning ──────────────────────────────────────────────────

/// Scan a docblock for type references in supported tags and emit
/// `SymbolSpan` entries with file-level byte offsets.
///
/// Returns a list of `@template` parameter definitions found in the
/// docblock, each as `(name, byte_offset)`.
pub(super) fn extract_docblock_symbols(
    docblock: &str,
    base_offset: u32,
    spans: &mut Vec<SymbolSpan>,
) -> Vec<(String, u32)> {
    // Tags whose immediate next token is a type.
    const TYPE_FIRST_TAGS: &[&str] = &[
        "@param",
        "@return",
        "@throws",
        "@var",
        "@property",
        "@property-read",
        "@property-write",
        "@mixin",
        "@extends",
        "@implements",
        "@use",
        "@template-extends",
        "@template-implements",
        "@phpstan-return",
        "@phpstan-param",
        "@psalm-return",
        "@psalm-param",
        "@phpstan-var",
        "@psalm-var",
    ];

    let mut line_start: usize = 0;
    let mut template_params: Vec<(String, u32)> = Vec::new();

    for line in docblock.split('\n') {
        if let Some(at_pos) = line.find('@') {
            let tag_start_in_line = at_pos;
            let after_at = &line[tag_start_in_line..];

            let tag_end = after_at
                .find(|c: char| c.is_whitespace())
                .unwrap_or(after_at.len());
            let tag = &after_at[..tag_end];
            let tag_lower = tag.to_ascii_lowercase();

            if tag_lower == "@method" {
                extract_method_tag_symbols(
                    line,
                    tag_start_in_line,
                    tag_end,
                    line_start,
                    base_offset,
                    spans,
                );
                line_start += line.len() + 1;
                continue;
            }

            // @template tags: `@template T of BoundType`
            // @template-covariant / @template-contravariant are variants.
            // The first token after the tag is the parameter name (skip it),
            // then if followed by `of`, the next token is the bound type.
            if tag_lower == "@template"
                || tag_lower == "@template-covariant"
                || tag_lower == "@template-contravariant"
                || tag_lower == "@phpstan-template"
                || tag_lower == "@psalm-template"
                || tag_lower == "@phpstan-template-covariant"
                || tag_lower == "@psalm-template-covariant"
                || tag_lower == "@phpstan-template-contravariant"
                || tag_lower == "@psalm-template-contravariant"
            {
                if let Some(tp) = extract_template_tag_symbols(
                    after_at,
                    tag_end,
                    tag_start_in_line,
                    line_start,
                    base_offset,
                    spans,
                ) {
                    template_params.push(tp);
                }
                line_start += line.len() + 1;
                continue;
            }

            let is_type_first = TYPE_FIRST_TAGS.iter().any(|t| tag_lower == *t);

            if is_type_first {
                let after_tag = &after_at[tag_end..];
                let after_tag_trimmed = after_tag.trim_start();
                if !after_tag_trimmed.is_empty() {
                    let type_start_in_line =
                        tag_start_in_line + tag_end + (after_tag.len() - after_tag_trimmed.len());

                    let (type_token, _remainder) = split_type_token(after_tag_trimmed);
                    if !type_token.is_empty() {
                        emit_type_spans(
                            type_token,
                            base_offset + (line_start + type_start_in_line) as u32,
                            spans,
                        );
                    }
                }
            }
        }

        line_start += line.len() + 1;
    }

    template_params
}

// ─── Type span emission ─────────────────────────────────────────────────────

/// Emit `SymbolSpan` entries for a type token, splitting unions and
/// intersections and skipping scalars.
pub(super) fn emit_type_spans(
    type_token: &str,
    token_file_offset: u32,
    spans: &mut Vec<SymbolSpan>,
) {
    // Split on union `|` at depth 0.
    let union_parts = split_union_depth0(type_token);
    if union_parts.len() > 1 {
        let mut offset = 0usize;
        for part in &union_parts {
            if let Some(pos) = type_token[offset..].find(part) {
                let part_offset = offset + pos;
                emit_type_spans(part.trim(), token_file_offset + part_offset as u32, spans);
                offset = part_offset + part.len();
            }
        }
        return;
    }

    // Split on intersection `&` at depth 0.
    let intersection_parts = split_intersection_depth0(type_token);
    if intersection_parts.len() > 1 {
        let mut offset = 0usize;
        for part in &intersection_parts {
            if let Some(pos) = type_token[offset..].find(part) {
                let part_offset = offset + pos;
                emit_type_spans(part.trim(), token_file_offset + part_offset as u32, spans);
                offset = part_offset + part.len();
            }
        }
        return;
    }

    // Handle PHPStan conditional return types:
    //   ($paramName is Type ? TrueType : FalseType)
    //   ($paramName is not Type ? TrueType : FalseType)
    if type_token.starts_with('(') && type_token.ends_with(')') {
        let inner = &type_token[1..type_token.len() - 1];
        // Look for ` is ` at depth 0 to identify a conditional type.
        if let Some(is_pos) = find_keyword_depth0(inner, " is ") {
            let after_is = &inner[is_pos + 4..];
            // Skip optional `not ` keyword.
            let (after_keyword, keyword_extra) = if let Some(rest) = after_is.strip_prefix("not ") {
                (rest, 4usize)
            } else {
                (after_is, 0usize)
            };
            // Find ` ? ` at depth 0 to separate condition type from true branch.
            if let Some(q_pos) = find_keyword_depth0(after_keyword, " ? ") {
                let condition_type = after_keyword[..q_pos].trim();
                let after_q = &after_keyword[q_pos + 3..];
                // Find ` : ` at depth 0 to separate true branch from false branch.
                if let Some(c_pos) = find_keyword_depth0(after_q, " : ") {
                    let true_type = after_q[..c_pos].trim();
                    let false_type = after_q[c_pos + 3..].trim();

                    // Byte offset of the condition type within the original token.
                    // token_file_offset points at `(`, +1 for `(`, +is_pos for `$param`,
                    // +4 for ` is `, +keyword_extra for optional `not `.
                    let cond_offset_in_inner = is_pos + 4 + keyword_extra;
                    let cond_leading =
                        after_keyword[..q_pos].len() - after_keyword[..q_pos].trim_start().len();
                    let cond_file_offset =
                        token_file_offset + 1 + (cond_offset_in_inner + cond_leading) as u32;
                    if !condition_type.is_empty() {
                        emit_type_spans(condition_type, cond_file_offset, spans);
                    }

                    // True type offset.
                    let true_region = &after_q[..c_pos];
                    let true_leading = true_region.len() - true_region.trim_start().len();
                    let true_offset_in_inner = cond_offset_in_inner + q_pos + 3;
                    let true_file_offset =
                        token_file_offset + 1 + (true_offset_in_inner + true_leading) as u32;
                    if !true_type.is_empty() {
                        emit_type_spans(true_type, true_file_offset, spans);
                    }

                    // False type offset.
                    let false_region = &after_q[c_pos + 3..];
                    let false_leading = false_region.len() - false_region.trim_start().len();
                    let false_offset_in_inner = true_offset_in_inner + c_pos + 3;
                    let false_file_offset =
                        token_file_offset + 1 + (false_offset_in_inner + false_leading) as u32;
                    if !false_type.is_empty() {
                        emit_type_spans(false_type, false_file_offset, spans);
                    }
                    return;
                }
            }
        }
    }

    // Single type — strip nullable prefix.
    let (type_name, extra_offset) = if let Some(rest) = type_token.strip_prefix('?') {
        (rest, 1u32)
    } else {
        (type_token, 0u32)
    };

    if type_name.is_empty() {
        return;
    }

    // Handle `$this` as a self-reference (equivalent to `static`).
    if type_name == "$this" {
        let start = token_file_offset + extra_offset;
        let end = start + type_name.len() as u32;
        spans.push(SymbolSpan {
            start,
            end,
            kind: SymbolKind::SelfStaticParent {
                keyword: "static".to_string(),
            },
        });
        return;
    }

    // Handle callable types: `Closure(ParamType): ReturnType`,
    // `callable(A, B): C`, `\Closure(): Pencil`, etc.
    // Detect by finding `(` at depth 0 (angle/brace) that is *not* at
    // position 0 (position-0 parens are the conditional-type case
    // handled above).
    if let Some(paren_pos) = find_callable_paren(type_name) {
        let base_name = &type_name[..paren_pos];

        // Emit span for the callable base name (e.g. `Closure`, `\Closure`).
        let base_trimmed = base_name
            .split('<')
            .next()
            .unwrap_or(base_name)
            .split('{')
            .next()
            .unwrap_or(base_name);
        let name_for_check = base_trimmed
            .strip_prefix('\\')
            .unwrap_or(base_trimmed)
            .trim();
        if is_navigable_type(name_for_check) {
            let is_fqn = base_trimmed.starts_with('\\');
            let name = base_trimmed
                .strip_prefix('\\')
                .unwrap_or(base_trimmed)
                .trim()
                .to_string();
            let start = token_file_offset + extra_offset;
            let end = start + base_trimmed.len() as u32;
            spans.push(SymbolSpan {
                start,
                end,
                kind: SymbolKind::ClassReference { name, is_fqn },
            });
        }

        // Find matching `)` respecting nesting.
        let inner_start = paren_pos + 1;
        let bytes = type_name.as_bytes();
        let mut depth = 1u32;
        let mut close_paren = inner_start;
        while close_paren < bytes.len() && depth > 0 {
            match bytes[close_paren] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                close_paren += 1;
            }
        }

        if depth == 0 {
            // Recurse into parameter types inside `(...)`.
            let inner = &type_name[inner_start..close_paren];
            if !inner.trim().is_empty() {
                let mut d = 0u32;
                let mut arg_start_idx = 0usize;
                let inner_bytes = inner.as_bytes();
                for i in 0..=inner_bytes.len() {
                    let at_end = i == inner_bytes.len();
                    let is_comma = !at_end && inner_bytes[i] == b',' && d == 0;
                    if (at_end && d == 0) || is_comma {
                        let arg = &inner[arg_start_idx..i];
                        let trimmed = arg.trim();
                        if !trimmed.is_empty() {
                            let leading_ws = arg.len() - arg.trim_start().len();
                            let arg_file_offset = token_file_offset
                                + extra_offset
                                + (inner_start + arg_start_idx + leading_ws) as u32;
                            emit_type_spans(trimmed, arg_file_offset, spans);
                        }
                        arg_start_idx = i + 1;
                    } else if !at_end {
                        match inner_bytes[i] {
                            b'<' | b'(' | b'{' => d += 1,
                            b'>' | b')' | b'}' if d > 0 => d -= 1,
                            _ => {}
                        }
                    }
                }
            }

            // Recurse into the return type after `): `.
            let after_close = &type_name[close_paren + 1..];
            let after_trimmed = after_close.trim_start();
            if let Some(after_colon) = after_trimmed.strip_prefix(':') {
                let ret_trimmed = after_colon.trim_start();
                if !ret_trimmed.is_empty() {
                    let ret_offset_in_type = type_name.len() - ret_trimmed.len();
                    let ret_file_offset =
                        token_file_offset + extra_offset + ret_offset_in_type as u32;
                    emit_type_spans(ret_trimmed, ret_file_offset, spans);
                }
            }
        }

        return;
    }

    // Strip generic suffix and array suffix to get the base type name.
    let base = type_name.split('<').next().unwrap_or(type_name);
    let base = base.split('{').next().unwrap_or(base);
    let base = base.strip_suffix("[]").unwrap_or(base);

    let name_for_check = base.strip_prefix('\\').unwrap_or(base).trim();

    if is_navigable_type(name_for_check) {
        let is_fqn = base.starts_with('\\');
        let name = base.strip_prefix('\\').unwrap_or(base).trim().to_string();
        let start = token_file_offset + extra_offset;
        let end = start + base.len() as u32;

        spans.push(SymbolSpan {
            start,
            end,
            kind: SymbolKind::ClassReference { name, is_fqn },
        });
    }

    // Recurse into generic type arguments: `Foo<Bar, Baz>` → process `Bar, Baz`.
    if let Some(gen_start) = type_name.find('<') {
        // Find the matching closing `>` (respecting nesting depth).
        let inner_start = gen_start + 1;
        let bytes = type_name.as_bytes();
        let mut depth = 1u32;
        let mut gen_end = inner_start;
        while gen_end < bytes.len() && depth > 0 {
            match bytes[gen_end] {
                b'<' => depth += 1,
                b'>' => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                gen_end += 1;
            }
        }
        if depth == 0 {
            let inner = &type_name[inner_start..gen_end];
            // Split on `,` at depth 0 to get individual type arguments.
            let mut d = 0u32;
            let mut arg_start_idx = 0usize;
            let inner_bytes = inner.as_bytes();
            for i in 0..=inner_bytes.len() {
                let at_end = i == inner_bytes.len();
                let is_comma = !at_end && inner_bytes[i] == b',' && d == 0;
                if at_end && d == 0 || is_comma {
                    let arg = &inner[arg_start_idx..i];
                    let trimmed = arg.trim();
                    if !trimmed.is_empty() {
                        // Compute the offset of the trimmed arg within inner.
                        let leading_ws = arg.len() - arg.trim_start().len();
                        let arg_file_offset = token_file_offset
                            + extra_offset
                            + (inner_start + arg_start_idx + leading_ws) as u32;
                        emit_type_spans(trimmed, arg_file_offset, spans);
                    }
                    arg_start_idx = i + 1;
                } else if !at_end {
                    match inner_bytes[i] {
                        b'<' | b'(' | b'{' => d += 1,
                        b'>' | b')' | b'}' if d > 0 => d -= 1,
                        _ => {}
                    }
                }
            }
        }
    }
}

// ─── Callable / keyword helpers ─────────────────────────────────────────────

/// Find the byte position of a `(` that starts a callable parameter list
/// within a type string.  Returns `None` when there is no `(` at
/// angle-bracket / brace depth 0 or when `(` is at position 0 (which
/// indicates a conditional / grouped type, not a callable).
fn find_callable_paren(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth_angle = 0i32;
    let mut depth_brace = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'<' => depth_angle += 1,
            b'>' if depth_angle > 0 => depth_angle -= 1,
            b'{' => depth_brace += 1,
            b'}' if depth_brace > 0 => depth_brace -= 1,
            b'(' if depth_angle == 0 && depth_brace == 0 && i > 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Find the byte position of `keyword` (e.g. `" is "`, `" ? "`, `" : "`)
/// within `s` at parenthesis/angle-bracket depth 0.  Returns `None` when
/// the keyword only appears inside nested delimiters.
fn find_keyword_depth0(s: &str, keyword: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let kw_bytes = keyword.as_bytes();
    let kw_len = kw_bytes.len();
    if bytes.len() < kw_len {
        return None;
    }
    let mut depth = 0i32;
    for i in 0..=bytes.len() - kw_len {
        match bytes[i] {
            b'<' | b'(' | b'{' => depth += 1,
            b'>' | b')' | b'}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {}
        }
        if depth == 0 && &bytes[i..i + kw_len] == kw_bytes {
            return Some(i);
        }
    }
    None
}

// ─── @template tag extraction ───────────────────────────────────────────────

/// Handle `@template` (and variants) tags which have the form:
/// `@template T of BoundType`
///
/// The first token after the tag is the template parameter name — its
/// `(name, byte_offset)` pair is returned so the caller can record a
/// [`super::TemplateParamDef`].  If followed by the keyword `of`, the next
/// token is the bound type which is emitted as a `ClassReference`.
fn extract_template_tag_symbols(
    after_at: &str,
    tag_end: usize,
    tag_start_in_line: usize,
    line_start: usize,
    base_offset: u32,
    spans: &mut Vec<SymbolSpan>,
) -> Option<(String, u32)> {
    // Skip the tag itself to get to the template parameter name.
    let after_tag = &after_at[tag_end..];
    let after_tag_trimmed = after_tag.trim_start();
    if after_tag_trimmed.is_empty() {
        return None;
    }

    // The first non-whitespace token is the parameter name (e.g. `T`, `TNode`).
    let param_end = after_tag_trimmed
        .find(|c: char| c.is_whitespace())
        .unwrap_or(after_tag_trimmed.len());

    let param_name = &after_tag_trimmed[..param_end];
    // Compute the byte offset of the param name within the file.
    let param_offset_in_after_at = after_at.len() - after_tag_trimmed.len();
    let param_file_offset =
        base_offset + (line_start + tag_start_in_line + param_offset_in_after_at) as u32;

    let result = Some((param_name.to_string(), param_file_offset));

    let after_param = &after_tag_trimmed[param_end..];
    let after_param_trimmed = after_param.trim_start();

    // Check for `of` keyword.
    if !after_param_trimmed.starts_with("of ") && !after_param_trimmed.starts_with("of\t") {
        return result;
    }

    // Skip `of` and whitespace to get to the bound type.
    let after_of = &after_param_trimmed[2..]; // skip "of"
    let after_of_trimmed = after_of.trim_start();
    if after_of_trimmed.is_empty() {
        return result;
    }

    // Compute the offset of the bound type within the original line.
    // after_at starts at tag_start_in_line within the line.
    // after_of_trimmed starts at:
    //   tag_start_in_line + tag_end + (whitespace before param)
    //   + param_end + (whitespace before "of") + 2 + (whitespace after "of")
    let bound_offset_in_after_at = after_at.len() - after_of_trimmed.len();
    let bound_start_in_line = tag_start_in_line + bound_offset_in_after_at;

    let (type_token, _remainder) = split_type_token(after_of_trimmed);
    if !type_token.is_empty() {
        emit_type_spans(
            type_token,
            base_offset + (line_start + bound_start_in_line) as u32,
            spans,
        );
    }

    result
}

// ─── @method tag extraction ─────────────────────────────────────────────────

/// Handle `@method` tags which have the form:
/// `@method [static] ReturnType methodName(ParamType $p, ...)`
fn extract_method_tag_symbols(
    line: &str,
    tag_start_in_line: usize,
    tag_end_in_tag: usize,
    line_start: usize,
    base_offset: u32,
    spans: &mut Vec<SymbolSpan>,
) {
    let after_tag = &line[tag_start_in_line + tag_end_in_tag..];
    let after_tag_trimmed = after_tag.trim_start();
    if after_tag_trimmed.is_empty() {
        return;
    }

    let mut rest = after_tag_trimmed;
    let mut rest_offset =
        tag_start_in_line + tag_end_in_tag + (after_tag.len() - after_tag_trimmed.len());

    // Skip optional `static` keyword.
    if rest.starts_with("static ") || rest.starts_with("static\t") {
        let skip = "static".len();
        let after_static = rest[skip..].trim_start();
        let whitespace_len = rest.len() - skip - after_static.len();
        rest_offset += skip + whitespace_len;
        rest = after_static;
    }

    if rest.is_empty() {
        return;
    }

    // Extract return type.
    let (return_type, remainder) = split_type_token(rest);
    if !return_type.is_empty() {
        emit_type_spans(
            return_type,
            base_offset + (line_start + rest_offset) as u32,
            spans,
        );
    }

    // After the return type, find the `(` for parameter list.
    if let Some(paren_pos) = remainder.find('(') {
        let close = remainder[paren_pos..].find(')');
        if let Some(close_pos) = close {
            let inner = &remainder[paren_pos + 1..paren_pos + close_pos];
            let inner_offset_in_line = rest_offset
                + return_type.len()
                + (remainder.len() - rest[return_type.len()..].len())
                + paren_pos
                + 1;

            // Simple comma-split at depth 0 for parameters.
            let mut depth = 0i32;
            let mut param_start = 0usize;

            for (i, ch) in inner.char_indices() {
                match ch {
                    '<' | '(' | '{' => depth += 1,
                    '>' | ')' | '}' => depth -= 1,
                    ',' if depth == 0 => {
                        let param = inner[param_start..i].trim();
                        emit_method_param_type(
                            param,
                            line_start,
                            inner_offset_in_line,
                            param_start,
                            base_offset,
                            spans,
                        );
                        param_start = i + 1;
                    }
                    _ => {}
                }
            }
            // Last parameter.
            let param = inner[param_start..].trim();
            emit_method_param_type(
                param,
                line_start,
                inner_offset_in_line,
                param_start,
                base_offset,
                spans,
            );
        }
    }
}

/// Emit a type span for a single parameter in a `@method` tag's parameter list.
fn emit_method_param_type(
    param: &str,
    line_start: usize,
    inner_offset_in_line: usize,
    param_start_in_inner: usize,
    base_offset: u32,
    spans: &mut Vec<SymbolSpan>,
) {
    if param.is_empty() {
        return;
    }
    // A parameter looks like `TypeHint $varName` or just `$varName`.
    if let Some(dollar_pos) = param.find('$') {
        let type_part = param[..dollar_pos].trim();
        if !type_part.is_empty() {
            let type_start_in_param = param.find(type_part).unwrap_or(0);
            let (type_token, _) = split_type_token(type_part);
            if !type_token.is_empty() {
                let file_offset = base_offset
                    + (line_start
                        + inner_offset_in_line
                        + param_start_in_inner
                        + type_start_in_param) as u32;
                emit_type_spans(type_token, file_offset, spans);
            }
        }
    }
}
