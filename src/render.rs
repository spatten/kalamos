//! Render the whole static site.
use std::fs;
use std::path::{Path, PathBuf};
use tera::{self, Context, Tera};
use thiserror::Error;
use walkdir::WalkDir;

use crate::page::Page;
use crate::parser;
use crate::post::Post;

pub trait RenderableFromPath: TryFrom<PathBuf, Error = Error> + std::fmt::Debug {
    fn url(&self) -> PathBuf;
    fn input_path(&self) -> PathBuf;
    fn output_path(&self) -> PathBuf;
}

pub trait Render
where
    Self: Sized,
{
    type FileType: RenderableFromPath;

    /// Create a Page or Post object from a file
    fn from_content(file: Self::FileType, content: &str) -> Result<Self, Error>;

    /// Generate a context for the template
    fn to_context(&self) -> Context;

    /// Render the file and write it to the output directory
    fn render(&self, templates: &Tera, output_dir: &Path, posts: &[Post]) -> Result<(), Error>;

    /// The directory to read from. For Posts, this is the posts directory. For Pages, this is the pages directory.
    fn read_directory() -> String;

    /// For Posts, read all files in the posts directory and create Posts from them
    /// For Pages, read all files in the pages directory and create Pages from them
    fn read_from_directory(root_dir: &Path) -> Result<Vec<Self>, Error> {
        let posts_path = root_dir.join(Self::read_directory());
        let post_files = WalkDir::new(posts_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| -> Result<Self::FileType, Error> {
                let p = e.path().to_path_buf();
                let path = p
                    .strip_prefix(root_dir)
                    .map_err(|e| Error::StripPrefix(p.clone(), e))?
                    .to_path_buf();
                Self::FileType::try_from(path)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let posts = post_files
            .into_iter()
            .map(|post_file| {
                let full_path = root_dir.join(post_file.input_path().as_path());
                let content = fs::read_to_string(full_path).map_err(Error::ReadFile)?;
                Self::from_content(post_file, &content)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(posts.into_iter().collect())
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
    #[error("strip prefix error: path: {0}: {1}")]
    StripPrefix(PathBuf, std::path::StripPrefixError),
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
    let mut posts = Post::read_from_directory(root_dir)?;
    posts.sort();
    posts.reverse();

    for post in &posts {
        post.render(&templates, output_dir, &posts)?;
    }

    // get all the md, html and xml files in the pages directory, render them and write them to the output directory
    let pages = Page::read_from_directory(root_dir)?;
    for page in &pages {
        page.render(&templates, output_dir, &posts)?;
    }

    // copy all files in the direct_copy directory
    let direct_copy_path = root_dir.join("direct_copy");
    for entry in WalkDir::new(&direct_copy_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let p = entry.path();
        let stripped = p
            .strip_prefix(&direct_copy_path)
            .map_err(|e| Error::StripPrefix(p.to_path_buf(), e))?;
        let output_path = output_dir.join(stripped);
        let output_dir = output_path
            .parent()
            .ok_or(Error::Path(output_path.to_path_buf()))?;
        fs::create_dir_all(output_dir).map_err(Error::CopyDir)?;
        fs::copy(p, output_path).map_err(Error::CopyDir)?;
    }
    Ok(())
}
