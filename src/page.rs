//! A page is a maud template and its markdown content
use std::fs;
use std::path::{Path, PathBuf};
use tera::{self, Context, Tera};
use thiserror::Error;
use walkdir::WalkDir;

use crate::markdown;

#[derive(Error, Debug)]
pub enum Error {
    #[error("tera error")]
    Tera(tera::Error),
    #[error("path error")]
    Path(PathBuf),
    #[error("read error")]
    ReadFile(std::io::Error),
    #[error("markdown error")]
    Markdown(markdown::Error),
    #[error("write error")]
    WriteFile(std::io::Error),
}
/// pass in a path containing glob patterns for the pages
/// Eg. load_templates("/path/to/project") would load all the templates in /path/to/project/layouts/*.html
pub fn load_templates(path: &Path) -> Result<Tera, Error> {
    let layout_path = path.join("layouts/*.html");
    let layout_path = layout_path
        .to_str()
        .ok_or(Error::Path(path.to_path_buf()))?;
    Tera::new(layout_path).map_err(Error::Tera)
}

pub fn render_pages(
    templates: &Tera,
    pages_dir: &Path,
    output_dir: &Path,
) -> Result<Vec<String>, Error> {
    let pages_path = pages_dir.join("pages");
    let pages_path = pages_path
        .to_str()
        .ok_or(Error::Path(pages_dir.to_path_buf()))?;
    // get all the md files in the pages directory and render them using the default layout
    fs::create_dir_all(output_dir).map_err(Error::WriteFile)?;
    WalkDir::new(pages_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|e| e == "md"))
        .map(|e| e.path().to_path_buf())
        .map(|p| -> Result<String, Error> {
            let content = fs::read_to_string(&p).map_err(Error::ReadFile)?;
            let page = markdown::parse(&content).map_err(Error::Markdown)?;
            let context = Context::from(page);
            let template = context
                .get("template")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            let content = render_layout(templates, &format!("{template}.html"), &context)?;
            let rest_of_path = p.strip_prefix(pages_path).unwrap();
            let output_path = output_dir.join(rest_of_path).with_extension("html");
            fs::write(&output_path, &content).map_err(Error::WriteFile)?;
            Ok(content)
        })
        .collect::<Result<Vec<_>, Error>>()
}

pub fn render_layout(templates: &Tera, layout: &str, context: &Context) -> Result<String, Error> {
    templates.render(layout, context).map_err(Error::Tera)
}
