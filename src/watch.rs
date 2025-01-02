use crate::render;
use log::info;
use notify::{Error, Event, RecursiveMode, Watcher};
use std::{path::Path, sync::mpsc};

pub fn watch(input_dir: &Path, output_dir: &Path) -> Result<(), Error> {
    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

    let mut watcher =
        notify::recommended_watcher(tx).unwrap_or_else(|e| panic!("notify error: ${e}"));

    watcher.watch(input_dir, RecursiveMode::Recursive)?;
    for result in rx {
        match result {
            Ok(event) => {
                // deal with case where the output directory is a subdirectory of the input directory
                if event.paths.iter().all(|p| p.starts_with(output_dir)) {
                    continue;
                }
                info!("change event: {:?}", event);
                info!(
                    "Rendering posts and pages in {:?} to {:?}",
                    input_dir, output_dir
                );
                render::render_dir(input_dir, output_dir).unwrap_or_else(|e| {
                    info!("Error rendering posts and pages: {}", e);
                });
            }
            Err(e) => info!("change event error: {:?}", e),
        }
    }
    Ok(())
}
