//! Hover support (`textDocument/hover`).
//!
//! This module resolves the symbol under the cursor and returns a
//! human-readable description including type information, method
//! signatures, and docblock descriptions.
//!
//! The implementation reuses the same symbol-map lookup that powers
//! go-to-definition, and the same type-resolution pipeline that
//! powers completion.

use tower_lsp::lsp_types::*;

use crate::Backend;
use crate::completion::resolver::ResolutionCtx;
use crate::symbol_map::{SymbolKind, SymbolSpan};
use crate::types::*;

impl Backend {
    /// Handle a `textDocument/hover` request.
    ///
    /// Returns `Some(Hover)` when the symbol under the cursor can be
    /// resolved to a meaningful description, or `None` when resolution
    /// fails or the cursor is not on a navigable symbol.
    pub fn handle_hover(&self, uri: &str, content: &str, position: Position) -> Option<Hover> {
        let offset = Self::position_to_offset(content, position);

        // Fast path: consult precomputed symbol map.
        if let Some(symbol) = self.lookup_symbol_map_for_hover(uri, offset)
            && let Some(Some(hover)) =
                crate::util::catch_panic_unwind_safe("hover", uri, Some(position), || {
                    self.hover_from_symbol(&symbol.kind, uri, content, offset)
                })
        {
            return Some(hover);
        }

        // Retry with offset - 1 for cursor at end-of-token (same
        // heuristic as go-to-definition).
        if offset > 0
            && let Some(symbol) = self.lookup_symbol_map_for_hover(uri, offset - 1)
            && let Some(Some(hover)) =
                crate::util::catch_panic_unwind_safe("hover", uri, Some(position), || {
                    self.hover_from_symbol(&symbol.kind, uri, content, offset - 1)
                })
        {
            return Some(hover);
        }

        None
    }

    /// Look up the symbol at the given byte offset for hover purposes.
    fn lookup_symbol_map_for_hover(&self, uri: &str, offset: u32) -> Option<SymbolSpan> {
        let maps = self.symbol_maps.lock().ok()?;
        let map = maps.get(uri)?;
        map.lookup(offset).cloned()
    }

    /// Dispatch a symbol-map hit to the appropriate hover path.
    fn hover_from_symbol(
        &self,
        kind: &SymbolKind,
        uri: &str,
        content: &str,
        cursor_offset: u32,
    ) -> Option<Hover> {
        let ctx = self.file_context(uri);
        let current_class = Self::find_class_at_offset(&ctx.classes, cursor_offset);
        let class_loader = self.class_loader(&ctx);
        let function_loader = self.function_loader(&ctx);

        match kind {
            SymbolKind::Variable { name } => {
                self.hover_variable(name, uri, content, cursor_offset, current_class, &ctx)
            }

            SymbolKind::MemberAccess {
                subject_text,
                member_name,
                is_static,
                is_method_call,
            } => {
                let rctx = ResolutionCtx {
                    current_class,
                    all_classes: &ctx.classes,
                    content,
                    cursor_offset,
                    class_loader: &class_loader,
                    function_loader: Some(&function_loader),
                };

                let access_kind = if *is_static {
                    AccessKind::DoubleColon
                } else {
                    AccessKind::Arrow
                };

                let candidates = Self::resolve_target_classes(subject_text, access_kind, &rctx);

                for target_class in &candidates {
                    let merged = Self::resolve_class_fully(target_class, &class_loader);

                    if *is_method_call {
                        if let Some(method) = merged
                            .methods
                            .iter()
                            .find(|m| m.name.eq_ignore_ascii_case(member_name))
                        {
                            return Some(self.hover_for_method(method, &merged));
                        }
                    } else {
                        // Try property first, then constant
                        if let Some(prop) =
                            merged.properties.iter().find(|p| p.name == *member_name)
                        {
                            return Some(self.hover_for_property(prop, &merged));
                        }
                        if let Some(constant) =
                            merged.constants.iter().find(|c| c.name == *member_name)
                        {
                            return Some(self.hover_for_constant(constant, &merged));
                        }
                        // Could also be a method reference without call parens
                        if let Some(method) = merged
                            .methods
                            .iter()
                            .find(|m| m.name.eq_ignore_ascii_case(member_name))
                        {
                            return Some(self.hover_for_method(method, &merged));
                        }
                    }
                }
                None
            }

            SymbolKind::ClassReference { name, is_fqn } => {
                self.hover_class_reference(name, *is_fqn, uri, &ctx, &class_loader)
            }

            SymbolKind::ClassDeclaration { name } => {
                // Find the class in the current file's classes
                let cls = ctx.classes.iter().find(|c| c.name == *name)?;
                Some(self.hover_for_class_info(cls))
            }

            SymbolKind::FunctionCall { name } => {
                self.hover_function_call(name, &ctx, &function_loader)
            }

            SymbolKind::SelfStaticParent { keyword } => {
                // `$this` is represented as SelfStaticParent { keyword: "static" }
                // in the symbol map.  Detect it by checking the source text.
                // The cursor may land anywhere inside the `$this` token (5 bytes),
                // so look up to 4 bytes back for the `$` and check for `$this`.
                let is_this = keyword == "static" && {
                    let off = cursor_offset as usize;
                    let search_start = off.saturating_sub(4);
                    let window = content.get(search_start..off + 5).unwrap_or("");
                    window.contains("$this")
                };

                let resolved = match keyword.as_str() {
                    "self" | "static" => current_class.cloned(),
                    "parent" => current_class
                        .and_then(|cc| cc.parent_class.as_ref())
                        .and_then(|parent_name| class_loader(parent_name)),
                    _ => None,
                };
                if let Some(cls) = resolved {
                    let fqn = format_fqn(&cls.name, &cls.file_namespace);
                    let label = if is_this {
                        format!("$this: {}", fqn)
                    } else {
                        format!("{} ({})", keyword, fqn)
                    };
                    let mut lines = vec![format!("```php\n{}\n```", label)];
                    if let Some(desc) = extract_docblock_description(cls.class_docblock.as_deref())
                    {
                        lines.push(desc);
                    }
                    Some(make_hover(lines.join("\n\n")))
                } else {
                    let display = if is_this { "$this" } else { keyword };
                    Some(make_hover(format!("```php\n{}\n```", display)))
                }
            }

            SymbolKind::ConstantReference { name } => {
                // Try to find the constant in global defines
                let defines = self.global_defines.lock().ok()?;
                if defines.contains_key(name.as_str()) {
                    Some(make_hover(format!("```php\nconst {}\n```", name)))
                } else {
                    Some(make_hover(format!("```php\n{}\n```", name)))
                }
            }
        }
    }

    /// Produce hover information for a variable.
    fn hover_variable(
        &self,
        name: &str,
        _uri: &str,
        content: &str,
        cursor_offset: u32,
        current_class: Option<&ClassInfo>,
        ctx: &FileContext,
    ) -> Option<Hover> {
        let var_name = format!("${}", name);

        // $this resolves to the enclosing class
        if name == "this" {
            if let Some(cc) = current_class {
                let fqn = format_fqn(&cc.name, &cc.file_namespace);
                return Some(make_hover(format!("```php\n$this: {}\n```", fqn)));
            }
            return Some(make_hover("```php\n$this\n```".to_string()));
        }

        let class_loader = self.class_loader(ctx);
        let function_loader = self.function_loader(ctx);

        // Use the dummy class approach same as completion for top-level code
        let dummy_class;
        let effective_class = match current_class {
            Some(cc) => cc,
            None => {
                dummy_class = ClassInfo::default();
                &dummy_class
            }
        };

        let types = Self::resolve_variable_types(
            &var_name,
            effective_class,
            &ctx.classes,
            content,
            cursor_offset,
            &class_loader,
            Some(&function_loader as &dyn Fn(&str) -> Option<FunctionInfo>),
        );

        if types.is_empty() {
            return Some(make_hover(format!("```php\n{}\n```", var_name)));
        }

        let type_names: Vec<String> = types
            .iter()
            .map(|c| format_fqn(&c.name, &c.file_namespace))
            .collect();
        let type_str = type_names.join("|");

        Some(make_hover(format!(
            "```php\n{}: {}\n```",
            var_name, type_str
        )))
    }

    /// Produce hover information for a class reference.
    fn hover_class_reference(
        &self,
        name: &str,
        _is_fqn: bool,
        _uri: &str,
        _ctx: &FileContext,
        class_loader: &dyn Fn(&str) -> Option<ClassInfo>,
    ) -> Option<Hover> {
        // Try to resolve the class
        let class_info = class_loader(name);

        if let Some(cls) = class_info {
            Some(self.hover_for_class_info(&cls))
        } else {
            // Unknown class, just show the name
            Some(make_hover(format!("```php\n{}\n```", name)))
        }
    }

    /// Produce hover information for a function call.
    fn hover_function_call(
        &self,
        name: &str,
        _ctx: &FileContext,
        function_loader: &dyn Fn(&str) -> Option<FunctionInfo>,
    ) -> Option<Hover> {
        if let Some(func) = function_loader(name) {
            Some(hover_for_function(&func))
        } else {
            Some(make_hover(format!("```php\nfunction {}()\n```", name)))
        }
    }

    /// Build hover content for a method.
    fn hover_for_method(&self, method: &MethodInfo, owner: &ClassInfo) -> Hover {
        let visibility = format_visibility(method.visibility);
        let static_kw = if method.is_static { "static " } else { "" };
        let params = format_params(&method.parameters);
        let ret = method
            .return_type
            .as_ref()
            .map(|r| format!(": {}", r))
            .unwrap_or_default();

        let signature = format!(
            "{}{} function {}({}){}",
            visibility, static_kw, method.name, params, ret
        );

        let owner_fqn = format_fqn(&owner.name, &owner.file_namespace);

        let mut lines = vec![format!("```php\n{}\n```", signature)];
        lines.push(format!("Class: `{}`", owner_fqn));

        if method.is_deprecated {
            lines.push("**@deprecated**".to_string());
        }

        make_hover(lines.join("\n\n"))
    }

    /// Build hover content for a property.
    fn hover_for_property(&self, property: &PropertyInfo, owner: &ClassInfo) -> Hover {
        let visibility = format_visibility(property.visibility);
        let static_kw = if property.is_static { "static " } else { "" };
        let type_hint = property
            .type_hint
            .as_ref()
            .map(|t| format!("{} ", t))
            .unwrap_or_default();

        let signature = format!(
            "{}{} {}${}",
            visibility, static_kw, type_hint, property.name
        );

        let owner_fqn = format_fqn(&owner.name, &owner.file_namespace);

        let mut lines = vec![format!("```php\n{}\n```", signature)];
        lines.push(format!("Class: `{}`", owner_fqn));

        if property.is_deprecated {
            lines.push("**@deprecated**".to_string());
        }

        make_hover(lines.join("\n\n"))
    }

    /// Build hover content for a class constant.
    fn hover_for_constant(&self, constant: &ConstantInfo, owner: &ClassInfo) -> Hover {
        let visibility = format_visibility(constant.visibility);
        let type_hint = constant
            .type_hint
            .as_ref()
            .map(|t| format!(": {}", t))
            .unwrap_or_default();

        let signature = format!("{} const {}{}", visibility, constant.name, type_hint);

        let owner_fqn = format_fqn(&owner.name, &owner.file_namespace);

        let mut lines = vec![format!("```php\n{}\n```", signature)];
        lines.push(format!("Class: `{}`", owner_fqn));

        if constant.is_deprecated {
            lines.push("**@deprecated**".to_string());
        }

        make_hover(lines.join("\n\n"))
    }

    /// Build hover content for a class/interface/trait/enum.
    fn hover_for_class_info(&self, cls: &ClassInfo) -> Hover {
        let kind_str = match cls.kind {
            ClassLikeKind::Class => {
                if cls.is_abstract {
                    "abstract class"
                } else if cls.is_final {
                    "final class"
                } else {
                    "class"
                }
            }
            ClassLikeKind::Interface => "interface",
            ClassLikeKind::Trait => "trait",
            ClassLikeKind::Enum => "enum",
        };

        let fqn = format_fqn(&cls.name, &cls.file_namespace);

        let mut signature = format!("{} {}", kind_str, fqn);

        if let Some(ref parent) = cls.parent_class {
            signature.push_str(&format!(" extends {}", parent));
        }

        if !cls.interfaces.is_empty() {
            let keyword = if cls.kind == ClassLikeKind::Interface {
                "extends"
            } else {
                "implements"
            };
            signature.push_str(&format!(" {} {}", keyword, cls.interfaces.join(", ")));
        }

        let mut lines = vec![format!("```php\n{}\n```", signature)];

        if cls.is_deprecated {
            lines.push("**@deprecated**".to_string());
        }

        if let Some(desc) = extract_docblock_description(cls.class_docblock.as_deref()) {
            lines.push(desc);
        }

        make_hover(lines.join("\n\n"))
    }
}

// ─── Free helper functions ──────────────────────────────────────────────────

/// Create a `Hover` with Markdown content.
fn make_hover(contents: String) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: contents,
        }),
        range: None,
    }
}

/// Format a visibility keyword.
fn format_visibility(vis: Visibility) -> &'static str {
    match vis {
        Visibility::Public => "public ",
        Visibility::Protected => "protected ",
        Visibility::Private => "private ",
    }
}

/// Format a parameter list for display.
fn format_params(params: &[ParameterInfo]) -> String {
    params
        .iter()
        .map(|p| {
            let mut parts = Vec::new();
            if let Some(ref th) = p.type_hint {
                parts.push(th.clone());
            }
            if p.is_variadic {
                parts.push(format!("...{}", p.name));
            } else if p.is_reference {
                parts.push(format!("&{}", p.name));
            } else {
                parts.push(p.name.clone());
            }
            let param_str = parts.join(" ");
            if !p.is_required && !p.is_variadic {
                format!("{} = ...", param_str)
            } else {
                param_str
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Build a fully-qualified name from a short name and optional namespace.
fn format_fqn(name: &str, namespace: &Option<String>) -> String {
    if let Some(ns) = namespace {
        format!("{}\\{}", ns, name)
    } else {
        name.to_string()
    }
}

/// Build hover content for a standalone function.
fn hover_for_function(func: &FunctionInfo) -> Hover {
    let params = format_params(&func.parameters);
    let ret = func
        .return_type
        .as_ref()
        .map(|r| format!(": {}", r))
        .unwrap_or_default();

    let fqn = if let Some(ref ns) = func.namespace {
        format!("{}\\{}", ns, func.name)
    } else {
        func.name.clone()
    };

    let signature = format!("function {}({}){}", fqn, params, ret);

    let mut lines = vec![format!("```php\n{}\n```", signature)];

    if func.is_deprecated {
        lines.push("**@deprecated**".to_string());
    }

    make_hover(lines.join("\n\n"))
}

/// Extract the human-readable description text from a raw docblock string.
///
/// Strips the `/**` and `*/` delimiters, leading `*` characters, and all
/// `@tag` lines. Returns `None` if no description text remains.
fn extract_docblock_description(docblock: Option<&str>) -> Option<String> {
    let raw = docblock?;
    let inner = raw
        .trim()
        .strip_prefix("/**")
        .unwrap_or(raw)
        .strip_suffix("*/")
        .unwrap_or(raw);

    let mut lines = Vec::new();
    for line in inner.lines() {
        let trimmed = line.trim().trim_start_matches('*').trim();

        // Skip empty lines at the very start
        if lines.is_empty() && trimmed.is_empty() {
            continue;
        }

        // Stop at the first @tag (they come after the description)
        if trimmed.starts_with('@') {
            break;
        }

        lines.push(trimmed.to_string());
    }

    // Trim trailing empty lines
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests;
