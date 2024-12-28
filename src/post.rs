use chrono::NaiveDate;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
use toml::Value;

use crate::markdown;
use crate::render::Error as RenderError;
use crate::render::Render;

#[derive(Debug, Serialize)]
pub struct Post {
    /// A relative path to the file, relative to the root of the site
    pub path: PathBuf,
    /// the title of the page
    pub title: String,
    /// the template to use to render the page
    pub template: String,
    /// The content of the page
    pub content: String,
    /// The date the post was published
    pub date: NaiveDate,
}

impl Render for Post {
    fn to_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("title", &self.title);
        context.insert("path", &self.path);
        context.insert("url", &self.path);
        context.insert("date", &self.date);
        context.insert("body", &self.content);
        context
    }

    // TODO: use the date to generate the output path
    fn output_path(&self, output_dir: &Path) -> PathBuf {
        output_dir.join(&self.path)
    }

    fn from_file(root_path: &Path, path: &Path) -> Result<Box<Self>, RenderError> {
        println!("post::from_files for path {:?}", path);
        let full_path = root_path.join(path);
        let content = fs::read_to_string(&full_path).map_err(RenderError::ReadFile)?;
        let page = markdown::parse(&content).map_err(RenderError::Markdown)?;
        let title = page
            .frontmatter
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let template = page
            .frontmatter
            .get("template")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        let date = page
            .frontmatter
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or(RenderError::MissingField("date".to_string()))?;
        let date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| RenderError::ParseDate(date.to_string(), e))?;
        Ok(Box::new(Self {
            path: path.to_path_buf(),
            title,
            template,
            content: page.body,
            date,
        }))
    }

    fn render(&self, templates: &Tera, output_dir: &Path) -> Result<(), RenderError> {
        let output = templates
            // TODO: use the template from the post
            // .render(&self.template, &self.to_context())
            .render("post.html", &self.to_context())
            .map_err(RenderError::Tera)?;
        let output_path = output_dir.join(&self.path).with_extension("html");
        let parent = output_path
            .parent()
            .ok_or(RenderError::CreateDir(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no parent directory",
            )))?;
        fs::create_dir_all(parent).map_err(RenderError::CreateDir)?;
        println!("writing post to {:?}", output_path);
        fs::write(&output_path, output).map_err(RenderError::WriteFile)?;
        Ok(())
    }
}
