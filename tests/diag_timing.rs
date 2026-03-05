//! Quick timing test to measure diagnostic performance on `example.php`.
//!
//! Run with:
//!   cargo test --release -p phpantom_lsp --test diag_timing -- --nocapture

mod common;

use common::create_test_backend;
use std::time::Instant;

#[tokio::test]
async fn time_diagnostics_on_example_php() {
    let content = std::fs::read_to_string("example.php").expect("failed to read example.php");
    let uri = "file:///example.php";
    let backend = create_test_backend();
    backend.update_ast(uri, &content);

    // ── Deprecated diagnostics ──────────────────────────────────────────
    let start = Instant::now();
    let mut deprecated_out = Vec::new();
    backend.collect_deprecated_diagnostics(uri, &content, &mut deprecated_out);
    let deprecated_time = start.elapsed();

    // ── Unused import diagnostics ───────────────────────────────────────
    let start = Instant::now();
    let mut unused_out = Vec::new();
    backend.collect_unused_import_diagnostics(uri, &content, &mut unused_out);
    let unused_time = start.elapsed();

    // ── Unknown class diagnostics ───────────────────────────────────────
    let start = Instant::now();
    let mut unknown_out = Vec::new();
    backend.collect_unknown_class_diagnostics(uri, &content, &mut unknown_out);
    let unknown_time = start.elapsed();

    let total = deprecated_time + unused_time + unknown_time;

    eprintln!();
    eprintln!(
        "=== Diagnostic timing on example.php ({} lines) ===",
        content.lines().count()
    );
    eprintln!(
        "  deprecated:     {:>10.3?}  ({} diagnostics)",
        deprecated_time,
        deprecated_out.len()
    );
    eprintln!(
        "  unused_imports: {:>10.3?}  ({} diagnostics)",
        unused_time,
        unused_out.len()
    );
    eprintln!(
        "  unknown_classes:{:>10.3?}  ({} diagnostics)",
        unknown_time,
        unknown_out.len()
    );
    eprintln!("  ──────────────────────────────────");
    eprintln!("  TOTAL:          {:>10.3?}", total);
    eprintln!();

    // In debug builds the threshold is generous (20 s); in release builds
    // diagnostics on example.php should comfortably finish under 2 s.
    let budget_secs = if cfg!(debug_assertions) { 20.0 } else { 2.0 };
    assert!(
        total.as_secs_f64() < budget_secs,
        "Diagnostics took {:.3?} which exceeds the {:.0} s budget. \
         This runs on every keystroke — investigate which provider is slow.",
        total,
        budget_secs,
    );
}

#[tokio::test]
async fn time_diagnostics_on_phpstan_fixture() {
    let path = "benches/fixtures/diagnostics/phpstan.php";
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping: {path} not found");
            return;
        }
    };
    let uri = "file:///bench/phpstan.php";
    let backend = create_test_backend();
    backend.update_ast(uri, &content);

    let start = Instant::now();
    let mut deprecated_out = Vec::new();
    backend.collect_deprecated_diagnostics(uri, &content, &mut deprecated_out);
    let deprecated_time = start.elapsed();

    let start = Instant::now();
    let mut unused_out = Vec::new();
    backend.collect_unused_import_diagnostics(uri, &content, &mut unused_out);
    let unused_time = start.elapsed();

    let start = Instant::now();
    let mut unknown_out = Vec::new();
    backend.collect_unknown_class_diagnostics(uri, &content, &mut unknown_out);
    let unknown_time = start.elapsed();

    let total = deprecated_time + unused_time + unknown_time;

    eprintln!();
    eprintln!(
        "=== Diagnostic timing on phpstan.php ({} lines) ===",
        content.lines().count()
    );
    eprintln!(
        "  deprecated:     {:>10.3?}  ({} diagnostics)",
        deprecated_time,
        deprecated_out.len()
    );
    eprintln!(
        "  unused_imports: {:>10.3?}  ({} diagnostics)",
        unused_time,
        unused_out.len()
    );
    eprintln!(
        "  unknown_classes:{:>10.3?}  ({} diagnostics)",
        unknown_time,
        unknown_out.len()
    );
    eprintln!("  ──────────────────────────────────");
    eprintln!("  TOTAL:          {:>10.3?}", total);
    eprintln!();

    let budget_secs = if cfg!(debug_assertions) { 120.0 } else { 5.0 };
    assert!(
        total.as_secs_f64() < budget_secs,
        "Diagnostics took {:.3?} on the large phpstan fixture — too slow for interactive use \
         (budget: {:.0} s).",
        total,
        budget_secs,
    );
}
