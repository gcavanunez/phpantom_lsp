/// Completion-related modules.
///
/// This sub-module groups all completion logic:
/// - **handler**: Top-level completion request orchestration
/// - **target**: Extracting the completion target (access operator and subject)
/// - **resolver**: Resolving the subject to a concrete class type
/// - **call_resolution**: Call expression and callable target resolution (method
///   calls, static calls, function calls, constructor calls, signature help,
///   named-argument completion)
/// - **type_resolution**: Type-hint string to `ClassInfo` mapping (unions,
///   intersections, generics, type aliases, object shapes, property types)
/// - **builder**: Building LSP `CompletionItem`s from resolved class info
/// - **class_completion**: Class name completions (class, interface, trait, enum)
/// - **constant_completion**: Global constant name completions
/// - **function_completion**: Standalone function name completions
/// - **namespace_completion**: Namespace declaration completions
/// - **variable_completion**: Variable name completions and scope collection
/// - **phpdoc**: PHPDoc tag completion inside `/** … */` blocks
/// - **phpdoc_context**: PHPDoc context detection and symbol info extraction
///   (`DocblockContext`, `SymbolInfo`, `detect_context`, `extract_symbol_info`,
///   `detect_docblock_typing_position`, `extract_phpdoc_prefix`)
/// - **named_args**: Named argument completion inside function/method call parens
/// - **array_shape**: Array shape key completion (`$arr['` → suggest known keys)
///   and raw variable type resolution for array shape value chaining
/// - **comment_position**: Comment and docblock position detection (`is_inside_docblock`,
///   `is_inside_non_doc_comment`, `position_to_byte_offset`)
/// - **throws_analysis**: Throws analysis pipeline (throw scanning, catch-block filtering,
///   uncaught detection, method `@throws` / return-type lookup, import helpers)
///   used by both phpdoc and catch_completion
/// - **foreach_resolution**: Foreach value/key and array destructuring type resolution
///   (extracted from `variable_resolution` for navigability)
/// - **catch_completion**: Smart exception type completion inside `catch()` clauses
/// - **conditional_resolution**: PHPStan conditional return type resolution at call sites
/// - **type_narrowing**: instanceof / assert / custom type guard narrowing
/// - **type_hint_completion**: Type completion inside function/method parameter lists,
///   return types, and property declarations (offers native PHP types + class names)
/// - **source_helpers**: Source-text scanning helpers (closure return types,
///   first-class callable resolution, `new` expression parsing, array access)
/// - **variable_resolution**: Variable type resolution via assignment scanning
/// - **rhs_resolution**: Right-hand-side expression resolution for variable
///   assignments (instantiation, array access, function/method/static calls,
///   property access, match, ternary, clone)
/// - **class_string_resolution**: Class-string variable resolution (`$cls = User::class`)
/// - **raw_type_inference**: Raw type inference for variable assignments (array shapes,
///   array functions, generator yields)
/// - **closure_resolution**: Closure and arrow-function parameter resolution
///
/// Class inheritance merging (traits, mixins, parent chain) lives in the
/// top-level `crate::inheritance` module since it is shared infrastructure
/// used by completion, definition, and future features (hover, references).
pub mod array_shape;
pub(crate) mod builder;
pub(crate) mod call_resolution;
pub(crate) mod catch_completion;
pub(crate) mod class_completion;
pub(crate) mod class_string_resolution;
pub(crate) mod closure_resolution;
pub mod comment_position;
pub(crate) mod conditional_resolution;
pub(crate) mod constant_completion;
pub(crate) mod foreach_resolution;
pub(crate) mod function_completion;
pub(crate) mod handler;
pub mod named_args;
pub(crate) mod namespace_completion;
pub mod phpdoc;
pub(crate) mod phpdoc_context;
pub(crate) mod raw_type_inference;
pub(crate) mod resolver;
pub(crate) mod rhs_resolution;
pub(crate) mod source_helpers;
pub(crate) mod target;
pub(crate) mod throws_analysis;
pub(crate) mod type_hint_completion;
pub(crate) mod type_narrowing;
pub(crate) mod type_resolution;
pub(crate) mod use_edit;
pub(crate) mod variable_completion;
pub(crate) mod variable_resolution;
