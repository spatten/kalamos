use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::markdown;
use crate::render::Error as RenderError;
use crate::render::Render;

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

impl Render for Page {
    fn to_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("title", &self.title);
        context.insert("path", &self.path);
        context.insert("url", &self.path);
        context.insert("body", &self.content);
        context
    }

    fn output_path(&self, output_dir: &Path) -> PathBuf {
        output_dir.join(&self.path)
    }

    fn from_file(root_path: &Path, path: &Path) -> Result<Box<Self>, RenderError> {
        let full_path = root_path.join(path);
        let content = fs::read_to_string(&full_path).map_err(RenderError::ReadFile)?;
        let page = markdown::parse(&content).map_err(RenderError::Markdown)?;
        let title = page
            .frontmatter
            .get("title")
            .unwrap_or(&toml::Value::String("".to_string()))
            .to_string();
        let template = page
            .frontmatter
            .get("template")
            .unwrap_or(&toml::Value::String("default".to_string()))
            .to_string();
        Ok(Box::new(Self {
            path: path.to_path_buf(),
            title,
            template,
            content: page.body,
        }))
    }

    fn render(&self, templates: &Tera, output_dir: &Path) -> Result<(), RenderError> {
        let output = templates
            // TODO: use the template from the page
            // .render(&self.template, &self.to_context())
            .render("default.html", &self.to_context())
            .map_err(RenderError::Tera)?;
        let output_path = output_dir.join(&self.path).with_extension("html");
        println!("writing page to {:?}", output_path);
        fs::write(&output_path, output).map_err(RenderError::WriteFile)?;
        Ok(())
    }
}
