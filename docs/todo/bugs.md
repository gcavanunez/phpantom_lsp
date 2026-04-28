# PHPantom — Bug Fixes

Every bug below must be fixed at its root cause. "Detect the
symptom and suppress the diagnostic" is not an acceptable fix.
If the type resolution pipeline produces wrong data, fix the
pipeline so it produces correct data. Downstream consumers
(diagnostics, hover, completion, definition) should never need
to second-guess upstream output.



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
