use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use phpantom_lsp::Backend;
use phpantom_lsp::config;
use tower_lsp::{LspService, Server};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Green.on_default());

#[derive(Parser)]
#[command(name = "phpantom_lsp", styles = STYLES)]
#[command(
    version,
    about = "A fast and lightweight PHP Language Server Protocol implementation"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Create a default .phpantom.toml configuration file in the current directory and exit.
    #[arg(long, global = true)]
    init: bool,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Analyze PHP files and report diagnostics in a PHPStan-like table format.
    ///
    /// Runs PHPantom's own diagnostics (no PHPStan, no external tools) across
    /// your codebase. Useful for measuring type coverage and finding gaps
    /// without opening files one by one in an editor.
    Analyze {
        /// Path to analyze (file or directory). Defaults to the entire project.
        #[arg(value_name = "PATH")]
        path: Option<std::path::PathBuf>,

        /// Minimum severity level to report.
        #[arg(long, default_value = "all")]
        severity: SeverityArg,

        /// Disable coloured output.
        #[arg(long)]
        no_colour: bool,

        /// Project root directory. Defaults to the current working directory.
        #[arg(long, value_name = "DIR")]
        project_root: Option<std::path::PathBuf>,
    },
}

/// Minimum severity level for the analyze command.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum SeverityArg {
    /// Show all diagnostics (error, warning, info, hint).
    All,
    /// Show only errors and warnings.
    Warning,
    /// Show only errors.
    Error,
}

impl From<SeverityArg> for phpantom_lsp::analyse::SeverityFilter {
    fn from(arg: SeverityArg) -> Self {
        match arg {
            SeverityArg::All => Self::All,
            SeverityArg::Warning => Self::Warning,
            SeverityArg::Error => Self::Error,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.init {
        let cwd = std::env::current_dir().unwrap_or_else(|e| {
            eprintln!("Error: cannot determine current directory: {}", e);
            std::process::exit(1);
        });

        match config::create_default_config(&cwd) {
            Ok(true) => {
                println!("Created {} in {}", config::CONFIG_FILE_NAME, cwd.display());
            }
            Ok(false) => {
                println!(
                    "{} already exists in {}",
                    config::CONFIG_FILE_NAME,
                    cwd.display()
                );
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        return;
    }

    match cli.command {
        Some(Command::Analyze {
            path,
            severity,
            no_colour,
            project_root,
        }) => {
            let workspace_root = project_root
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| {
                    eprintln!("Error: cannot determine project root directory");
                    std::process::exit(1);
                });

            // Auto-detect colour support: enabled unless --no-colour is
            // passed or stdout is not a terminal.
            let use_colour = !no_colour && atty_stdout();

            let options = phpantom_lsp::analyse::AnalyseOptions {
                workspace_root,
                path_filter: path,
                severity_filter: severity.into(),
                use_colour,
            };

            let exit_code = phpantom_lsp::analyse::run(options).await;
            std::process::exit(exit_code);
        }
        None => {
            // Default: run the LSP server over stdin/stdout.
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .with_writer(std::io::stderr)
                .init();

            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::build(Backend::new).finish();

            Server::new(stdin, stdout, socket).serve(service).await;
        }
    }
}

/// Check if stdout is a terminal (for colour auto-detection).
fn atty_stdout() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}
