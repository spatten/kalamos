use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::markdown;
use crate::render::Error as RenderError;
use crate::render::Render;

#[derive(Debug, Serialize, Deserialize)]
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

impl Post {
    const DEFAULT_TEMPLATE: &str = "post";
    const READ_DIRECTORY: &str = "posts";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostFrontmatter {
    pub title: String,
    pub template: Option<String>,
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

    fn from_file(root_path: &Path, path: &Path) -> Result<Box<Self>, RenderError> {
        println!("post::from_files for path {:?}", path);
        let full_path = root_path.join(path);
        let content = fs::read_to_string(&full_path).map_err(RenderError::ReadFile)?;
        let page = markdown::parse(&content).map_err(RenderError::Markdown)?;
        let res: PostFrontmatter = page.frontmatter.try_into().map_err(|e| {
            RenderError::ParseFrontmatter(format!(
                "frontmatter for {:?}: {:?}",
                path,
                e.to_string()
            ))
        })?;

        let mut template = res.template.unwrap_or(Post::DEFAULT_TEMPLATE.to_string());
        template.push_str(".html");
        Ok(Box::new(Post {
            path: path.to_path_buf(),
            title: res.title,
            template,
            content: page.body,
            date: res.date,
        }))
    }

    fn render(&self, templates: &Tera, output_dir: &Path) -> Result<(), RenderError> {
        let output = templates
            .render(&self.template, &self.to_context())
            .map_err(RenderError::Tera)?;
        let relative_path = self.path.strip_prefix(Post::READ_DIRECTORY).unwrap();
        let output_path = output_dir
            .join(self.date.year().to_string())
            .join(self.date.month().to_string())
            .join(self.date.day().to_string())
            .join(relative_path)
            .with_extension("html");
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
