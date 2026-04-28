# PHPantom — Bug Fixes

Every bug below must be fixed at its root cause. "Detect the
symptom and suppress the diagnostic" is not an acceptable fix.
If the type resolution pipeline produces wrong data, fix the
pipeline so it produces correct data. Downstream consumers
(diagnostics, hover, completion, definition) should never need
to second-guess upstream output.

## B6. Superlinear hover scaling on large single files

**Discovered:** Psalm `ArrayFunctionCallTest.php` porting (Phase 3.5B).

A 1095-line PHP file with 88 `assertType()` calls (each requiring a
hover) takes 126s to process. Splitting into two ~550-line halves takes
11s + 14s = 25s total. The 5x slowdown suggests O(n*m) or worse
behavior where the forward walker or type resolution restarts from the
file beginning for each hover, and/or resolved types are not cached
between hover requests on the same file content.

**Repro:** Run the extraction script on `ArrayFunctionCallTest.php`
and place the output in `tests/psalm_assertions/`, then run the
assert_type_runner.

**Expected:** Processing time should scale roughly linearly with file
size and assertion count.

## B7. `empty()` narrowing resolves to `null` instead of `mixed|null`

**Discovered:** Psalm `TypeReconciliation/EmptyTest.php` porting
(Phase 3.5B).

When `empty($a)` is true for a `mixed` parameter, the variable should
retain its base type intersected with falsy values (`mixed|null`), not
collapse entirely to `null`.

**When fixed:** Create `tests/psalm_assertions/type_reconciliation_empty.php`
with this content and verify it passes:

```php
<?php
// Source: Psalm TypeReconciliation/EmptyTest.php
namespace PsalmTest_type_reconciliation_empty_1 {
    /** @param mixed $a */
    function foo($a): void {
        if (empty($a)) {
            assertType('mixed|null', $a);
        }
    }
}
```

## B8. Binary expression type inference gaps

**Discovered:** SKIP audit across `tests/phpstan_nsrt/binary.php`
and `tests/psalm_assertions/binary_operation.php`.

The hover/forward-walk pipeline does not resolve types for many
binary expressions:

- **String concatenation with int LHS:** `$integer . $string`
  resolves to no type instead of `string`.
- **Spaceship operator:** `'foo' <=> 'bar'` resolves to no type
  instead of `int`.
- **Bitwise on strings:** `"x" & "y"`, `$string & "x"`, `~"a"`
  resolve to `int` instead of `string`. PHP applies bitwise ops
  character-by-character when both operands are strings.
- **Logical operators:** `true && false`, `true || false`,
  `true xor false`, `!true` resolve to no type instead of `bool`.
  `xor` on bools resolves to `int` instead of `bool`.
- **Compound assignment:** `/=`, `*=`, `%=`, `**=`, `<<=`, `&=`,
  `|=`, `^=` resolve to no type instead of the correct result type.
- **Mixed arithmetic:** `1 + $mixed`, `$mixed / 1` resolve to no
  type instead of `float|int`.
- **Exponent:** `4 ** 5` resolves to `int|float` instead of `int`
  (when both operands are non-negative integers).
- **Numeric string increment:** `$a = "123"; $a++` resolves to
  `string` instead of `float|int`.
- **String concat compound:** `$d -= getNumeric()` with numeric
  return resolves to `int` instead of `float|int`.

**Tests:** SKIPs in `tests/phpstan_nsrt/binary.php` (lines 153,
168, 183-188, 218-224, 229-242, 247-250) and
`tests/psalm_assertions/binary_operation.php` (lines 10, 17, 56,
68-69, 87, 100, 165, 175-176).

## B9. `isset()` narrowing not implemented

**Discovered:** SKIP audit of
`tests/phpstan_nsrt/isset-narrowing.php`.

`isset($x)` in a condition should strip `null` from the variable's
type (like `$x !== null`). Currently the variable retains its
nullable type through the truthy branch. This affects 14 assertions
across property access, array access, and simple variable patterns.

**Tests:** All 18 SKIPs in `tests/phpstan_nsrt/isset-narrowing.php`.

**When fixed:** Remove the SKIPs and verify
`cargo nextest run "run_assert_type::isset-narrowing"` passes.

## B10. First-class callable invocation return types

**Discovered:** SKIP audit of
`tests/phpstan_nsrt/static-late-binding.php`.

`Foo::method(...)` creates a `Closure` from a method, and
`Foo::method(...)()` immediately invokes it. The return type of the
invocation should match the method's return type. Currently hover
returns no type for these expressions.

This also affects `static::method(...)()`, `self::method(...)()`,
`parent::method(...)()`, and `$this->method(...)()`.

**Tests:** SKIPs in `tests/phpstan_nsrt/static-late-binding.php`
(lines 72-78, 88, 90-97).

## B11. Unbound template parameter resolves to raw name instead of `mixed`

**Discovered:** SKIP audit of
`tests/phpstan_nsrt/generic-traits.php`.

When a class uses a trait with `@template T` but does not provide a
concrete type via `@use`, the template parameter `T` should resolve
to its bound (or `mixed` if unbounded). Currently it resolves to
the raw name `T`.

**Test:** `tests/phpstan_nsrt/generic-traits.php` line 78.

## Bulk un-SKIP after fixes

There are `// SKIP` markers across `tests/phpstan_nsrt/*.php` and
`tests/psalm_assertions/*.php` covering gaps in the type engine.
When working on any type engine improvement, grep for `// SKIP` in
the assertion files to find tests that may now pass. Run
`cargo nextest run --test assert_type_runner --no-fail-fast` with
the SKIP removed to verify.

Some SKIPs are **out of scope** for an LSP (value-range tracking,
int overflow detection, constant-expression folding, `*NEVER*`
after impossible conditions, `*ERROR*` diagnostics). These should
just be removed from the test files.
