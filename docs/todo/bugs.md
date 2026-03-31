# PHPantom — Bug Fixes

#### B17. `&&` short-circuit narrowing does not eliminate `null` for null-initialized variables

| | |
|---|---|
| **Impact** | Low |
| **Effort** | Low |

When a variable is initialized as `null` and later guarded by
`$var !== null &&` in the same `if` condition, PHPantom still
resolves the variable as `null` on the right side of `&&`,
producing a `scalar_member_access` diagnostic.

**Reproducer:**

```php
$lastPaidEnd = null;

foreach ($periods as $period) {
    if ($lastPaidEnd !== null && $lastPaidEnd->diffInDays($periodStart) > 0) {
        // PHPantom reports: Cannot access method 'diffInDays' on type 'null'
    }
    $lastPaidEnd = $period->ending->startOfDay();
}
```

**Expected:** The `!== null` check on the left side of `&&` should
narrow away `null` for the right side, resolving `$lastPaidEnd` to
`Carbon` (or whatever the reassignment type is).

**Root cause:** This is the remaining gap from the earlier B11 fix
(null-init + guard clause). The partial fix handled early-return
guard clauses (`if ($x === null) return;`) but does not handle
`&&` short-circuit narrowing where the null check and the member
access are in the same compound condition.

**Where to fix:**
- `src/completion/types/narrowing.rs` — extend the `&&` narrowing
  logic to propagate `!== null` / `!is_null()` guards to later
  operands in the same compound condition.

**Discovered in:** analyze-triage iteration 8 (CustomerService.php:302).