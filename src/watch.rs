use crate::render;
use log::info;
use notify::{Error, Event, RecursiveMode, Watcher};
use std::{path::Path, sync::mpsc};

pub fn watch(input_dir: &Path, output_dir: &Path) -> Result<(), Error> {
    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

    // Use recommended_watcher() to automatically select the best implementation
    // for your platform. The `EventHandler` passed to this constructor can be a
    // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
    // another type the trait is implemented for.
    let mut watcher =
        notify::recommended_watcher(tx).unwrap_or_else(|e| panic!("notify error: ${e}"));

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
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
