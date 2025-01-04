use clap::{Parser, Subcommand};
use kalamos::{
    config::Config,
    deploy::{self},
    render, serve, watch,
};
use log::info;
use std::{path::PathBuf, thread};

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

    /// Serve a static site and watch for changes to the input directory.
    /// To see logs, run with `RUST_LOG=info kalamos serve`
    #[command()]
    Serve {
        /// The directory to serve
        #[arg(default_value = DEFAULT_INPUT_DIR)]
        input_dir: PathBuf,
        /// The port to serve on
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,
        /// the output directory.
        #[arg(default_value = DEFAULT_OUTPUT_DIR, short, long)]
        output_dir: PathBuf,
    },

    #[command()]
    Deploy {
        /// The directory to generate the site from
        #[arg(default_value = DEFAULT_INPUT_DIR)]
        input_dir: PathBuf,
        /// The directory of the generated site
        #[arg(default_value = DEFAULT_OUTPUT_DIR)]
        output_dir: PathBuf,
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

#[tokio::main]
async fn main() {
    env_logger::init();
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
        Commands::Serve {
            input_dir,
            output_dir,
            port,
        } => {
            info!("Serving {:?} on port {}...", input_dir, port);
            let output_dir_clone = output_dir.clone();

            let spawner = thread::spawn(move || {
                serve::serve(&output_dir_clone, port).unwrap_or_else(|e| {
                    panic!("Error serving: {:?}", e);
                });
            });
            let watcher = thread::spawn(move || {
                info!(
                    "Watching {:?} and outputting to {:?}",
                    input_dir, output_dir
                );
                watch::watch(&input_dir, &output_dir).unwrap_or_else(|e| {
                    panic!("Error watching: {:?}", e);
                });
            });
            spawner.join().unwrap();
            watcher.join().unwrap();
        }
        Commands::Deploy {
            input_dir,
            output_dir,
        } => {
            let config = Config::load(&input_dir).unwrap_or_else(|e| {
                panic!("Error loading config: {:?}", e);
            });
            if let Some(config) = config {
                deploy::deploy(&input_dir, &output_dir, &config.deploy.map(|c| c.into()))
                    .await
                    .unwrap_or_else(|e| panic!("Error deploying: {:?}", e));
            } else {
                println!("No config file found");
            }
        }
        Commands::New { name, template } => {
            info!("New site: {:?}, template: {:?}", name, template);
        }
    }
}
