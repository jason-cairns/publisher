use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use publisher::{
    Edition, InspectOptions, RenderOptions, RenderTarget, ScopeId, inspect_publication,
    parse_publication, publication_scope_ids, render_target,
};

/// Publisher CLI: render a Typst publication or inspect its scope graph.
#[derive(Parser)]
#[command(name = "publisher", about = "Render and inspect a Typst publication")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render the publication or a named scope to HTML or PDF.
    Render {
        /// Root source document, e.g. index.typ.
        #[arg(long)]
        root: PathBuf,
        /// Edition to produce.
        #[arg(long, value_enum)]
        to: EditionArg,
        /// Render only this named scope (default: the whole publication).
        #[arg(long)]
        scope: Option<String>,
        /// Output directory (default: ./build).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Inspect the publication's scopes, graph, and diagnostics.
    Inspect {
        /// Root source document, e.g. index.typ.
        #[arg(long)]
        root: PathBuf,
        #[command(subcommand)]
        what: InspectWhat,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum EditionArg {
    Html,
    Pdf,
}

impl From<EditionArg> for Edition {
    fn from(value: EditionArg) -> Self {
        match value {
            EditionArg::Html => Edition::Html,
            EditionArg::Pdf => Edition::Pdf,
        }
    }
}

#[derive(Subcommand)]
enum InspectWhat {
    /// Print scopes, routes, active scope stacks, and diagnostics.
    Scopes,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Render {
            root,
            to,
            scope,
            out,
        } => render(root, to.into(), scope, out),
        Command::Inspect { root, what } => match what {
            InspectWhat::Scopes => inspect_scopes(root),
        },
    }
}

fn render(
    root: PathBuf,
    edition: Edition,
    scope: Option<String>,
    out: Option<PathBuf>,
) -> Result<(), String> {
    let publication = parse_publication(&root).map_err(|error| error.to_string())?;
    let source_root = source_root_for(&root);
    let output_dir = out.unwrap_or_else(|| PathBuf::from("build"));

    let target = match scope {
        Some(id) => RenderTarget::NamedScope(ScopeId::new(id)),
        None => RenderTarget::WholePublication,
    };
    let artifact_name = match &target {
        RenderTarget::NamedScope(id) => id.as_str().to_string(),
        _ => "publication".to_string(),
    };

    let options = RenderOptions {
        source_root,
        output_dir,
        artifact_name,
    };

    let artifacts = render_target(&publication, &target, edition, &options)
        .map_err(|error| error.to_string())?;

    for path in &artifacts.html_paths {
        println!("wrote {}", path.display());
    }
    if let Some(path) = &artifacts.pdf_path {
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn inspect_scopes(root: PathBuf) -> Result<(), String> {
    let publication = parse_publication(&root).map_err(|error| error.to_string())?;
    let report = publication.validate();
    let source_root = source_root_for(&root);
    let options = InspectOptions::new().source_root(source_root);

    print!("{}", inspect_publication(&publication, &report, &options));

    let scope_ids = publication_scope_ids(&publication);
    if !scope_ids.is_empty() {
        println!("\nRenderable scopes");
        for id in scope_ids {
            println!("  {id}");
        }
    }
    Ok(())
}

fn source_root_for(root: &Path) -> PathBuf {
    match root.parent() {
        Some(parent) if parent.as_os_str().is_empty() => PathBuf::from("."),
        Some(parent) => parent.to_path_buf(),
        None => PathBuf::from("."),
    }
}
