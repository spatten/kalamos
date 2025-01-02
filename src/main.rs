use clap::{Parser, Subcommand};
use kalamos::render;
use log::info;
use notify::{Event, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::mpsc};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate the static site.
    /// To see logs, run with `RUST_LOG=info kalamos generate`
    Generate {
        /// the input directory. Defaults to the current directory.
        #[arg(default_value = DEFAULT_INPUT_DIR, short, long)]
        input_dir: PathBuf,
        /// the output directory.
        #[arg(default_value = DEFAULT_OUTPUT_DIR, short, long)]
        output_dir: PathBuf,
    },

    /// Watch the file system and rebuild if the files change
    /// To see logs, run with `RUST_LOG=info kalamos watch`
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
    /// To see logs, run with `RUST_LOG=info kalamos serve`
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
            info!("input_dir: {:?}, output_dir: {:?}", input_dir, output_dir);
            render::render_dir(&input_dir, &output_dir).unwrap_or_else(|e| {
                panic!("Error rendering posts and pages: {}", e);
            });
        }
        Commands::Serve { input_dir, port } => {
            info!("Serving {:?} on port {}...", input_dir, port);
        }
        Commands::Watch {
            input_dir,
            output_dir,
        } => {
            let input_dir = input_dir.canonicalize().unwrap();
            let output_dir = output_dir.canonicalize().unwrap();
            info!(
                "Watching {:?} and outputting to {:?}",
                input_dir, output_dir
            );
            let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

            // Use recommended_watcher() to automatically select the best implementation
            // for your platform. The `EventHandler` passed to this constructor can be a
            // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
            // another type the trait is implemented for.
            let mut watcher =
                notify::recommended_watcher(tx).unwrap_or_else(|e| panic!("notify error: ${e}"));

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            watcher.watch(&input_dir, RecursiveMode::Recursive).unwrap();
            for result in rx {
                match result {
                    Ok(event) => {
                        // deal with case where the output directory is a subdirectory of the input directory
                        if event.paths.iter().all(|p| p.starts_with(&output_dir)) {
                            continue;
                        }
                        info!("change event: {:?}", event);
                        render::render_dir(&input_dir, &output_dir).unwrap_or_else(|e| {
                            info!("Error rendering posts and pages: {}", e);
                        });
                    }
                    Err(e) => info!("change event error: {:?}", e),
                }
            }
        }
        Commands::New { name, template } => {
            info!("New site: {:?}, template: {:?}", name, template);
        }
    }
}
