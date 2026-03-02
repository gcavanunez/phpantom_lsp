/// Type-hint extraction at variable definition sites.
///
/// This submodule contains the AST walk that finds the type hint string
/// for a variable at its definition site (parameter, property, promoted
/// property, closure/arrow function parameter).
///
/// The entry point is [`find_type_hint_at_definition`], called from the
/// `Backend` methods in the parent `variable` module.
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::parser::extract_hint_string;

/// Find the type hint string for a variable at its definition site
/// in the AST.
pub(super) fn find_type_hint_at_definition(
    program: &Program<'_>,
    var_name: &str,
    cursor_offset: u32,
) -> Option<String> {
    find_type_hint_in_statements(program.statements.iter(), var_name, cursor_offset)
}

/// Walk statements looking for the scope that contains the cursor,
/// then extract the type hint.
fn find_type_hint_in_statements<'a, I>(
    statements: I,
    var_name: &str,
    cursor_offset: u32,
) -> Option<String>
where
    I: Iterator<Item = &'a Statement<'a>>,
{
    for stmt in statements {
        match stmt {
            Statement::Class(class) => {
                let start = class.left_brace.start.offset;
                let end = class.right_brace.end.offset;
                if cursor_offset >= start && cursor_offset <= end {
                    return find_type_hint_in_class_members(
                        class.members.iter(),
                        var_name,
                        cursor_offset,
                    );
                }
            }
            Statement::Interface(iface) => {
                let start = iface.left_brace.start.offset;
                let end = iface.right_brace.end.offset;
                if cursor_offset >= start && cursor_offset <= end {
                    return find_type_hint_in_class_members(
                        iface.members.iter(),
                        var_name,
                        cursor_offset,
                    );
                }
            }
            Statement::Trait(trait_def) => {
                let start = trait_def.left_brace.start.offset;
                let end = trait_def.right_brace.end.offset;
                if cursor_offset >= start && cursor_offset <= end {
                    return find_type_hint_in_class_members(
                        trait_def.members.iter(),
                        var_name,
                        cursor_offset,
                    );
                }
            }
            Statement::Enum(enum_def) => {
                let start = enum_def.left_brace.start.offset;
                let end = enum_def.right_brace.end.offset;
                if cursor_offset >= start && cursor_offset <= end {
                    return find_type_hint_in_class_members(
                        enum_def.members.iter(),
                        var_name,
                        cursor_offset,
                    );
                }
            }
            Statement::Namespace(ns) => {
                if let Some(hint) =
                    find_type_hint_in_statements(ns.statements().iter(), var_name, cursor_offset)
                {
                    return Some(hint);
                }
            }
            Statement::Function(func) => {
                // Check parameter list span (cursor might be on a
                // parameter declaration, which is outside the body).
                let param_span = func.parameter_list.span();
                if cursor_offset >= param_span.start.offset
                    && cursor_offset <= param_span.end.offset
                {
                    return find_type_hint_in_params(&func.parameter_list, var_name, cursor_offset);
                }
                let body_start = func.body.left_brace.start.offset;
                let body_end = func.body.right_brace.end.offset;
                if cursor_offset >= body_start && cursor_offset <= body_end {
                    return find_type_hint_in_params(&func.parameter_list, var_name, cursor_offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// Search class members for a method containing the cursor, then
/// extract the type hint from its parameters or promoted properties.
fn find_type_hint_in_class_members<'a, I>(
    members: I,
    var_name: &str,
    cursor_offset: u32,
) -> Option<String>
where
    I: Iterator<Item = &'a ClassLikeMember<'a>>,
{
    for member in members {
        match member {
            ClassLikeMember::Method(method) => {
                if let MethodBody::Concrete(body) = &method.body {
                    let body_start = body.left_brace.start.offset;
                    let body_end = body.right_brace.end.offset;
                    if cursor_offset >= body_start && cursor_offset <= body_end {
                        // Check if cursor is on a closure/arrow function parameter.
                        let body_stmts: Vec<&Statement> = body.statements.iter().collect();
                        if let Some(hint) =
                            find_type_hint_in_nested_closure(&body_stmts, var_name, cursor_offset)
                        {
                            return Some(hint);
                        }
                        return find_type_hint_in_params(
                            &method.parameter_list,
                            var_name,
                            cursor_offset,
                        );
                    }
                }
                // Also check parameter list span directly (cursor might
                // be on the parameter itself, outside the body).
                let param_span = method.parameter_list.span();
                if cursor_offset >= param_span.start.offset
                    && cursor_offset <= param_span.end.offset
                {
                    return find_type_hint_in_params(
                        &method.parameter_list,
                        var_name,
                        cursor_offset,
                    );
                }
            }
            ClassLikeMember::Property(property) => {
                // Check property variables.
                for var in property.variables().iter() {
                    if var.name == var_name {
                        let var_start = var.span.start.offset;
                        let var_end = var.span.end.offset;
                        if cursor_offset >= var_start && cursor_offset < var_end {
                            return property.hint().map(|h| extract_hint_string(h));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if cursor is on a closure/arrow function parameter and extract
/// its type hint.
fn find_type_hint_in_nested_closure(
    stmts: &[&Statement<'_>],
    var_name: &str,
    cursor_offset: u32,
) -> Option<String> {
    for &stmt in stmts {
        let stmt_span = stmt.span();
        if cursor_offset < stmt_span.start.offset || cursor_offset > stmt_span.end.offset {
            continue;
        }
        if let Some(hint) = find_type_hint_in_closure_stmt(stmt, var_name, cursor_offset) {
            return Some(hint);
        }
    }
    None
}

/// Recursively search a statement for a closure/arrow function whose
/// parameter the cursor is on.
fn find_type_hint_in_closure_stmt(
    stmt: &Statement<'_>,
    var_name: &str,
    cursor_offset: u32,
) -> Option<String> {
    match stmt {
        Statement::Expression(expr_stmt) => {
            find_type_hint_in_closure_expr(expr_stmt.expression, var_name, cursor_offset)
        }
        Statement::Return(ret) => ret
            .value
            .and_then(|expr| find_type_hint_in_closure_expr(expr, var_name, cursor_offset)),
        Statement::If(if_stmt) => {
            for inner in if_stmt.body.statements() {
                if let Some(h) = find_type_hint_in_closure_stmt(inner, var_name, cursor_offset) {
                    return Some(h);
                }
            }
            None
        }
        Statement::Foreach(foreach) => {
            for inner in foreach.body.statements() {
                if let Some(h) = find_type_hint_in_closure_stmt(inner, var_name, cursor_offset) {
                    return Some(h);
                }
            }
            None
        }
        Statement::Block(block) => {
            for inner in block.statements.iter() {
                if let Some(h) = find_type_hint_in_closure_stmt(inner, var_name, cursor_offset) {
                    return Some(h);
                }
            }
            None
        }
        _ => None,
    }
}

/// Recursively search an expression for a closure/arrow function whose
/// parameter the cursor is on.
fn find_type_hint_in_closure_expr(
    expr: &Expression<'_>,
    var_name: &str,
    cursor_offset: u32,
) -> Option<String> {
    let span = expr.span();
    if cursor_offset < span.start.offset || cursor_offset > span.end.offset {
        return None;
    }

    match expr {
        Expression::Closure(closure) => {
            let body_start = closure.body.left_brace.start.offset;
            let body_end = closure.body.right_brace.end.offset;
            if cursor_offset >= body_start && cursor_offset <= body_end {
                // Check nested closures first.
                let body_stmts: Vec<&Statement> = closure.body.statements.iter().collect();
                if let Some(hint) =
                    find_type_hint_in_nested_closure(&body_stmts, var_name, cursor_offset)
                {
                    return Some(hint);
                }
                return find_type_hint_in_params(&closure.parameter_list, var_name, cursor_offset);
            }
            // Check parameter list directly.
            let param_span = closure.parameter_list.span();
            if cursor_offset >= param_span.start.offset && cursor_offset <= param_span.end.offset {
                return find_type_hint_in_params(&closure.parameter_list, var_name, cursor_offset);
            }
            None
        }
        Expression::ArrowFunction(arrow) => {
            let param_span = arrow.parameter_list.span();
            if cursor_offset >= param_span.start.offset && cursor_offset <= param_span.end.offset {
                return find_type_hint_in_params(&arrow.parameter_list, var_name, cursor_offset);
            }
            // Body expression.
            find_type_hint_in_closure_expr(arrow.expression, var_name, cursor_offset)
        }
        Expression::Assignment(assignment) => {
            find_type_hint_in_closure_expr(assignment.rhs, var_name, cursor_offset)
                .or_else(|| find_type_hint_in_closure_expr(assignment.lhs, var_name, cursor_offset))
        }
        Expression::Call(call) => match call {
            Call::Function(func_call) => {
                for arg in func_call.argument_list.arguments.iter() {
                    let arg_expr: &Expression<'_> = arg.value();
                    if let Some(h) =
                        find_type_hint_in_closure_expr(arg_expr, var_name, cursor_offset)
                    {
                        return Some(h);
                    }
                }
                None
            }
            Call::Method(method_call) => {
                for arg in method_call.argument_list.arguments.iter() {
                    let arg_expr: &Expression<'_> = arg.value();
                    if let Some(h) =
                        find_type_hint_in_closure_expr(arg_expr, var_name, cursor_offset)
                    {
                        return Some(h);
                    }
                }
                None
            }
            Call::StaticMethod(static_call) => {
                for arg in static_call.argument_list.arguments.iter() {
                    let arg_expr: &Expression<'_> = arg.value();
                    if let Some(h) =
                        find_type_hint_in_closure_expr(arg_expr, var_name, cursor_offset)
                    {
                        return Some(h);
                    }
                }
                None
            }
            _ => None,
        },
        Expression::Parenthesized(p) => {
            find_type_hint_in_closure_expr(p.expression, var_name, cursor_offset)
        }
        _ => None,
    }
}

/// Extract the type hint string for `var_name` from a parameter list.
fn find_type_hint_in_params(
    params: &FunctionLikeParameterList<'_>,
    var_name: &str,
    _cursor_offset: u32,
) -> Option<String> {
    for param in params.parameters.iter() {
        if param.variable.name == var_name {
            return param.hint.as_ref().map(|h| extract_hint_string(h));
        }
    }
    None
}
