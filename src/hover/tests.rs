use super::*;

#[test]
fn extract_description_simple() {
    let doc = "/** This is a simple description. */";
    assert_eq!(
        extract_docblock_description(Some(doc)),
        Some("This is a simple description.".to_string())
    );
}

#[test]
fn extract_description_multiline() {
    let doc = "/**\n * First line.\n * Second line.\n * @param string $x\n */";
    assert_eq!(
        extract_docblock_description(Some(doc)),
        Some("First line.\nSecond line.".to_string())
    );
}

#[test]
fn extract_description_none_when_only_tags() {
    let doc = "/**\n * @return string\n */";
    assert_eq!(extract_docblock_description(Some(doc)), None);
}

#[test]
fn extract_description_none_when_empty() {
    assert_eq!(extract_docblock_description(None), None);
}

#[test]
fn format_fqn_with_namespace() {
    assert_eq!(
        format_fqn("User", &Some("App\\Models".to_string())),
        "App\\Models\\User"
    );
}

#[test]
fn format_fqn_without_namespace() {
    assert_eq!(format_fqn("User", &None), "User");
}

#[test]
fn format_params_empty() {
    assert_eq!(format_params(&[]), "");
}

#[test]
fn format_params_with_types() {
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
            is_required: false,
            is_variadic: false,
            is_reference: false,
        },
    ];
    assert_eq!(format_params(&params), "string $name, int $age = ...");
}

#[test]
fn format_params_variadic() {
    let params = vec![ParameterInfo {
        name: "$items".to_string(),
        type_hint: Some("string".to_string()),
        is_required: false,
        is_variadic: true,
        is_reference: false,
    }];
    assert_eq!(format_params(&params), "string ...$items");
}

#[test]
fn format_params_reference() {
    let params = vec![ParameterInfo {
        name: "$arr".to_string(),
        type_hint: Some("array".to_string()),
        is_required: true,
        is_variadic: false,
        is_reference: true,
    }];
    assert_eq!(format_params(&params), "array &$arr");
}

#[test]
fn format_visibility_all() {
    assert_eq!(format_visibility(Visibility::Public), "public ");
    assert_eq!(format_visibility(Visibility::Protected), "protected ");
    assert_eq!(format_visibility(Visibility::Private), "private ");
}
