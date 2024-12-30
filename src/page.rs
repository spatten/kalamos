use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::parser;
use crate::post::Post;
use crate::render::Error as RenderError;
use crate::render::Render;

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    /// A relative path to the file, relative to the root of the site
    pub path: PathBuf,
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

    fn from_content(path: &Path, extension: &str, content: &str) -> Result<Self, RenderError> {
        let slug = path
            .with_extension("")
            .file_name()
            .ok_or(RenderError::Path(path.to_path_buf()))?
            .to_str()
            .ok_or(RenderError::Path(path.to_path_buf()))?
            .to_string();
        let page = if !Self::extension_is_markdown(extension) {
            Self::from_non_markdown_content(content, path, slug, extension)?
        } else {
            Self::from_markdown_content(content, path, slug, extension)?
        };

        Ok(page)
    }

    fn from_non_markdown_content(
        content: &str,
        path: &Path,
        slug: String,
        extension: &str,
    ) -> Result<Self, RenderError> {
        let (frontmatter, body) =
            parser::extract_frontmatter(content).map_err(RenderError::Markdown)?;

        let frontmatter: PageFrontmatter = frontmatter.try_into().map_err(|e| {
            RenderError::ParseFrontmatter(format!(
                "frontmatter for {:?}: {:?}",
                path,
                e.to_string()
            ))
        })?;

        let mut template = Page::DEFAULT_TEMPLATE.to_string();
        template.push_str(".html");
        Ok(Self {
            path: path.to_path_buf(),
            title: frontmatter.title,
            template,
            content: body,
            slug: slug.clone(),
            extension: extension.to_string(),
        })
    }

    fn from_markdown_content(
        content: &str,
        path: &Path,
        slug: String,
        extension: &str,
    ) -> Result<Self, RenderError> {
        let parsed = parser::parse_markdown(content).map_err(RenderError::Markdown)?;
        let frontmatter: PageFrontmatter = parsed.frontmatter.try_into().map_err(|e| {
            RenderError::ParseFrontmatter(format!(
                "frontmatter for {:?}: {:?}",
                path,
                e.to_string()
            ))
        })?;
        let mut template = frontmatter
            .template
            .unwrap_or(Page::DEFAULT_TEMPLATE.to_string());
        template.push_str(".html");

        Ok(Self {
            path: path.to_path_buf(),
            title: frontmatter.title,
            template,
            content: parsed.body,
            slug,
            extension: extension.to_string(),
        })
    }
}

impl Render for Page {
    fn to_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("title", &self.title);
        context.insert("path", &self.path);
        context.insert("url", &self.path);
        context.insert("body", &self.content);
        context.insert("slug", &self.slug);
        context.insert("current_date", &Utc::now().date_naive());
        context
    }

    fn from_file(root_path: &Path, path: &Path) -> Result<Option<Self>, RenderError> {
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if !Self::VALID_EXTENSIONS.contains(&extension) {
            return Ok(None);
        }
        let full_path = root_path.join(path);
        let content = fs::read_to_string(&full_path).map_err(RenderError::ReadFile)?;
        let content = Self::from_content(path, extension, &content)?;
        Ok(Some(content))
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
        println!("writing page to {:?}", output_path);
        fs::write(&output_path, output).map_err(RenderError::WriteFile)?;
        Ok(())
    }

    fn read_directory() -> String {
        Page::READ_DIRECTORY.to_string()
    }
}
