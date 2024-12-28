use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::markdown;
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageFrontmatter {
    pub title: String,
    pub template: Option<String>,
}

impl Page {
    const DEFAULT_TEMPLATE: &str = "default";
    const READ_DIRECTORY: &str = "pages";
}

impl Render for Page {
    fn to_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("title", &self.title);
        context.insert("path", &self.path);
        context.insert("url", &self.path);
        context.insert("body", &self.content);
        context
    }

    fn from_file(root_path: &Path, path: &Path) -> Result<Self, RenderError> {
        let full_path = root_path.join(path);
        let content = fs::read_to_string(&full_path).map_err(RenderError::ReadFile)?;
        let page = markdown::parse(&content).map_err(RenderError::Markdown)?;
        let frontmatter: PageFrontmatter = page.frontmatter.try_into().map_err(|e| {
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
        let res = Self {
            path: path.to_path_buf(),
            title: frontmatter.title,
            template,
            content: page.body,
        };
        Ok(res)
    }

    fn render(
        &self,
        templates: &Tera,
        output_dir: &Path,
        posts: &[Post],
    ) -> Result<(), RenderError> {
        let mut context = self.to_context();
        context.insert("posts", posts);
        let output = templates
            .render(&self.template, &context)
            .map_err(RenderError::Tera)?;
        let relative_path = self.path.strip_prefix(Page::READ_DIRECTORY).unwrap();
        let output_path = output_dir.join(relative_path).with_extension("html");

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
}
