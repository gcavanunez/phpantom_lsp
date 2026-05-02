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

- **Lines 16, 29, 41, 56, 68:** SPL iterator stubs
  (`CachingIterator`, `InfiniteIterator`, `LimitIterator`,
  `CallbackFilterIterator`, `NoRewindIterator`) lack `@template`
  annotations, so generic type propagation through iterator
  decorator constructors is impossible with current stubs.
- **Lines 602, 788:** Union generic method resolution and static
  method template inference work correctly in single-namespace
  files. The failures are caused by the multi-namespace test
  runner not resolving short class names to FQN across namespace
  blocks in the same file.

**Fixed:**
- Line 122 — `@var` docblocks with additional tags
  (e.g. `@psalm-suppress`) after the type corrupted the type
  string. Fixed in `parse_inline_var_docblock_no_var`.
- Line 752 — `new ArrayCollection([])` inferred
  `ArrayCollection<array, array>` instead of
  `ArrayCollection<never, never>`. Root cause: when both `@param`
  and `@psalm-param` existed for the same parameter,
  `extract_param_raw_type_from_info` returned the first match in
  document order instead of respecting `@phpstan-param` >
  `@psalm-param` > `@param` priority.

**Tests:** SKIPs in `tests/psalm_assertions/template_class_template.php`
(lines 16, 29, 41, 56, 68, 602, 788).



## B16. PDOStatement fetch mode-dependent return types

**Blocked on:** [phpstorm-stubs#1882](https://github.com/JetBrains/phpstorm-stubs/pull/1882)

`PDOStatement::fetch()` and `PDOStatement::fetchAll()` return
different types depending on the fetch mode constant passed as
the first argument. Once the upstream PR is merged and we update
our stubs, the existing conditional return type support should
handle this automatically.

**Tests:** Assertion lines were removed from
`tests/psalm_assertions/method_call.php` (out of scope until
upstream stubs land).


## Bulk un-SKIP after fixes

There are `// SKIP` markers across `tests/psalm_assertions/*.php`
covering gaps in the type engine. When working on any type engine
improvement, grep for `// SKIP` in the assertion files to find
tests that may now pass. Run
`cargo nextest run --test assert_type_runner --no-fail-fast` with
the SKIP removed to verify.

Remaining SKIPs (11) are:
- `template_class_template.php` (7) — B14: SPL stubs lack
  @template annotations (5), multi-namespace test runner
  limitation (2)
- `magic_method_annotation.php` (3) — B14 cross-namespace
  resolution in single-file test runner
- `mixin_annotation.php` (1) — `IteratorIterator` not in fixture
  runner stubs (feature works with full stubs)
