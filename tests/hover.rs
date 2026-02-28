//! Integration tests for `textDocument/hover`.

mod common;

use common::{create_psr4_workspace, create_test_backend, create_test_backend_with_function_stubs};
use phpantom_lsp::Backend;
use tower_lsp::lsp_types::*;

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Register file content in the backend (sync) and return the hover result
/// at the given (0-based) line and character.
fn hover_at(
    backend: &Backend,
    uri: &str,
    content: &str,
    line: u32,
    character: u32,
) -> Option<Hover> {
    // Parse and populate ast_map, use_map, namespace_map, symbol_maps
    backend.update_ast(uri, content);

    backend.handle_hover(uri, content, Position { line, character })
}

/// Extract the Markdown text from a Hover response.
fn hover_text(hover: &Hover) -> &str {
    match &hover.contents {
        HoverContents::Markup(markup) => &markup.value,
        _ => panic!("Expected MarkupContent"),
    }
}

// ─── Variable hover ─────────────────────────────────────────────────────────

#[test]
fn hover_this_variable() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class User {
    public function greet(): string {
        return $this->name();
    }
}
"#;

    // Hover on `$this` (line 3, within the `$this` token)
    let hover = hover_at(&backend, uri, content, 3, 16).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("$this"), "should mention $this: {}", text);
    assert!(text.contains("User"), "should resolve to User: {}", text);
}

#[test]
fn hover_variable_with_type() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Order {
    public string $id;
}
class Service {
    public function run(): void {
        $order = new Order();
        $order->id;
    }
}
"#;

    // Hover on `$order` at line 7 (the usage)
    let hover = hover_at(&backend, uri, content, 7, 9).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("$order"), "should mention $order: {}", text);
    assert!(text.contains("Order"), "should resolve to Order: {}", text);
}

#[test]
fn hover_variable_without_type() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
function test() {
    $x = 42;
    echo $x;
}
"#;

    // Hover on `$x` at line 3
    let hover = hover_at(&backend, uri, content, 3, 10).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("$x"), "should mention $x: {}", text);
}

// ─── Method hover ───────────────────────────────────────────────────────────

#[test]
fn hover_method_call() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Calculator {
    public function add(int $a, int $b): int {
        return $a + $b;
    }
    public function run(): void {
        $this->add(1, 2);
    }
}
"#;

    // Hover on `add` in `$this->add(1, 2)` (line 6)
    let hover = hover_at(&backend, uri, content, 6, 16).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("add"), "should contain method name: {}", text);
    assert!(text.contains("int $a"), "should show params: {}", text);
    assert!(text.contains(": int"), "should show return type: {}", text);
    assert!(
        text.contains("Calculator"),
        "should show owner class: {}",
        text
    );
}

#[test]
fn hover_static_method() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Factory {
    public static function create(string $name): self {
        return new self();
    }
}
class Usage {
    public function run(): void {
        Factory::create('test');
    }
}
"#;

    // Hover on `create` in `Factory::create` (line 8)
    let hover = hover_at(&backend, uri, content, 8, 18).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("create"),
        "should contain method name: {}",
        text
    );
    assert!(text.contains("static"), "should indicate static: {}", text);
    assert!(
        text.contains("string $name"),
        "should show params: {}",
        text
    );
}

// ─── Property hover ─────────────────────────────────────────────────────────

#[test]
fn hover_property_access() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Config {
    public string $name;
    public function show(): void {
        echo $this->name;
    }
}
"#;

    // Hover on `name` in `$this->name` (line 4)
    let hover = hover_at(&backend, uri, content, 4, 21).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("name"),
        "should contain property name: {}",
        text
    );
    assert!(text.contains("string"), "should show type: {}", text);
    assert!(text.contains("Config"), "should show owner: {}", text);
}

#[test]
fn hover_static_property() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Registry {
    public static int $count;
}
class Usage {
    public function run(): void {
        echo Registry::$count;
    }
}
"#;

    // Hover on `$count` in `Registry::$count` (line 6)
    let hover = hover_at(&backend, uri, content, 6, 24).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("count"),
        "should contain property name: {}",
        text
    );
    assert!(text.contains("static"), "should indicate static: {}", text);
    assert!(text.contains("int"), "should show type: {}", text);
}

// ─── Constant hover ─────────────────────────────────────────────────────────

#[test]
fn hover_class_constant() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Status {
    const ACTIVE = 'active';
}
class Usage {
    public function run(): void {
        echo Status::ACTIVE;
    }
}
"#;

    // Hover on `ACTIVE` in `Status::ACTIVE` (line 6)
    let hover = hover_at(&backend, uri, content, 6, 22).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("ACTIVE"),
        "should contain constant name: {}",
        text
    );
    assert!(text.contains("Status"), "should show owner: {}", text);
}

// ─── Class hover ────────────────────────────────────────────────────────────

#[test]
fn hover_class_reference() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Animal {
    public string $species;
}
class Zoo {
    public function adopt(Animal $pet): void {}
}
"#;

    // Hover on `Animal` in the type hint (line 5)
    let hover = hover_at(&backend, uri, content, 5, 28).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("class"), "should show class kind: {}", text);
    assert!(text.contains("Animal"), "should show class name: {}", text);
}

#[test]
fn hover_interface_reference() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
interface Printable {
    public function print(): void;
}
class Document implements Printable {
    public function print(): void {}
}
"#;

    // Hover on `Printable` in the implements clause (line 4)
    let hover = hover_at(&backend, uri, content, 4, 32).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("interface"),
        "should show interface kind: {}",
        text
    );
    assert!(
        text.contains("Printable"),
        "should show interface name: {}",
        text
    );
}

#[test]
fn hover_class_declaration() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
/**
 * Represents a blog post.
 */
class BlogPost {
    public string $title;
}
"#;

    // Hover on `BlogPost` declaration (line 4)
    let hover = hover_at(&backend, uri, content, 4, 8).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("BlogPost"),
        "should show class name: {}",
        text
    );
    assert!(
        text.contains("Represents a blog post"),
        "should include docblock description: {}",
        text
    );
}

#[test]
fn hover_abstract_class() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
abstract class Shape {
    abstract public function area(): float;
}
class Circle extends Shape {
    public function area(): float { return 3.14; }
}
"#;

    // Hover on `Shape` in extends clause (line 4)
    let hover = hover_at(&backend, uri, content, 4, 23).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("abstract class"),
        "should show abstract class: {}",
        text
    );
    assert!(text.contains("Shape"), "should show class name: {}", text);
}

#[test]
fn hover_final_class() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
final class Singleton {
    public static function instance(): self { return new self(); }
}
function test(Singleton $s): void {}
"#;

    // Hover on `Singleton` in function param (line 4)
    let hover = hover_at(&backend, uri, content, 4, 17).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("final class"),
        "should show final class: {}",
        text
    );
}

// ─── Self / static / parent hover ───────────────────────────────────────────

#[test]
fn hover_self_keyword() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Foo {
    public static function make(): self {
        return new self();
    }
}
"#;

    // Hover on `self` at line 3 inside `new self()`
    let hover = hover_at(&backend, uri, content, 3, 20).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("self"), "should mention self: {}", text);
    assert!(text.contains("Foo"), "should resolve to Foo: {}", text);
}

#[test]
fn hover_parent_keyword() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Base {
    public function hello(): string { return 'hi'; }
}
class Child extends Base {
    public function hello(): string {
        return parent::hello();
    }
}
"#;

    // Hover on `parent` at line 6
    let hover = hover_at(&backend, uri, content, 6, 16).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("parent"), "should mention parent: {}", text);
    assert!(text.contains("Base"), "should resolve to Base: {}", text);
}

// ─── Function call hover ────────────────────────────────────────────────────

#[test]
fn hover_user_function() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
function greet(string $name): string {
    return "Hello, $name!";
}
greet('World');
"#;

    // Hover on `greet` at line 4
    let hover = hover_at(&backend, uri, content, 4, 2).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("greet"),
        "should contain function name: {}",
        text
    );
    assert!(
        text.contains("string $name"),
        "should show params: {}",
        text
    );
    assert!(
        text.contains(": string"),
        "should show return type: {}",
        text
    );
}

// ─── Deprecated marker ──────────────────────────────────────────────────────

#[test]
fn hover_deprecated_method() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Legacy {
    /**
     * @deprecated Use newMethod() instead.
     */
    public function oldMethod(): void {}
    public function run(): void {
        $this->oldMethod();
    }
}
"#;

    let hover = hover_at(&backend, uri, content, 7, 16).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("oldMethod"),
        "should contain method name: {}",
        text
    );
    assert!(
        text.contains("@deprecated"),
        "should show deprecated: {}",
        text
    );
}

#[test]
fn hover_deprecated_class() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
/**
 * @deprecated Use NewApi instead.
 */
class OldApi {
    public function run(): void {}
}
function test(OldApi $api): void {}
"#;

    // Hover on OldApi in function param (line 7)
    let hover = hover_at(&backend, uri, content, 7, 17).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("OldApi"), "should show class name: {}", text);
    assert!(
        text.contains("@deprecated"),
        "should show deprecated: {}",
        text
    );
}

// ─── Cross-file hover ───────────────────────────────────────────────────────

#[test]
fn hover_cross_file_class() {
    let (backend, _dir) = create_psr4_workspace(
        r#"{
            "autoload": {
                "psr-4": { "App\\": "src/" }
            }
        }"#,
        &[
            (
                "src/Models/Product.php",
                r#"<?php
namespace App\Models;
/**
 * Represents a product in the catalog.
 */
class Product {
    public string $name;
    public float $price;
    public function discount(float $percent): float {
        return $this->price * (1 - $percent / 100);
    }
}
"#,
            ),
            (
                "src/Service.php",
                r#"<?php
namespace App;
use App\Models\Product;
class Service {
    public function run(): void {
        $p = new Product();
        $p->discount(10);
    }
}
"#,
            ),
        ],
    );

    let product_uri = format!(
        "file://{}",
        _dir.path().join("src/Models/Product.php").display()
    );
    let product_content =
        std::fs::read_to_string(_dir.path().join("src/Models/Product.php")).unwrap();
    backend.update_ast(&product_uri, &product_content);

    let service_uri = format!("file://{}", _dir.path().join("src/Service.php").display());
    let service_content = std::fs::read_to_string(_dir.path().join("src/Service.php")).unwrap();

    // Hover on `Product` type reference (line 5: `$p = new Product()`)
    let hover = hover_at(&backend, &service_uri, &service_content, 5, 20)
        .expect("expected hover on Product");
    let text = hover_text(&hover);
    assert!(
        text.contains("Product"),
        "should resolve cross-file class: {}",
        text
    );
    assert!(
        text.contains("Represents a product"),
        "should include docblock from cross-file class: {}",
        text
    );
}

#[test]
fn hover_cross_file_method() {
    let (backend, _dir) = create_psr4_workspace(
        r#"{
            "autoload": {
                "psr-4": { "App\\": "src/" }
            }
        }"#,
        &[
            (
                "src/Models/Item.php",
                r#"<?php
namespace App\Models;
class Item {
    public function getLabel(): string {
        return 'label';
    }
}
"#,
            ),
            (
                "src/Handler.php",
                r#"<?php
namespace App;
use App\Models\Item;
class Handler {
    public function process(): void {
        $item = new Item();
        $item->getLabel();
    }
}
"#,
            ),
        ],
    );

    let item_uri = format!(
        "file://{}",
        _dir.path().join("src/Models/Item.php").display()
    );
    let item_content = std::fs::read_to_string(_dir.path().join("src/Models/Item.php")).unwrap();
    backend.update_ast(&item_uri, &item_content);

    let handler_uri = format!("file://{}", _dir.path().join("src/Handler.php").display());
    let handler_content = std::fs::read_to_string(_dir.path().join("src/Handler.php")).unwrap();

    // Hover on `getLabel` (line 6)
    let hover = hover_at(&backend, &handler_uri, &handler_content, 6, 16)
        .expect("expected hover on getLabel");
    let text = hover_text(&hover);
    assert!(
        text.contains("getLabel"),
        "should resolve cross-file method: {}",
        text
    );
    assert!(
        text.contains(": string"),
        "should show return type: {}",
        text
    );
    assert!(text.contains("Item"), "should show owner class: {}", text);
}

// ─── Enum hover ─────────────────────────────────────────────────────────────

#[test]
fn hover_enum_declaration() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
/**
 * Possible statuses for an order.
 */
enum OrderStatus: string {
    case Pending = 'pending';
    case Shipped = 'shipped';
}
function process(OrderStatus $status): void {}
"#;

    // Hover on `OrderStatus` in the function param (line 8)
    let hover = hover_at(&backend, uri, content, 8, 20).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("enum"), "should show enum kind: {}", text);
    assert!(
        text.contains("OrderStatus"),
        "should show enum name: {}",
        text
    );
    assert!(
        text.contains("Possible statuses"),
        "should include docblock: {}",
        text
    );
}

// ─── Trait hover ────────────────────────────────────────────────────────────

#[test]
fn hover_trait_reference() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
/**
 * Provides soft-delete functionality.
 */
trait SoftDeletes {
    public function trash(): void {}
}
class Post {
    use SoftDeletes;
}
"#;

    // Hover on `SoftDeletes` in the use statement (line 8)
    let hover = hover_at(&backend, uri, content, 8, 10).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("trait"), "should show trait kind: {}", text);
    assert!(
        text.contains("SoftDeletes"),
        "should show trait name: {}",
        text
    );
    assert!(
        text.contains("Provides soft-delete"),
        "should include docblock: {}",
        text
    );
}

// ─── Visibility display ─────────────────────────────────────────────────────

#[test]
fn hover_shows_visibility() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Vault {
    private string $secret;
    protected int $level;
    public function getSecret(): string {
        echo $this->secret;
        echo $this->level;
        return $this->secret;
    }
}
"#;

    // Hover on `secret` property (line 5)
    let hover = hover_at(&backend, uri, content, 5, 22).expect("expected hover on secret");
    let text = hover_text(&hover);
    assert!(
        text.contains("private"),
        "should show private visibility: {}",
        text
    );

    // Hover on `level` property (line 6)
    let hover = hover_at(&backend, uri, content, 6, 22).expect("expected hover on level");
    let text = hover_text(&hover);
    assert!(
        text.contains("protected"),
        "should show protected visibility: {}",
        text
    );
}

// ─── Inheritance hover ──────────────────────────────────────────────────────

#[test]
fn hover_inherited_method() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class BaseRepo {
    public function findAll(): array {
        return [];
    }
}
class UserRepo extends BaseRepo {
    public function run(): void {
        $this->findAll();
    }
}
"#;

    // Hover on `findAll` in the child class (line 8)
    let hover = hover_at(&backend, uri, content, 8, 16).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("findAll"),
        "should show inherited method: {}",
        text
    );
    assert!(
        text.contains(": array"),
        "should show return type: {}",
        text
    );
}

// ─── Class with parent and implements ───────────────────────────────────────

#[test]
fn hover_class_with_extends_and_implements() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
interface Loggable {
    public function log(): void;
}
class Base {}
class App extends Base implements Loggable {
    public function log(): void {}
}
function test(App $app): void {}
"#;

    // Hover on `App` in the function parameter (line 8)
    let hover = hover_at(&backend, uri, content, 8, 16).expect("expected hover");
    let text = hover_text(&hover);
    assert!(text.contains("class App"), "should show class: {}", text);
    // Parent/interface names may have a leading `\` from the parser
    assert!(
        text.contains("extends") && text.contains("Base"),
        "should show parent: {}",
        text
    );
    assert!(
        text.contains("implements") && text.contains("Loggable"),
        "should show interfaces: {}",
        text
    );
}

// ─── No hover on whitespace ─────────────────────────────────────────────────

#[test]
fn hover_on_whitespace_returns_none() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php

class Foo {}
"#;

    // Hover on the blank line (line 1)
    let hover = hover_at(&backend, uri, content, 1, 0);
    assert!(hover.is_none(), "should not produce hover on blank line");
}

// ─── Stub function hover ────────────────────────────────────────────────────

#[test]
fn hover_stub_function() {
    let backend = create_test_backend_with_function_stubs();
    let uri = "file:///test.php";
    let content = r#"<?php
$x = str_contains('hello', 'ell');
"#;

    // Hover on `str_contains` (line 1)
    let hover = hover_at(&backend, uri, content, 1, 8).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("str_contains"),
        "should show function name: {}",
        text
    );
    assert!(
        text.contains("string $haystack"),
        "should show params: {}",
        text
    );
    assert!(text.contains(": bool"), "should show return type: {}", text);
}

// ─── Namespaced class hover ─────────────────────────────────────────────────

#[test]
fn hover_shows_fqn() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
namespace App\Models;

/**
 * A customer entity.
 */
class Customer {
    public string $email;
}

class Service {
    public function run(): void {
        $c = new Customer();
        $c->email;
    }
}
"#;

    // Hover on Customer reference at line 12
    let hover = hover_at(&backend, uri, content, 12, 18).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("App\\Models\\Customer"),
        "should show FQN: {}",
        text
    );
    assert!(
        text.contains("A customer entity"),
        "should include docblock: {}",
        text
    );
}

// ─── Method with reference and variadic params ──────────────────────────────

#[test]
fn hover_method_with_reference_param() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Sorter {
    public function sort(array &$items): void {}
    public function run(): void {
        $this->sort([]);
    }
}
"#;

    let hover = hover_at(&backend, uri, content, 4, 16).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("&$items"),
        "should show reference param: {}",
        text
    );
}

#[test]
fn hover_method_with_variadic_param() {
    let backend = create_test_backend();
    let uri = "file:///test.php";
    let content = r#"<?php
class Logger {
    public function log(string ...$messages): void {}
    public function run(): void {
        $this->log('a', 'b');
    }
}
"#;

    let hover = hover_at(&backend, uri, content, 4, 16).expect("expected hover");
    let text = hover_text(&hover);
    assert!(
        text.contains("...$messages"),
        "should show variadic param: {}",
        text
    );
}
