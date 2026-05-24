use std::process::ExitCode;

use publisher::{
    InspectOptions, RenderOptions, inspect_publication, parse_publication, render_publication,
};

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

    if !report.is_ok() {
        return ExitCode::FAILURE;
    }

    let render_options = RenderOptions {
        source_root: "examples/api_sketch_site".into(),
        output_dir: "build/api-sketch".into(),
        artifact_name: "api-sketch".to_string(),
    };

    match render_publication(&publication, &render_options) {
        Ok(artifacts) => {
            println!("wrote {}", artifacts.typst_path.display());
            println!("wrote {}", artifacts.pdf_path.display());
            for html_path in artifacts.html_paths {
                println!("wrote {}", html_path.display());
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("failed to render {root}: {error}");
            ExitCode::FAILURE
        }
    }
}
