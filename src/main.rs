use clap::Parser;
use kalamos::render;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// The path to the root of the site. This will contain the pages and layouts directories.
    #[arg(short, long)]
    path: PathBuf,
    /// The path to the output directory. This will contain the rendered pages.
    #[arg(short, long)]
    output: PathBuf,
}

/// Render a static site.
///  cargo run -- --path tests/it/testdata/simple_site --output /tmp/output
fn main() {
    let args = Args::parse();
    let templates = render::load_templates(&args.path).expect("should load templates");
    let pages = render::pages(&templates, &args.path, &args.output).expect("should load pages");
    for page in pages {
        println!("{:?}", page);
    }
}
