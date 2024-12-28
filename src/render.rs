//! Render the whole static site.
use std::path::{Path, PathBuf};
use tera::{self, Context, Tera};
use thiserror::Error;
use walkdir::WalkDir;

use crate::markdown;
use crate::page::Page;
use crate::post::Post;

pub trait Render
where
    Self: Sized,
{
    fn from_file(root_path: &Path, path: &Path) -> Result<Self, Error>;

    fn to_context(&self) -> Context;

    fn render(&self, templates: &Tera, output_dir: &Path, posts: &[Post]) -> Result<(), Error>;
}

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
    #[error("parse frontmatter error")]
    ParseFrontmatter(String),
    #[error("missing field")]
    MissingField(String),
    #[error("parse date error")]
    ParseDate(String, chrono::ParseError),
    #[error("strip prefix error")]
    StripPrefix(std::path::StripPrefixError),
    #[error("create dir error")]
    CreateDir(std::io::Error),
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

pub fn render_all(templates: &Tera, root_dir: &Path, output_dir: &Path) -> Result<(), Error> {
    // get all the md files in the posts directory and create Posts from them
    let posts_path = root_dir.join("posts");
    let posts = WalkDir::new(posts_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|e| e == "md"))
        .map(|e| -> Result<PathBuf, Error> {
            let p = e.path().to_path_buf();
            Ok(p.strip_prefix(root_dir)
                .map_err(Error::StripPrefix)?
                .to_path_buf())
        })
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .map(|p| Post::from_file(root_dir, &p))
        .collect::<Result<Vec<_>, Error>>()?;
    println!("read posts");

    for post in &posts {
        post.render(templates, output_dir, &posts)?;
    }
    println!("rendered posts");

    // get all the md files in the pages directory and create Pages from them
    let pages_path = root_dir.join("pages");
    let pages = WalkDir::new(pages_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|e| e == "md"))
        .map(|e| -> Result<PathBuf, Error> {
            let p = e.path().to_path_buf();
            Ok(p.strip_prefix(root_dir)
                .map_err(Error::StripPrefix)?
                .to_path_buf())
        })
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .map(|p| Page::from_file(root_dir, &p))
        .collect::<Result<Vec<_>, Error>>()?;
    println!("read pages");
    for page in &pages {
        page.render(templates, output_dir, &posts)?;
    }
    println!("rendered pages");
    Ok(())
}
