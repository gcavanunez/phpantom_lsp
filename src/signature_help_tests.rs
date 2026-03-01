use super::*;

// ── detect_call_site_text_fallback ──────────────────────────────

#[test]
fn detect_simple_function_call() {
    let content = "<?php\nfoo(";
    let pos = Position {
        line: 1,
        character: 4,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn detect_second_parameter() {
    let content = "<?php\nfoo($a, ";
    let pos = Position {
        line: 1,
        character: 8,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 1);
}

#[test]
fn detect_third_parameter() {
    let content = "<?php\nfoo($a, $b, ";
    let pos = Position {
        line: 1,
        character: 13,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 2);
}

#[test]
fn detect_method_call() {
    let content = "<?php\n$obj->bar(";
    let pos = Position {
        line: 1,
        character: 10,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "$obj->bar");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn detect_static_method_call() {
    let content = "<?php\nFoo::bar(";
    let pos = Position {
        line: 1,
        character: 9,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "Foo::bar");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn detect_constructor_call() {
    let content = "<?php\nnew Foo(";
    let pos = Position {
        line: 1,
        character: 8,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "new Foo");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn detect_none_outside_parens() {
    let content = "<?php\nfoo();";
    let pos = Position {
        line: 1,
        character: 6,
    };
    assert!(detect_call_site_text_fallback(content, pos).is_none());
}

#[test]
fn detect_nested_call_inner() {
    // Cursor inside inner call
    let content = "<?php\nfoo(bar(";
    let pos = Position {
        line: 1,
        character: 8,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "bar");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn detect_with_string_containing_comma() {
    let content = "<?php\nfoo('a,b', ";
    let pos = Position {
        line: 1,
        character: 12,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 1);
}

#[test]
fn detect_with_nested_parens_containing_comma() {
    let content = "<?php\nfoo(bar(1, 2), ";
    let pos = Position {
        line: 1,
        character: 16,
    };
    let site = detect_call_site_text_fallback(content, pos).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 1);
}

// ── count_top_level_commas ──────────────────────────────────────

#[test]
fn count_commas_empty() {
    let chars: Vec<char> = "()".chars().collect();
    assert_eq!(count_top_level_commas(&chars, 1, 1), 0);
}

#[test]
fn count_commas_two() {
    let chars: Vec<char> = "($a, $b, $c)".chars().collect();
    assert_eq!(count_top_level_commas(&chars, 1, 11), 2);
}

#[test]
fn count_commas_nested() {
    let chars: Vec<char> = "(foo(1, 2), $b)".chars().collect();
    assert_eq!(count_top_level_commas(&chars, 1, 14), 1);
}

#[test]
fn count_commas_in_string() {
    let chars: Vec<char> = "('a,b', $c)".chars().collect();
    assert_eq!(count_top_level_commas(&chars, 1, 10), 1);
}

// ── format_param_label ──────────────────────────────────────────

#[test]
fn format_param_simple() {
    let p = ParameterInfo {
        name: "$x".to_string(),
        type_hint: Some("int".to_string()),
        is_required: true,
        is_variadic: false,
        is_reference: false,
    };
    assert_eq!(format_param_label(&p), "int $x");
}

#[test]
fn format_param_variadic() {
    let p = ParameterInfo {
        name: "$items".to_string(),
        type_hint: Some("string".to_string()),
        is_required: false,
        is_variadic: true,
        is_reference: false,
    };
    assert_eq!(format_param_label(&p), "string ...$items");
}

#[test]
fn format_param_reference() {
    let p = ParameterInfo {
        name: "$arr".to_string(),
        type_hint: Some("array".to_string()),
        is_required: true,
        is_variadic: false,
        is_reference: true,
    };
    assert_eq!(format_param_label(&p), "array &$arr");
}

#[test]
fn format_param_no_type() {
    let p = ParameterInfo {
        name: "$x".to_string(),
        type_hint: None,
        is_required: true,
        is_variadic: false,
        is_reference: false,
    };
    assert_eq!(format_param_label(&p), "$x");
}

// ── build_signature ─────────────────────────────────────────────

#[test]
fn build_signature_label() {
    let params = vec![
        ParameterInfo {
            name: "$name".to_string(),
            type_hint: Some("string".to_string()),
            is_required: true,
            is_variadic: false,
            is_reference: false,
        },
        ParameterInfo {
            name: "$age".to_string(),
            type_hint: Some("int".to_string()),
            is_required: true,
            is_variadic: false,
            is_reference: false,
        },
    ];
    let sig = build_signature("greet", &params, Some("void"));
    assert_eq!(sig.label, "greet(string $name, int $age): void");
}

#[test]
fn build_signature_parameter_offsets() {
    let params = vec![
        ParameterInfo {
            name: "$a".to_string(),
            type_hint: None,
            is_required: true,
            is_variadic: false,
            is_reference: false,
        },
        ParameterInfo {
            name: "$b".to_string(),
            type_hint: None,
            is_required: true,
            is_variadic: false,
            is_reference: false,
        },
    ];
    let sig = build_signature("f", &params, None);
    // label: "f($a, $b)"
    //         0123456789
    let pi = sig.parameters.unwrap();
    assert_eq!(pi[0].label, ParameterLabel::LabelOffsets([2, 4])); // "$a"
    assert_eq!(pi[1].label, ParameterLabel::LabelOffsets([6, 8])); // "$b"
}

#[test]
fn build_signature_no_params() {
    let sig = build_signature("foo", &[], Some("void"));
    assert_eq!(sig.label, "foo(): void");
    assert!(sig.parameters.unwrap().is_empty());
}

// ── clamp_active_param ──────────────────────────────────────────

#[test]
fn clamp_within_range() {
    let params = vec![
        ParameterInfo {
            name: "$a".to_string(),
            type_hint: None,
            is_required: true,
            is_variadic: false,
            is_reference: false,
        },
        ParameterInfo {
            name: "$b".to_string(),
            type_hint: None,
            is_required: true,
            is_variadic: false,
            is_reference: false,
        },
    ];
    assert_eq!(clamp_active_param(0, &params), 0);
    assert_eq!(clamp_active_param(1, &params), 1);
}

#[test]
fn clamp_exceeds_range() {
    let params = vec![ParameterInfo {
        name: "$a".to_string(),
        type_hint: None,
        is_required: true,
        is_variadic: false,
        is_reference: false,
    }];
    assert_eq!(clamp_active_param(5, &params), 0);
}

#[test]
fn clamp_empty_params() {
    assert_eq!(clamp_active_param(0, &[]), 0);
}

// ── detect_call_site_from_map ───────────────────────────────────

/// Helper: parse PHP source and build a SymbolMap, then call
/// `detect_call_site_from_map` at the given line/character.
fn map_detect(content: &str, line: u32, character: u32) -> Option<CallSiteContext> {
    use bumpalo::Bump;
    use mago_database::file::FileId;

    let arena = Bump::new();
    let file_id = FileId::new("test.php");
    let program = mago_syntax::parser::parse_file_content(&arena, file_id, content);
    let sm = crate::symbol_map::extract_symbol_map(program, content);
    let pos = Position { line, character };
    detect_call_site_from_map(&sm, content, pos)
}

#[test]
fn map_simple_function_call() {
    // "foo($a, );" — cursor on the space before `)`, after the comma
    //  f o o ( $ a ,   ) ;
    //  0 1 2 3 4 5 6 7 8 9   (col on line 1)
    let content = "<?php\nfoo($a, );";
    let site = map_detect(content, 1, 7).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 1);
}

#[test]
fn map_function_call_first_param() {
    // "foo($a);" — cursor on `$` inside parens
    //  f o o ( $ a ) ;
    //  0 1 2 3 4 5 6 7
    let content = "<?php\nfoo($a);";
    let site = map_detect(content, 1, 5).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_method_call() {
    // "$obj->bar($x);" — cursor on `$x` inside parens
    //  $ o b j - > b a r (  $  x  )  ;
    //  0 1 2 3 4 5 6 7 8 9 10 11 12 13
    let content = "<?php\n$obj->bar($x);";
    let site = map_detect(content, 1, 11).unwrap();
    assert_eq!(site.call_expression, "$obj->bar");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_property_chain_method_call() {
    // "$this->prop->method($x);" — cursor on `$x` inside method parens
    //  $ t h i s - > p r o  p  -  >  m  e  t  h  o  d  (  $  x  )  ;
    //  0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23
    let content = "<?php\n$this->prop->method($x);";
    let site = map_detect(content, 1, 22).unwrap();
    assert_eq!(site.call_expression, "$this->prop->method");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_chained_method_result() {
    // "$obj->first()->second($x);" — cursor inside second()'s parens
    //  $ o b j - > f i r s  t  (  )  -  >  s  e  c  o  n  d  (  $  x  )  ;
    //  0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25
    let content = "<?php\n$obj->first()->second($x);";
    let site = map_detect(content, 1, 24).unwrap();
    assert_eq!(site.call_expression, "$obj->first()->second");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_static_method_call() {
    // "Foo::bar($x);" — cursor on `$x` inside parens
    //  F o o : : b a r (  $  x  )  ;
    //  0 1 2 3 4 5 6 7 8  9 10 11 12
    let content = "<?php\nFoo::bar($x);";
    let site = map_detect(content, 1, 10).unwrap();
    assert_eq!(site.call_expression, "Foo::bar");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_constructor_call() {
    // "new Foo($x);" — cursor on `$x` inside parens
    //  n e w   F o o (  $  x  )  ;
    //  0 1 2 3 4 5 6 7  8  9 10 11
    let content = "<?php\nnew Foo($x);";
    let site = map_detect(content, 1, 9).unwrap();
    assert_eq!(site.call_expression, "new Foo");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_nested_call_inner() {
    // "foo(bar($x));" — cursor inside bar()'s parens on `$x`
    //  f o o ( b a r (  $  x  )  )  ;
    //  0 1 2 3 4 5 6 7  8  9 10 11 12
    let content = "<?php\nfoo(bar($x));";
    let site = map_detect(content, 1, 9).unwrap();
    assert_eq!(site.call_expression, "bar");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_nested_call_outer() {
    // "foo(bar($x), $y);" — cursor on `$y` in foo()'s second arg
    //  f o o ( b a r (  $  x  )  ,     $  y  )  ;
    //  0 1 2 3 4 5 6 7  8  9 10 11 12 13 14 15 16
    let content = "<?php\nfoo(bar($x), $y);";
    let site = map_detect(content, 1, 14).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 1);
}

#[test]
fn map_string_with_commas() {
    // "foo('a,b', $x);" — comma inside string not counted
    //  f o o ( '  a  ,  b  '  ,     $  x  )  ;
    //  0 1 2 3 4  5  6  7  8  9 10 11 12 13 14
    let content = "<?php\nfoo('a,b', $x);";
    let site = map_detect(content, 1, 11).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 1);
}

#[test]
fn map_nullsafe_method_call() {
    // "$obj?->format($x);" — cursor on `$x` inside parens
    //  $ o b j ?  -  >  f  o  r  m  a  t  (  $  x  )  ;
    //  0 1 2 3 4  5  6  7  8  9 10 11 12 13 14 15 16 17
    let content = "<?php\n$obj?->format($x);";
    let site = map_detect(content, 1, 15).unwrap();
    assert_eq!(site.call_expression, "$obj->format");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_new_expression_chain() {
    // "(new Foo())->method($x);" — cursor on `$x`
    //  (  n  e  w     F  o  o  (  )  )  -  >  m  e  t  h  o  d  (  $  x  )  ;
    //  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23
    let content = "<?php\n(new Foo())->method($x);";
    let site = map_detect(content, 1, 21).unwrap();
    assert_eq!(site.call_expression, "Foo->method");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_none_outside_parens() {
    // "foo();" — cursor on `;` after closing paren
    //  f o o ( ) ;
    //  0 1 2 3 4 5
    let content = "<?php\nfoo();";
    assert!(map_detect(content, 1, 5).is_none());
}

#[test]
fn map_deep_property_chain() {
    // "$a->b->c->d($x);" — cursor on `$x` inside d()'s parens
    //  $ a - > b -  >  c  -  >  d  (  $  x  )  ;
    //  0 1 2 3 4 5  6  7  8  9 10 11 12 13 14 15
    let content = "<?php\n$a->b->c->d($x);";
    let site = map_detect(content, 1, 13).unwrap();
    assert_eq!(site.call_expression, "$a->b->c->d");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_function_return_chain() {
    // "app()->make($x);" — cursor on `$x` inside make()'s parens
    //  a p p (  )  -  >  m  a  k  e  (  $  x  )  ;
    //  0 1 2 3  4  5  6  7  8  9 10 11 12 13 14 15
    let content = "<?php\napp()->make($x);";
    let site = map_detect(content, 1, 13).unwrap();
    assert_eq!(site.call_expression, "app()->make");
    assert_eq!(site.active_parameter, 0);
}

#[test]
fn map_third_parameter() {
    // "foo($a, $b, $c);" — cursor on `$c` after two commas
    //  f o o ( $  a  ,     $  b  ,     $  c  )  ;
    //  0 1 2 3 4  5  6  7  8  9 10 11 12 13 14 15
    let content = "<?php\nfoo($a, $b, $c);";
    let site = map_detect(content, 1, 13).unwrap();
    assert_eq!(site.call_expression, "foo");
    assert_eq!(site.active_parameter, 2);
}
