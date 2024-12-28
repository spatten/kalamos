use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::markdown;
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

impl Render for Page {
    fn to_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("title", &self.title);
        context.insert("path", &self.path);
        context.insert("url", &self.path);
        context.insert("body", &self.content);
        context
    }

    fn from_file(root_path: &Path, path: &Path) -> Result<Box<Self>, RenderError> {
        let full_path = root_path.join(path);
        let content = fs::read_to_string(&full_path).map_err(RenderError::ReadFile)?;
        let page = markdown::parse(&content).map_err(RenderError::Markdown)?;
        let default_template = "default".to_string();
        let frontmatter: PageFrontmatter = page.frontmatter.try_into().map_err(|e| {
            RenderError::ParseFrontmatter(format!(
                "frontmatter for {:?}: {:?}",
                path,
                e.to_string()
            ))
        })?;
        let res = Self {
            path: path.to_path_buf(),
            title: frontmatter.title,
            template: frontmatter.template.unwrap_or(default_template),
            content: page.body,
        };
        Ok(Box::new(res))
    }

    fn render(&self, templates: &Tera, output_dir: &Path) -> Result<(), RenderError> {
        let output = templates
            // TODO: use the template from the page
            // .render(&self.template, &self.to_context())
            .render("default.html", &self.to_context())
            .map_err(RenderError::Tera)?;
        let relative_path = self.path.strip_prefix("pages").unwrap();
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
