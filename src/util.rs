use std::{fs, path::Path};

use walkdir::WalkDir;

use crate::render::Error;

pub fn copy_dir(src: &Path, dst: &Path) -> Result<(), Error> {
    let src = src
        .canonicalize()
        .map_err(|e| Error::Path(src.to_path_buf(), e.to_string()))?;

    fs::create_dir_all(dst).map_err(Error::CopyDir)?;
    let dst = dst
        .canonicalize()
        .map_err(|e| Error::Path(dst.to_path_buf(), e.to_string()))?;
    for entry in WalkDir::new(&src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let p = entry.path();
        let stripped = p
            .strip_prefix(&src)
            .map_err(|e| Error::StripPrefix(p.to_path_buf(), e))?;
        let output_path = dst.join(stripped);
        let output_dir = output_path.parent().ok_or(Error::Path(
            output_path.to_path_buf(),
            "parent not found".to_string(),
        ))?;
        fs::create_dir_all(output_dir).map_err(Error::CopyDir)?;
        fs::copy(p, output_path).map_err(Error::CopyDir)?;
    }
    Ok(())
}
