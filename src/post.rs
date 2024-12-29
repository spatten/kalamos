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
    /// path of the rendered file, relative to the root of the site
    /// 2024/12/28/my-post.html
    pub path: PathBuf,
    /// the title of the page
    pub title: String,
    /// the template to use to render the page
    pub template: String,
    /// The content of the page
    pub content: String,
    /// The date the post was published
    pub date: NaiveDate,
    /// The url of the post. This is path, but with a leading /
    pub url: PathBuf,
}

impl Post {
    const DEFAULT_TEMPLATE: &str = "post";
    const READ_DIRECTORY: &str = "posts";

    /// Extracts the date and slug from a file name
    /// The file name must be in the format YYYY-MM-DD-slug.md
    fn extract_date_and_slug(path: &Path) -> Result<(NaiveDate, String), RenderError> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PostFrontmatter {
    pub title: String,
    pub template: Option<String>,
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

    fn from_file(root_path: &Path, path: &Path) -> Result<Self, RenderError> {
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

        let path = path.to_path_buf();

        let relative_path = path.strip_prefix(Post::READ_DIRECTORY).unwrap();
        let (date, slug) = Post::extract_date_and_slug(relative_path)?;
        let output_path = PathBuf::from(date.year().to_string())
            .join(date.month().to_string())
            .join(slug)
            .with_extension("html");
        let url = PathBuf::from("/").join(&output_path);

        Ok(Post {
            path: output_path,
            title: res.title,
            template,
            content: page.body,
            date,
            url,
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
        let output_path = output_dir.join(&self.path);
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
