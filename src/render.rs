//! Render the whole static site.
use std::fs;
use std::path::{Path, PathBuf};
use tera::{self, Context, Tera};
use thiserror::Error;
use walkdir::WalkDir;

use crate::page::Page;
use crate::parser;
use crate::post::Post;

pub trait Render
where
    Self: Sized,
{
    fn from_file(root_path: &Path, path: &Path) -> Result<Option<Self>, Error>;

    fn to_context(&self) -> Context;

    fn render(&self, templates: &Tera, output_dir: &Path, posts: &[Post]) -> Result<(), Error>;

    fn read_directory() -> String;

    fn read_from_directory(root_dir: &Path) -> Result<Vec<Self>, Error> {
        let posts_path = root_dir.join(Self::read_directory());
        let posts = WalkDir::new(posts_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| -> Result<PathBuf, Error> {
                let p = e.path().to_path_buf();
                Ok(p.strip_prefix(root_dir)
                    .map_err(Error::StripPrefix)?
                    .to_path_buf())
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .map(|p| Self::from_file(root_dir, &p))
            .collect::<Result<Vec<Option<_>>, Error>>()?;
        Ok(posts.into_iter().flatten().collect())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("tera error: {0}")]
    Tera(tera::Error),
    #[error("path error: {0}")]
    Path(PathBuf),
    #[error("read error: {0}")]
    ReadFile(std::io::Error),
    #[error("markdown error: {0}")]
    Markdown(parser::Error),
    #[error("write error: {0}")]
    WriteFile(std::io::Error),
    #[error("parse frontmatter error: {0}")]
    ParseFrontmatter(String),
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("extract date from file name: {0}. File name format should be YYYY-MM-DD-slug.md")]
    ExtractDate(String),
    #[error("parse date error: {0}")]
    ParseDate(String, chrono::ParseError),
    #[error("strip prefix error: {0}")]
    StripPrefix(std::path::StripPrefixError),
    #[error("create dir error: {0}")]
    CreateDir(std::io::Error),
    #[error("copy dir error: {0}")]
    CopyDir(std::io::Error),
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

pub fn render_dir(root_dir: &Path, output_dir: &Path) -> Result<(), Error> {
    let templates = load_templates(root_dir)?;
    // get all the md files in the posts directory and create Posts from them
    // We need the posts as a variable to pass to the render function for posts and pages.
    // It can be used, for example, to get a list of all the posts to pass to the RSS feed
    // or to get a list of posts for a sidebar or an archives page.
    let posts = Post::read_from_directory(root_dir)?;
    println!("read posts");

    for post in &posts {
        post.render(&templates, output_dir, &posts)?;
    }
    println!("rendered posts");

    // get all the md files in the pages directory and create Pages from them
    let pages = Page::read_from_directory(root_dir)?;
    println!("read pages");
    for page in &pages {
        page.render(&templates, output_dir, &posts)?;
    }
    println!("rendered pages");

    // copy static files from assets directory
    let assets_path = root_dir.join("assets");
    WalkDir::new(assets_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .for_each(|e| {
            let p = e.path();
            let output_path = output_dir.join(p.strip_prefix(root_dir).unwrap());
            let output_dir = output_path.parent().unwrap();
            fs::create_dir_all(output_dir).unwrap();
            fs::copy(p, output_path).unwrap();
        });
    Ok(())
}
