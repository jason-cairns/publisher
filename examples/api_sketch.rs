use std::process::ExitCode;

use publisher::{InspectOptions, inspect_publication, parse_publication};

fn main() -> ExitCode {
    let debug_defaults = std::env::args()
        .skip(1)
        .any(|arg| arg == "--debug-defaults");
    let root = "examples/api_sketch_site/index.typ";

    let publication = match parse_publication(root) {
        Ok(publication) => publication,
        Err(error) => {
            eprintln!("failed to parse {root}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let report = publication.validate();
    let options = InspectOptions::new()
        .debug_defaults(debug_defaults)
        .source_root("examples/api_sketch_site");

    print!("{}", inspect_publication(&publication, &report, &options));

    if report.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
