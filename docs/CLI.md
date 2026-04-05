# CLI Reference

PHPantom is a language server, but it also ships CLI tools for batch
analysis and automated fixing. These run the same engine that powers the
editor, so results are consistent.

## Modes

| Command                  | Purpose                                              |
| ------------------------ | ---------------------------------------------------- |
| `phpantom_lsp`           | Start the LSP server over stdin/stdout (the default) |
| `phpantom_lsp analyze`   | Report diagnostics across the project                |
| `phpantom_lsp fix`       | Apply automated code fixes across the project        |
| `phpantom_lsp init`      | Generate a default `.phpantom.toml` config file      |

Running with no subcommand starts the language server. Editors launch
this automatically.

---

## `analyze`

Scans PHP files and reports PHPantom diagnostics in a PHPStan-style
table format. Use it to find spots where the LSP cannot resolve a
symbol, so you can achieve full completion coverage.

```sh
phpantom_lsp analyze                             # scan entire project
phpantom_lsp analyze src/                        # scan a subdirectory
phpantom_lsp analyze src/Foo.php                 # scan a single file
phpantom_lsp analyze --severity warning          # errors and warnings only
phpantom_lsp analyze --severity error            # errors only
phpantom_lsp analyze --project-root /path/to/app # explicit project root
phpantom_lsp analyze --no-colour                 # plain text output
```

### Options

| Flag                       | Description                                                      |
| -------------------------- | ---------------------------------------------------------------- |
| `[PATH]`                   | File or directory to analyze. Defaults to the entire project.    |
| `--severity <LEVEL>`       | Minimum severity: `all` (default), `warning`, or `error`.        |
| `--project-root <DIR>`     | Project root directory. Defaults to the current working directory.|
| `--no-colour`              | Disable ANSI colour output.                                      |

### Exit codes

| Code | Meaning                 |
| ---- | ----------------------- |
| 0    | No diagnostics found    |
| 1    | Diagnostics were found  |

### Example output

```
 ------ -------------------------------------------
   Line   src/Service/UserService.php
 ------ -------------------------------------------
   15     Unknown class 'App\Models\LegacyUser'.
          🪪  unknown_class
   42     Call to undefined method Post::archive().
          🪪  unknown_member
 ------ -------------------------------------------
```

### Reported diagnostics

The analyze command reports the same diagnostics you see in your editor.
Each has a rule identifier shown below the message.

| Identifier               | Severity | Description                                          |
| ------------------------ | -------- | ---------------------------------------------------- |
| `syntax_error`           | Error    | PHP parse errors                                     |
| `unknown_class`          | Warning  | Class, interface, trait, or enum not resolvable       |
| `unknown_member`         | Warning  | Property or method not found on the resolved class    |
| `unknown_function`       | Error    | Function call not resolvable                          |
| `argument_count`         | Error    | Wrong number of arguments to a function or method     |
| `implementation_error`   | Error    | Missing required interface or abstract methods        |
| `scalar_member_access`   | Error    | Member access on a scalar type (int, string, etc.)    |
| `unused_import`          | Hint     | `use` statement with no references in the file        |
| `deprecated`             | Hint     | Reference to a `@deprecated` symbol                   |

---

## `fix`

Applies code fixes across the project. Specify which rules to run, or
omit `--rule` to run all preferred native fixers.

```sh
phpantom_lsp fix                                  # apply all preferred fixers
phpantom_lsp fix --rule unused_import             # only remove unused imports
phpantom_lsp fix --rule unused_import --rule deprecated  # multiple rules
phpantom_lsp fix --dry-run                        # preview without writing
phpantom_lsp fix src/                             # restrict to a subdirectory
phpantom_lsp fix src/Foo.php                      # fix a single file
phpantom_lsp fix --project-root /path/to/app      # explicit project root
```

### Options

| Flag                       | Description                                                          |
| -------------------------- | -------------------------------------------------------------------- |
| `[PATH]`                   | File or directory to fix. Defaults to the entire project.            |
| `--rule <RULE>`            | Rule to apply (repeatable). Omit to run all preferred native rules.  |
| `--dry-run`                | Report what would change without writing files.                      |
| `--with-phpstan`           | Enable PHPStan-based fixers (future feature).                        |
| `--project-root <DIR>`     | Project root directory. Defaults to the current working directory.    |
| `--no-colour`              | Disable ANSI colour output.                                         |

### Exit codes

| Code | Meaning                                          |
| ---- | ------------------------------------------------ |
| 0    | Fixes applied successfully (or nothing to fix)   |
| 1    | Error (bad arguments, write failure, etc.)       |
| 2    | Dry-run found fixable issues (nothing written)   |

### Available rules

Rules correspond to diagnostic identifiers.

| Rule               | Description                    |
| ------------------ | ------------------------------ |
| `unused_import`    | Remove unused `use` statements |

### Example output

```
 ------ -------------------------------------------
   Line   src/Service/UserService.php
 ------ -------------------------------------------
    5     Unused import 'App\Models\LegacyUser'
          🔧  unused_import
    6     Unused import 'App\Support\OldHelper'
          🔧  unused_import
 ------ -------------------------------------------

 [FIXED] Applied 2 fixes across 1 file
```

### Dry-run example

```sh
phpantom_lsp fix --dry-run --project-root /path/to/app
```

```
 ------ -------------------------------------------
   Line   src/Service/UserService.php
 ------ -------------------------------------------
    5     Unused import 'App\Models\LegacyUser'
          🔧  unused_import
 ------ -------------------------------------------

 [DRY RUN] 1 fix in 1 file (not applied)
```

Use `--dry-run` in CI to enforce that imports stay clean:

```sh
phpantom_lsp fix --dry-run --rule unused_import --project-root .
# Exit code 2 means fixable issues exist. Fail the build.
```

### Idempotency

Running `fix` twice produces the same result as running it once. If
all issues are already resolved, the command exits with code 0 and
writes nothing.

---

## `init`

Creates a default `.phpantom.toml` in the current directory with all
options documented and commented out. Safe to run if the file already
exists (it will not overwrite).

```sh
phpantom_lsp init
```

See [Project Configuration](SETUP.md#project-configuration) for details
on available settings.

---

## Common patterns

### CI pipeline: diagnostics gate

Fail the build when PHPantom finds unresolvable symbols:

```sh
phpantom_lsp analyze --severity warning --project-root . --no-colour
```

### CI pipeline: enforce clean imports

Fail the build when unused imports exist:

```sh
phpantom_lsp fix --dry-run --rule unused_import --project-root . --no-colour
```

### Pre-commit hook: auto-fix imports

Clean up imports before every commit:

```sh
phpantom_lsp fix --rule unused_import --project-root .
```

### Combine analyze and fix

Run fixes first, then check what remains:

```sh
phpantom_lsp fix --project-root .
phpantom_lsp analyze --project-root .
```
