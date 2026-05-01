# PHPantom — Bug Fixes

Every bug below must be fixed at its root cause. "Detect the
symptom and suppress the diagnostic" is not an acceptable fix.
If the type resolution pipeline produces wrong data, fix the
pipeline so it produces correct data. Downstream consumers
(diagnostics, hover, completion, definition) should never need
to second-guess upstream output.






## B14. Template/generic resolution in multi-namespace test files

**Discovered:** SKIP audit of
`tests/psalm_assertions/template_class_template.php`.

Remaining failures have multiple root causes (the original
multi-namespace theory was incorrect for most of them):

- **Lines 16, 29, 41, 56, 68:** Generic constructor inference
  through iterator decorators (`CachingIterator(new ArrayIterator(...))`)
  does not propagate template parameters. Fails in single-namespace
  files too.
- **Line 602:** Union generic method resolution (`C<A>|C<B>` → `->get()`)
  does not resolve per-branch template substitutions.
- **Line 752:** `new ArrayCollection()` with no args infers
  `ArrayCollection<array, array>` instead of `ArrayCollection<never, never>`.
- **Line 788:** Static method call `Collection::fromClassString(A::class)`
  does not propagate the method-level template to the return type.

**Fixed:** Line 122 — `@var` docblocks with additional tags
(e.g. `@psalm-suppress`) after the type corrupted the type string.
Fixed in `parse_inline_var_docblock_no_var`.

**Tests:** SKIPs in `tests/psalm_assertions/template_class_template.php`
(lines 16, 29, 41, 56, 68, 602, 752, 788).



## B16. PDOStatement fetch mode-dependent return types

**Blocked on:** [phpstorm-stubs#1882](https://github.com/JetBrains/phpstorm-stubs/pull/1882)

`PDOStatement::fetch()` and `PDOStatement::fetchAll()` return
different types depending on the fetch mode constant passed as
the first argument. Once the upstream PR is merged and we update
our stubs, the existing conditional return type support should
handle this automatically.

**Tests:** SKIPs in `tests/psalm_assertions/method_call.php`
(lines 79-85, 87-89).


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
