use clap::Parser;
use kalamos::page;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    let templates = page::load_templates(&args.path).expect("should load templates");
    println!("{:?}", templates);
    let pages = page::render_pages(&templates, &args.path).expect("should load pages");
    for page in pages {
        println!("{}", page);
    }
}
