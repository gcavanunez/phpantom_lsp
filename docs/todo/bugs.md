# PHPantom — Bug Fixes

Known bugs and incorrect behaviour. These are distinct from feature
requests — they represent cases where existing functionality produces
wrong results. Bugs should generally be fixed before new features at
the same impact tier.

Items are ordered by **impact** (descending), then **effort** (ascending)
within the same impact tier.

| Label      | Scale                                                                                                                  |
| ---------- | ---------------------------------------------------------------------------------------------------------------------- |
| **Impact** | **Critical**, **High**, **Medium-High**, **Medium**, **Low-Medium**, **Low**                                           |
| **Effort** | **Low** (≤ 1 day), **Medium** (2-5 days), **Medium-High** (1-2 weeks), **High** (2-4 weeks), **Very High** (> 1 month) |

---

#### B1. Nullable type prefix not stripped during diagnostic class lookup

| | |
|---|---|
| **Impact** | Medium-High |
| **Effort** | Low |

When a variable's resolved type is `?ClassName` (nullable shorthand),
the diagnostic pipeline uses the full string including the `?` prefix
as the class lookup key. The lookup fails, producing a spurious
"subject type '?Foo' could not be resolved" warning even though `Foo`
is a valid, loadable class.

**Observed:** 3 diagnostics in `shared` for
`?Luxplus\Core\Database\Model\Subscriptions\Subscription`. The class
exists and loads fine without the `?` prefix.

**Fix:** Strip the leading `?` (and/or `null|` / `|null` union
components) before class lookup in the diagnostic subject resolution
path. The completion pipeline already handles this correctly via
`clean_type()`, so the gap is specific to the diagnostic code path.

---

#### B2. Generic type parameters not stripped during diagnostic class lookup

| | |
|---|---|
| **Impact** | Medium |
| **Effort** | Low |

When a resolved type string includes generic parameters (e.g.
`PaymentOptionLocaleCollection<PaymentOptionLocale>`), the diagnostic
pipeline uses the full parameterised string as the class lookup key.
The lookup fails because no class is registered under the name that
includes `<...>`.

**Observed:** 3 diagnostics in `shared` for
`PaymentOptionLocaleCollection<Luxplus\Core\Database\Model\Payments\PaymentOptionLocale>`.
Methods like `getTotalWeight()`, `isNotEmpty()`, and `first()` all
exist on the class or its parent `Collection`.

**Fix:** Strip everything from the first `<` onward before performing
class lookup. `clean_type()` already does this for the completion
pipeline; the diagnostic resolution path needs the same treatment.

---

#### B3. Trait static/self suppression not applied inside closures

| | |
|---|---|
| **Impact** | Medium |
| **Effort** | Low |

The diagnostic pass suppresses `$this`/`self`/`static`/`parent` member
access inside traits (since the host class provides those members). This
works for direct method bodies but fails when the access is inside a
closure nested within a trait method. The closure's byte offset is still
within the trait's range, and `find_innermost_enclosing_class` returns
the trait, but something in the suppression path doesn't fire.

**Observed:** `SalesInfoGlobalTrait` has `static::where()` and
`static::query()` inside `retry(3, function() { ... })`. These produce
"Method 'where' not found on class 'SalesInfoGlobalTrait'" (2
diagnostics). Direct trait method bodies are correctly suppressed.

**Fix:** Investigate why the `subject_text == "static"` check on the
`MemberAccess` span doesn't match when the call is inside a closure.
The `expr_to_subject_text` function returns `"static"` for
`Expression::Static`, so the span should have the right subject text.
Possibly the closure introduces a scope boundary that changes how the
span is emitted or how the enclosing class is resolved.

---

#### B4. Variable reassignment loses type when parameter name is reused

| | |
|---|---|
| **Impact** | Medium |
| **Effort** | Medium |

When a method parameter is reassigned mid-body, PHPantom sometimes
continues to use the parameter's original type instead of the new
assignment's type.

**Observed:** In `FileUploadService::uploadFile()`, the `$file`
parameter is typed `UploadedFile`. Later, `$file = $result->getFile()`
reassigns it to a different type. PHPantom still resolves `$file->id`
and `$file->name` against `UploadedFile` instead of the model returned
by `getFile()`. This produces 2 false-positive "not found" diagnostics.

**Fix:** The variable resolution pipeline should prefer the most recent
assignment when multiple definitions exist for the same variable name
within the same scope at the cursor offset.

---

#### B5. Docblock `@see` reference prepends file namespace

| | |
|---|---|
| **Impact** | Low |
| **Effort** | Low |

When a docblock contains `@see Fully\Qualified\ClassName`, PHPantom
prepends the current file's namespace to the reference, producing an
invalid doubled namespace like
`Luxplus\Core\Database\Model\Products\Filters\Luxplus\Core\Elasticsearch\Queries\ProductQuery`.

**Observed:** 1 diagnostic in `ProductFilterTermCollection.php` where
`@see Luxplus\Core\Elasticsearch\Queries\ProductQuery::search_with_filter()`
becomes an unknown class with a doubled namespace prefix.

**Fix:** Treat `@see` references the same as `use` imports: if the
reference is already fully qualified (starts with the root namespace or
matches a known class), do not prepend the file namespace.

---

#### B6. Empty subject string in diagnostic messages

| | |
|---|---|
| **Impact** | Low |
| **Effort** | Low |

Some diagnostics display "Cannot resolve type of ''" with an empty
subject string. This happens when the subject extraction fails to
produce a text representation for complex expressions but a diagnostic
is still emitted.

**Observed:** 5 diagnostics in `shared` with empty subject strings,
triggered by patterns like `($a ?: $b)?->property` (ternary inside
nullable access) and similar compound expressions.

**Fix:** Skip emitting the `unresolved_member_access` diagnostic when
the extracted subject text is empty, or improve `expr_to_subject_text`
to handle ternary-in-nullable and similar compound patterns.

---

#### B7. Overloaded built-in function signatures not representable in stubs

| | |
|---|---|
| **Impact** | Low |
| **Effort** | Low |

Some PHP built-in functions have genuinely overloaded signatures where
the valid argument counts depend on which "form" is being called. The
phpstorm-stubs format cannot express this: it declares a single
signature, so one form's required parameters become false requirements
for the other form.

Around 415 cases where parameters were simply missing their default
values have been fixed upstream in phpstorm-stubs. The remaining cases
are true overloads that the stub format cannot represent:

- `array_keys(array $array): array` vs
  `array_keys(array $array, mixed $filter_value, bool $strict = false): array`
- `mt_rand(): int` vs `mt_rand(int $min, int $max): int`

PHPStan solves this with a separate function signature map
(`functionMap.php`) that overrides stub signatures with corrected
metadata including multiple accepted argument count ranges. PHPantom
needs a similar mechanism.

**Observed:** 10 diagnostics in `shared` (8 `array_keys`, 2 `mt_rand`).

**Fix:** Maintain a small overload map (similar to PHPStan's
`functionMap.php`) that declares alternative minimum argument counts
for functions with true overloads. The argument count checker consults
this map before flagging. The map only needs entries for functions
where the stub's single signature cannot represent the valid call
forms.

---

#### B8. `getCode`/`getMessage` not found through deep inheritance chains

| | |
|---|---|
| **Impact** | Low |
| **Effort** | Low |

Methods inherited from `Throwable` (like `getCode()` and
`getMessage()`) are not found on `QueryException`, which inherits
through `QueryException → PDOException → RuntimeException → Exception`.
The chain involves both vendor classes and stub classes.

**Observed:** 3 diagnostics in `shared` for `getCode()` and
`getMessage()` on `Illuminate\Database\QueryException`.

**Fix:** Investigate whether the inheritance chain breaks at the
vendor-to-stub boundary (PDOException is in stubs, RuntimeException
is in stubs). The chain resolution may stop walking when it crosses
from a vendor class to a stub class, or the depth limit may be
insufficient.