use clap::{Parser, Subcommand};
use kalamos::render;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate the static site.
    Generate {
        /// the input directory. Defaults to the current directory.
        input_dir: Option<PathBuf>,
        /// the output directory. Defaults to ./site
        output_dir: Option<PathBuf>,
    },

    /// Serve a static site.
    #[command()]
    Serve,

    /// Generate a new static site.
    #[command(arg_required_else_help = true)]
    New {
        /// the name of the new site
        name: String,
        /// The template to use for the new site
        template: String,
    },
}

const DEFAULT_OUTPUT_DIR: &str = "./site";

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Generate {
            input_dir,
            output_dir,
        } => {
            let input_dir = input_dir.unwrap_or(PathBuf::from("."));
            let output_dir = output_dir.unwrap_or(PathBuf::from(DEFAULT_OUTPUT_DIR));
            println!("input_dir: {:?}, output_dir: {:?}", input_dir, output_dir);
            render::render_dir(&input_dir, &output_dir).unwrap_or_else(|e| {
                panic!("Error rendering posts and pages: {}", e);
            });
        }
        Commands::Serve => {
            println!("Serving...");
        }
        Commands::New { name, template } => {
            println!("New site: {:?}, template: {:?}", name, template);
        }
    }
}
