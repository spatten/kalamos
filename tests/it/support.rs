use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

/// Create a YAML representation of a directory.
/// binary files are represented as a hash of their contents.
/// text files are represented by their contents.
pub fn dir_to_yaml(root_dir: &Path) -> Result<BTreeMap<PathBuf, String>, Error> {
    let files = WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .map(|p| yaml_for_file(&p, root_dir))
        .collect::<Result<Vec<_>, _>>()?;
    let hashmap = files.into_iter().collect::<BTreeMap<_, _>>();
    Ok(hashmap)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("serde error")]
    Serde(#[from] serde_yaml::Error),
    #[error("strip prefix error")]
    StripPrefix(#[from] std::path::StripPrefixError),
}

fn yaml_for_file(path: &Path, root_dir: &Path) -> Result<(PathBuf, String), Error> {
    let stripped_path = path
        .strip_prefix(root_dir)
        .map_err(Error::StripPrefix)?
        .to_path_buf();
    if binaryornot::is_binary(path).map_err(Error::Io)? {
        let hash = hash_file(path)?;
        Ok((stripped_path, hash))
    } else {
        let contents = fs::read_to_string(path);
        if let Ok(contents) = contents {
            Ok((stripped_path, contents))
        } else {
            // if we get a UTF-8 encoding error or something like that, just hash the file
            let hash = hash_file(path)?;
            Ok((stripped_path, hash))
        }
    }
}

fn hash_file(path: &Path) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(path).map_err(Error::Io)?;
    io::copy(&mut file, &mut hasher).map_err(Error::Io)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
