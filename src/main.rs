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
        #[arg(default_value = DEFAULT_INPUT_DIR, short, long)]
        input_dir: PathBuf,
        /// the output directory.
        #[arg(default_value = DEFAULT_OUTPUT_DIR, short, long)]
        output_dir: PathBuf,
    },

    /// Watch the file system and rebuild if the files change
    #[command()]
    Watch {
        /// The directory to watch
        #[arg(default_value = DEFAULT_INPUT_DIR)]
        input_dir: PathBuf,
        /// the output directory.
        #[arg(default_value = DEFAULT_OUTPUT_DIR, short, long)]
        output_dir: PathBuf,
    },

    /// Serve a static site.
    #[command()]
    Serve {
        /// The directory to serve
        #[arg(default_value = DEFAULT_OUTPUT_DIR)]
        input_dir: PathBuf,
        /// The port to serve on
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },

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
const DEFAULT_INPUT_DIR: &str = ".";
const DEFAULT_PORT: u16 = 7878;

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Generate {
            input_dir,
            output_dir,
        } => {
            println!("input_dir: {:?}, output_dir: {:?}", input_dir, output_dir);
            render::render_dir(&input_dir, &output_dir).unwrap_or_else(|e| {
                panic!("Error rendering posts and pages: {}", e);
            });
        }
        Commands::Serve { input_dir, port } => {
            println!("Serving {:?} on port {}...", input_dir, port);
        }
        Commands::Watch {
            input_dir,
            output_dir,
        } => {
            println!(
                "Watching {:?} and outputting to {:?}",
                input_dir, output_dir
            );
        }
        Commands::New { name, template } => {
            println!("New site: {:?}, template: {:?}", name, template);
        }
    }
}
