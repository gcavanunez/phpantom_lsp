# Changelog

All notable changes to PHPantom will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Docblock navigation.** Go-to-definition and hover now work on class names inside callable type annotations (`\Closure(Request): Response`), and Ctrl+Click on object shape properties (`$profile->name` from `@return object{name: string}`) jumps to the key inside the docblock.

### Changed

- **Named-argument completion uses symbol map.** Detection now uses precomputed `CallSite` data from the AST-based symbol map as the primary path, falling back to text scanning for broken ASTs. This correctly handles chain-resolution on `$this->prop->method(`, `$obj->first()->second(`, `(new Foo())->bar(`, and strings containing parentheses.
- **Member-access completion uses symbol map.** `try_member_access_completion` now consults the symbol map's `MemberAccess` entries before falling back to text-based subject extraction. This improves accuracy for `(new Foo)->`, call-result chains, array access chains, and null-safe chains when the parser successfully recovers the AST.

### Fixed

- **Named-argument resolution for non-variable subjects.** `resolve_named_arg_params` now routes all instance-method subjects (bare class names, chain results) through `resolve_target_classes` instead of returning empty results when the subject does not start with `$`. The same fix is applied to static-method resolution, which now falls back to variable/chain resolution when the class name is not found directly.
- **GTD for `@method`/`@property` on interfaces.** Go-to-definition now walks implemented interfaces (own and from parents) before checking `@mixin` classes, so virtual members declared on interfaces resolve correctly.
- **`?->` null-safe chain resolution.** The `rfind("->")` split incorrectly matched the `->` inside `?->`, leaving a trailing `?` on the left-hand side. Fixed at all seven call sites across resolver, text resolution, handler, foreach resolution, and signature help.
- **`(new Canvas())->easel` property access.** Parenthesized `new` expressions on the left side of `->` now resolve correctly for variable type inference.
- **Array function resolution.** `array_pop`, `array_filter`, `array_values`, `end`, and `array_map` now resolve element types correctly for go-to-definition and completion when the array comes from a method call chain or a `$var->prop` access.

## [0.4.0] - 2026-03-01

### Added

- **Signature help.** Parameter hints appear when typing inside function/method call parentheses. The active parameter highlights and updates as you type commas. Supports all call forms including constructors, static/instance methods, `self::`, `static::`, `parent::`, and cross-file resolution.
- **Hover.** Hovering over a symbol shows its type, signature, and docblock in a Markdown popup. Supports variables, methods, properties, constants, classes, functions, and keywords like `$this`/`self`/`static`/`parent`.
- **`in_array` strict-mode type narrowing.** `in_array($var, $haystack, true)` narrows `$var` to the haystack's element type. Works in `if`/`else`, `while`, guard clauses, and negated conditions.
- **Closure and arrow-function member access.** Variables holding closures, arrow functions, and first-class callables resolve to `Closure`, enabling `->bindTo()`, `->call()`, etc.
- **Closure parameter inference.** Untyped closure parameters are inferred from the callable signature of the receiving function (e.g. `$users->map(fn($u) => $u->name)` infers `$u` as `User`). Template parameters in callable signatures are substituted to concrete types.
- **First-class callable syntax.** `strlen(...)`, `$obj->method(...)`, and `Class::method(...)` resolve as `Closure`, and invoking the result resolves to the underlying return type.
- **Laravel Eloquent support:**
  - **Relationship properties.** Methods returning relationship types (e.g. `HasMany<Post>`) produce virtual properties with correct collection/model types. Supports all 10 relationship types via `@return` annotations or body-pattern inference (`$this->hasMany(Post::class)`).
  - **Relationship count properties.** `*_count` virtual properties (e.g. `$posts_count` as `int`) for each relationship method, matching `withCount`/`loadCount` behaviour.
  - **Scope methods.** `scopeActive()` produces an `active()` virtual method on both the Model and Builder. The `#[Scope]` attribute (Laravel 11+) is also supported. Scope methods chain correctly through Builder instances.
  - **Builder-as-static forwarding.** `User::where(...)->orderBy(...)->get()` resolves end-to-end by forwarding static calls on Models to the Eloquent Builder, with correct generic substitution.
  - **Factory support.** `User::factory()->create()` resolves to the model class via `@use HasFactory<UserFactory>` or naming convention fallback.
  - **Custom Eloquent collections.** Models with `#[CollectedBy]` or `@use HasCollection<X>` resolve to the custom collection class. Also detects `newCollection()` return type overrides.
  - **Cast properties.** `$casts` array and `casts()` method entries produce typed virtual properties (e.g. `datetime` → `Carbon`, enum casts → enum class, custom cast classes → `get()` return type).
  - **Accessor and mutator properties.** Legacy `getXAttribute()` and modern `Attribute`-returning accessors produce snake_case virtual properties.
  - **`$attributes` default properties.** Entries in the `$attributes` array produce typed virtual properties inferred from literal values (skipped when a cast exists).
  - **`$visible` array extraction.** Columns in `$visible` produce `mixed`-typed virtual properties when not covered by other sources.
- **Virtual member provider pipeline.** Priority-ordered pipeline for synthesizing members from `@method`/`@property`, `@mixin`, and framework-specific patterns. `@method`/`@property` tags and `@mixin` members are handled by a single PHPDoc provider; raw docblocks are parsed lazily.
- **Union completion sorting.** Members shared by all union types sort above branch-only members. Branch-only members show their originating class name.
- **Context-aware class name filtering.** `extends` only offers non-final classes (or interfaces in interface context), `implements` offers interfaces only, `use` inside a class body offers traits only. Built-in classes are filtered via a lightweight source scanner.
- **Multi-line method chain completion.** Fluent chains spanning multiple lines resolve correctly for completion and go-to-definition.
- **Template parameter bound resolution.** When a type is a `@template` parameter, the resolver falls back to the upper bound declared via `of`.
- **Transitive interface inheritance in go-to-implementation.** Go-to-implementation on an interface finds classes implementing it transitively through arbitrary depth.
- **Switch statement variable type tracking.** Variables assigned in `switch` case bodies resolve correctly in both brace and colon-delimited forms.
- **`unset()` variable tracking.** After `unset($var)`, the variable no longer appears in suggestions. Re-assignment restores it.
- **Class-string variable forwarding to conditional return types.** Variables holding `::class` values from `match`/ternary/assignment produce union return types when passed to `class-string<T>` → `T` methods.
- **Trait property `@var` docblock resolution.** Chaining on `$this->prop` resolves to the docblock type declared in a trait.
- **Anonymous class completion and go-to-definition.** `$this->` inside anonymous classes resolves to the anonymous class's own members, with full support for `extends`, `implements`, traits, and promoted properties.
- **Alphabetical `use` statement insertion.** Auto-imported `use` statements are inserted at the correct alphabetical position.
- **Namespace segment completion.** Typing `App\` shows next-level namespace segments as navigable items alongside matching classes.
- **String-aware completion suppression.** Completion is suppressed inside string literals but allowed in interpolation contexts.
- **Nested key completion for literal arrays.** `$cfg['db']['` offers nested keys from literal array assignments at arbitrary depth.
- **Generator yield type inference.** Inside generator bodies, `yield $var` infers TValue and `$var = yield` infers TSend from the `@return Generator<...>` annotation.
- **Conditional return types with template substitution.** Methods like `findOrFail` with conditional return types referencing `TModel` resolve correctly through Builder forwarding. Array argument detection selects the correct branch.

### Fixed

- **`@var` docblock variable names in suggestions.** `/** @var Type $varName */` names now appear in `$`-triggered completions.
- **`@var` without variable name for array access.** `/** @var array<int, Customer> */` followed by `$thing = []` resolves element types.
- **Inferred element type from array literals.** `$thing = [new Customer()]; $thing[0]->` resolves to `Customer`.
- **Inline array literal with index access.** `[Customer::first()][0]->` resolves to the element type.
- **`$this` in callable parameter types resolves to receiver class.** `Builder::when(true, function ($query) { ... })` infers `$query` as Builder, not the calling class.
- **Inline array element function calls.** `end($customers)->` resolves element types inline, not just when assigned to a variable.
- **Callable parameter type parsing in union types.** Callable signatures wrapped in unions like `(Closure(Builder<TModel>): mixed)|null` parse correctly.
- **Blank lines inside method chains.** Blank lines between chain segments no longer break resolution.
- **Arrow function parameter go-to-definition.** Clicking `$o` in `fn(Order $o) => $o->getItems()` jumps to the parameter, not an unrelated earlier variable.
- **Relationship property collection type.** `$product->reviews` uses the related model's custom collection, not the owning model's.
- **Protected members hidden from unrelated classes.** Protected members only appear in completions from the same class or subclasses.
- **UTF-8 boundary panic.** Multi-byte UTF-8 characters no longer cause panics in method body scanning.
- **Static call chain property access.** `User::where('active', 1)->first()->profile->` resolves through the entire chain instead of stopping at `::`.
- **Namespaced functions.** Functions in namespaces insert correct `use function FQN;` imports instead of bare names.
- **Docblock annotation scope leak.** `@param`/`@var` annotations no longer leak across sibling methods sharing the same parameter/variable name.
- **Variable property access in text-based resolution.** `$addr = $user->address; $addr->` resolves correctly in the fallback path.
- **Parameter type resolution scoped to correct method.** Same-named parameters in different methods resolve to the type for the method the cursor is in.
- **Array element access from method return types.** `$items[0]->` resolves when `$items` comes from a method returning `User[]` or `array<int, User>`.
- **Go-to-definition for static properties and typed constants.** `ClassName::$staticProp` and PHP 8.3 typed constants resolve correctly.
- **`static` return type resolved to concrete class.** `@return static` on a parent method returns the caller's subclass type.
- **Namespaced FQN return types.** Leading `\` is preserved so FQN return types don't get the caller's namespace prepended.
- **Parenthesized RHS expressions.** `$var = (new Foo())` and `$var = ($cond ? $a : $b)` resolve correctly.
- **`$var::` completion for class-string variables.** `$cls = User::class; $cls::` offers static members.
- **Go-to-definition for inherited members with same short name.** Child and parent classes with the same short name in different namespaces resolve correctly.
- **Go-to-definition on foreach variable.** Clicking `$b` in `foreach ($items as $b)` no longer jumps to a previous loop.
- **Multi-extends interfaces.** Interfaces extending multiple parents store all parent names.
- **Variable resolution inside trait method bodies.** Assignments and parameter types inside trait methods resolve correctly.
- **Go-to-definition on RHS variable.** In `$value = $value->value`, clicking the RHS `$value` jumps to the parameter declaration.
- **Conflicting use-import resolution.** When a short name collides with an existing import, a fully-qualified `\App\Exception` is inserted instead of a duplicate `use` statement.
- **Interface `@method`/`@property` tags on implementing classes.** Virtual members declared on interfaces now appear in completions on implementing classes.
- **Multi-namespace file class resolution.** Files with multiple `namespace { }` blocks resolve classes against the correct per-block namespace.
- **`$this`/`static`/`self` return type on trait methods.** These return types resolve to the using class, not the trait.
- **Mixin `$this`/`self`/`static` return types.** Left as-is instead of being rewritten to the mixin class name, fixing fluent API chains.
- **Go-to-definition member position scoped to declaring class.** In files with multiple classes defining the same method, the correct class is targeted.

## [0.3.0] - 2026-02-21

### Added

- **Go-to-implementation.** Jump from an interface or abstract class (or a method call typed as one) to all concrete implementations. Scans open files, class index, classmap, embedded stubs, and PSR-4 directories in five phases.
- **Method-level `@template` (general case).** When a method declares `@template T` with `@param T $model` and `@return Collection<T>`, the resolver infers `T` from the actual argument at the call site. Works with inline chains, static methods, `new` expressions, and cross-file resolution.
- **`@phpstan-type` / `@psalm-type` aliases.** Local type aliases defined on a class are expanded during resolution, including `@phpstan-import-type` for importing aliases from other classes.
- **Array function type preservation.** `array_filter`, `array_map`, `array_pop`, `current`, and similar functions preserve the element type instead of losing it to `array`.
- **Spread operator type tracking.** `$all = [...$users, ...$admins]` resolves to the union of element types from all spread sources.
- **Callable/closure variable invocation.** `$fn()->` resolves the return type when `$fn` holds a closure, arrow function, or a variable annotated as `Closure(...): T`.
- **Early return narrowing.** Guard clauses (`if (!$x instanceof Foo) return;`) narrow the type for subsequent code. Multiple guards stack. Works in ternaries and `match(true)`.
- **`instanceof` narrowing to interface and abstract types.**
- **`instanceof` narrowing inside ternary expressions.**
- **Trait `insteadof` / `as` conflict resolution.** Visibility changes and method aliasing via `as`, exclusion via `insteadof`.
- **Generics tracked through loop iterations.**
- **Yield type resolution.**
- **Chained method calls in variable assignment.** `$x = $this->foo()->bar()` resolves through the full chain.
- **Named key destructuring from array shapes.** `['name' => $name] = $shape` resolves `$name` to the correct type.
- **Type hint completion in function/method signatures.**
- **Variable and clone assignment type tracking.**
- **Iterate directly on function return values.** `foreach (getUsers() as $user)` resolves `$user`.
- **User constant completion.**
- **Required argument completion.**
- **Contextual try-catch completion.** Exception suggestions are scoped to what the `try` block can actually throw.
- **Void detection for `@return` PHPDoc suggestions.**

### Fixed

- More robust PHPDoc type parsing.
- Fixed false positive type lookups for internal stubs.
- Fixed crash in variable resolver.
- Fixed incorrect method resolution.
- Fixed finding definitions inside comments.
- Fixed use of incorrect import map for name resolution.
- Fixed completion suggestions being too aggressive or appearing in comments.

## [0.2.0] - 2026-02-18

### Added

- **Generics.** Class-level `@template` with `@extends` substitution through inheritance chains. Method-level `class-string<T>` pattern. Generic trait substitution.
- **Array shapes.** `['key' => Type]` literals offer key completion with no annotation needed. Incremental assignments extend the shape.
- **Object shapes.**
- **Array growth tracking.** `$arr[] = new Foo()` and `$arr['key'] = $value` build up the shape incrementally.
- **Array destructuring.** `[$a, $b] = $pair` resolves element types.
- **Array element access.** `$arr[0]->` resolves the element type.
- **Foreach key type resolution.** Keys from generic iterables and array shapes.
- **Iterable value type resolution.** Foreach on `Collection<User>`, `Generator<int, Item>`, and `@implements IteratorAggregate<int, User>`.
- **Ternary and null-coalescing type resolution.**
- **Match expression type inference.**
- **Named argument completion.**
- **Variable name suggestions.**
- **Standalone function completion.**
- **`define()` constant completion.**
- **Smart PHPDoc tag completion.** Tags filtered to context (`@var` only in property docblocks, `@param` only when there are undocumented parameters). `@throws` detects uncaught exceptions. `@param` pre-fills name and type. Already-documented tags are not suggested again.
- **Deprecated member detection.**
- **Promoted property type via `@param`.**
- **Property chaining.**
- **`require_once` function discovery.**
- **Go-to type definition from property.**

### Fixed

- Fixed `@mixin` context for return types.
- Fixed import of global classes and namespace context.
- Fixed go-to-definition for aliased classes.

## [0.1.0] - 2026-02-16

Initial release.

### Added

- **Completion.** Methods, properties, and constants via `->`, `?->`, and `::`. Context-aware visibility filtering. Alphabetically ordered results.
- **Class inheritance.** Parent classes, interfaces, and traits with correct member merging.
- **`self::`, `static::`, `parent::` resolution.**
- **PHPDoc support.** `@return` type resolution, `@property` virtual properties, `@method` virtual methods, `@mixin` class merging.
- **Conditional return types.** `@return ($param is class-string<T> ? T : mixed)` and similar PHPStan-style conditional types.
- **Inline `@var` annotations.** `/** @var User $user */` resolves the variable type.
- **Enum support.** Case completion, implicit `UnitEnum`/`BackedEnum` interface members.
- **Type narrowing.** `instanceof`, `is_a()`, and `@phpstan-assert` annotations.
- **Nullsafe operator.** `?->` completion.
- **Class name completion with auto-import.** Suggests class names and inserts the `use` statement.
- **Union type inference.**
- **Go-to-definition.** Classes, interfaces, traits, enums, methods, properties, constants, standalone functions, `new` expressions, and variable assignments.
- **PSR-4 lazy loading** via Composer.
- **Composer classmap support.**
- **Embedded phpstorm-stubs.** Standard library type information bundled in the binary.
- **Namespace aliasing and prefix imports.**
- **Zed editor extension.**

[Unreleased]: https://github.com/AJenbo/phpantom_lsp/compare/0.4.0...HEAD
[0.4.0]: https://github.com/AJenbo/phpantom_lsp/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/AJenbo/phpantom_lsp/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/AJenbo/phpantom_lsp/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/AJenbo/phpantom_lsp/commits/0.1.0