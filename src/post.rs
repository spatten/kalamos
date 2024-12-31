use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::parser;
use crate::render::Render;
use crate::render::{Error as RenderError, RenderableFromPath};

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    /// path of the input file, relative to the root of the site
    /// posts/my-post.html
    pub input_path: PathBuf,
    /// path of the output file, relative to the root of the site
    /// 2024/12/28/my-post.html
    pub output_path: PathBuf,
    /// the title of the page
    pub title: String,
    /// the template to use to render the page
    pub template: String,
    /// The content of the page
    pub content: String,
    /// The date the post was published
    pub date: NaiveDate,
    /// The url of the post. This is output_path, but with a leading / and an extension of html
    /// /2024/12/28/my-post.html
    pub url: PathBuf,
    /// The slug of the post
    /// my-post
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostFile {
    pub date: NaiveDate,
    pub slug: String,
    pub extension: String,
    pub url: PathBuf,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

impl TryFrom<PathBuf> for PostFile {
    type Error = RenderError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let (date, slug) = Self::extract_date_and_slug(&path)?;
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if !Post::VALID_EXTENSIONS.contains(&extension) {
            return Err(RenderError::Path(path.to_path_buf()));
        }
        let url = PathBuf::from(format!("/{}/{}/{}.html", date.year(), date.month(), slug));
        let output_path = PathBuf::from(format!("{}/{}/{}.html", date.year(), date.month(), slug));
        Ok(Self {
            date,
            slug,
            extension: extension.to_string(),
            url: url.clone(),
            input_path: path.to_path_buf(),
            output_path: output_path.clone(),
        })
    }
}

impl RenderableFromPath for PostFile {
    fn url(&self) -> PathBuf {
        self.url.clone()
    }

    fn input_path(&self) -> PathBuf {
        self.input_path.clone()
    }

    fn output_path(&self) -> PathBuf {
        self.output_path.clone()
    }
}

impl PostFile {
    /// Extracts the date and slug from a file name
    /// The file name must be in the format YYYY-MM-DD-slug.md
    fn extract_date_and_slug(path: &Path) -> Result<(NaiveDate, String), RenderError> {
        let path = path.with_extension("");
        let file_name = path
            .file_name()
            .ok_or(RenderError::ExtractDate(path.to_string_lossy().to_string()))?
            .to_str()
            .ok_or(RenderError::ExtractDate(path.to_string_lossy().to_string()))?;
        let parts = file_name.split("-").collect::<Vec<&str>>();
        if parts.len() < 4 {
            return Err(RenderError::ExtractDate(path.to_string_lossy().to_string()));
        }
        let date = parts
            .clone()
            .into_iter()
            .take(3)
            .collect::<Vec<_>>()
            .join("-");
        let slug = parts.into_iter().skip(3).collect::<Vec<_>>().join("-");
        let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map_err(|e| RenderError::ParseDate(path.to_string_lossy().to_string(), e))?;
        Ok((date, slug))
    }
}

impl Post {
    pub const DEFAULT_TEMPLATE: &str = "post";
    pub const READ_DIRECTORY: &str = "posts";
    pub const VALID_EXTENSIONS: [&str; 2] = ["md", "markdown"];
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostFrontmatter {
    pub title: String,
    pub template: Option<String>,
}

impl Render for Post {
    type FileType = PostFile;

    fn read_directory() -> String {
        Post::READ_DIRECTORY.to_string()
    }

    fn to_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("title", &self.title);
        context.insert("path", &self.output_path);
        context.insert("url", &self.url);
        context.insert("date", &self.date);
        context.insert("body", &self.content);
        context.insert("slug", &self.slug);
        context
    }

    fn from_content(post_file: PostFile, content: &str) -> Result<Self, RenderError> {
        let page = parser::parse_markdown(content).map_err(RenderError::Markdown)?;
        let res: PostFrontmatter = page.frontmatter.try_into().map_err(|e| {
            RenderError::ParseFrontmatter(format!(
                "frontmatter for {:?}: {:?}",
                post_file.url,
                e.to_string()
            ))
        })?;

        let mut template = res.template.unwrap_or(Post::DEFAULT_TEMPLATE.to_string());
        template.push_str(".html");

        Ok(Post {
            input_path: post_file.input_path.clone(),
            output_path: post_file.output_path.clone(),
            title: res.title,
            template,
            content: page.body,
            date: post_file.date,
            url: post_file.url.clone(),
            slug: post_file.slug.clone(),
        })
    }

    fn render(
        &self,
        templates: &Tera,
        output_dir: &Path,
        posts: &[Post],
    ) -> Result<(), RenderError> {
        let mut context = self.to_context();
        context.insert("posts", &posts);
        let output = templates
            .render(&self.template, &context)
            .map_err(RenderError::Tera)?;
        let output_path = output_dir.join(self.output_path.clone());
        let parent = output_path
            .parent()
            .ok_or(RenderError::CreateDir(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no parent directory",
            )))?;

        fs::create_dir_all(parent).map_err(RenderError::CreateDir)?;
        fs::write(&output_path, output).map_err(RenderError::WriteFile)?;
        Ok(())
    }
}
