use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::parser;
use crate::post::Post;
use crate::render::Render;
use crate::render::{Error as RenderError, RenderableFromPath};

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    /// The relative path to the input file, relative to the root of the site
    pub path: PathBuf,
    /// A relative path to the rendered file, relative to the root of the site
    pub url: PathBuf,
    /// the title of the page
    pub title: String,
    /// the template to use to render the page
    pub template: String,
    /// The content of the page
    pub content: String,
    /// The page slug
    pub slug: String,
    /// The extension of the input file
    pub extension: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageFile {
    pub slug: String,
    pub extension: String,
    pub filename: String,
    /// The url of the file, relative to the site root
    pub url: PathBuf,
    /// The relative path to the file, relative to the root of the site
    /// The extension will be .md for a markdown file, whereas the url with be .html
    pub path: PathBuf,
}

impl TryFrom<PathBuf> for PageFile {
    type Error = RenderError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let slug = path
            .with_extension("")
            .file_name()
            .ok_or(RenderError::Path(path.clone()))?
            .to_str()
            .ok_or(RenderError::Path(path.clone()))?
            .to_string();
        let extension = &path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if !Page::VALID_EXTENSIONS.contains(extension) {
            return Err(RenderError::Path(path.to_path_buf()));
        }
        let url_extension = if Page::extension_is_markdown(extension) {
            "html"
        } else {
            extension
        };
        Ok(Self {
            slug,
            extension: extension.to_string(),
            filename: path.file_name().unwrap().to_str().unwrap().to_string(),
            url: path.to_path_buf().with_extension(url_extension),
            path: path.to_path_buf(),
        })
    }
}

impl RenderableFromPath for PageFile {
    fn url(&self) -> PathBuf {
        self.url.clone()
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageFrontmatter {
    pub title: String,
    pub template: Option<String>,
}

impl Page {
    pub const DEFAULT_TEMPLATE: &str = "default";
    pub const READ_DIRECTORY: &str = "pages";
    pub const VALID_EXTENSIONS: [&str; 4] = ["md", "markdown", "html", "xml"];

    fn extension_is_markdown(extension: &str) -> bool {
        extension == "md" || extension == "markdown"
    }

    fn is_markdown(&self) -> bool {
        Self::extension_is_markdown(&self.extension)
    }

    fn from_non_markdown_content(content: &str, page_file: &PageFile) -> Result<Self, RenderError> {
        let (frontmatter, body) =
            parser::extract_frontmatter(content).map_err(RenderError::Markdown)?;

        let frontmatter: PageFrontmatter = frontmatter.try_into().map_err(|e| {
            RenderError::ParseFrontmatter(format!(
                "frontmatter for {:?}: {:?}",
                page_file.path,
                e.to_string()
            ))
        })?;

        let mut template = Page::DEFAULT_TEMPLATE.to_string();
        template.push_str(".html");
        Ok(Self {
            path: page_file.path.to_path_buf(),
            url: page_file.url.to_path_buf(),
            title: frontmatter.title,
            template,
            content: body,
            slug: page_file.slug.clone(),
            extension: page_file.extension.to_string(),
        })
    }

    fn from_markdown_content(content: &str, page_file: &PageFile) -> Result<Self, RenderError> {
        let parsed = parser::parse_markdown(content).map_err(RenderError::Markdown)?;
        let frontmatter: PageFrontmatter = parsed.frontmatter.try_into().map_err(|e| {
            RenderError::ParseFrontmatter(format!(
                "frontmatter for {:?}: {:?}",
                page_file.path,
                e.to_string()
            ))
        })?;
        let mut template = frontmatter
            .template
            .unwrap_or(Page::DEFAULT_TEMPLATE.to_string());
        template.push_str(".html");

        Ok(Self {
            path: page_file.path.to_path_buf(),
            url: page_file.url.to_path_buf(),
            title: frontmatter.title,
            template,
            content: parsed.body,
            slug: page_file.slug.clone(),
            extension: page_file.extension.to_string(),
        })
    }
}

impl Render for Page {
    type FileType = PageFile;

    fn to_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("title", &self.title);
        context.insert("path", &self.path);
        context.insert("url", &self.url);
        context.insert("body", &self.content);
        context.insert("slug", &self.slug);
        context.insert("current_date", &Utc::now().date_naive());
        context
    }

    fn from_content(page_file: PageFile, content: &str) -> Result<Self, RenderError> {
        let page = if !Self::extension_is_markdown(&page_file.extension) {
            Self::from_non_markdown_content(content, &page_file)?
        } else {
            Self::from_markdown_content(content, &page_file)?
        };

        Ok(page)
    }

    fn render(
        &self,
        templates: &Tera,
        output_dir: &Path,
        posts: &[Post],
    ) -> Result<(), RenderError> {
        let mut context = self.to_context();
        context.insert("posts", posts);

        let output = if self.is_markdown() {
            templates
                .render(&self.template, &context)
                .map_err(RenderError::Tera)?
        } else {
            let template = self
                .path
                .file_name()
                .ok_or(RenderError::Path(self.path.to_path_buf()))?
                .to_str()
                .ok_or(RenderError::Path(self.path.to_path_buf()))?;
            let mut templates = templates.clone();
            templates
                .add_raw_template(template, &self.content)
                .map_err(RenderError::Tera)?;
            templates
                .render(template, &context)
                .map_err(RenderError::Tera)?
        };

        let relative_path = self.path.strip_prefix(Self::read_directory()).unwrap();
        let output_path = if self.is_markdown() {
            output_dir.join(relative_path).with_extension("html")
        } else {
            output_dir.join(relative_path)
        };

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

    fn read_directory() -> String {
        Page::READ_DIRECTORY.to_string()
    }
}
