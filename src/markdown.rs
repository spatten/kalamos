//! Parse a markdown file with TOML frontmatter
use tera::Context;
use thiserror::Error;
type Frontmatter = toml::Value;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum Error {
    #[error("invalid frontmatter")]
    InvalidFrontmatter(String),
    #[error("content before frontmatter")]
    ContentBeforeFrontmatter(String),
}

#[derive(Debug)]
pub struct Page {
    pub frontmatter: Frontmatter,
    pub body: String,
}

/// Convert a Page to a Tera Context
impl From<Page> for Context {
    fn from(page: Page) -> Self {
        let mut context = Context::new();
        context.insert(
            "title",
            page.frontmatter
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or_default(),
        );
        context.insert("body", &page.body);
        context.insert(
            "template",
            page.frontmatter
                .get("template")
                .unwrap_or(&toml::Value::String("default".to_string())),
        );
        let default_vars = toml::map::Map::new();
        let default_vars = toml::Value::Table(default_vars);
        let vars = page.frontmatter.get("vars").unwrap_or(&default_vars);
        context.insert("vars", &vars);
        context
    }
}

pub fn parse(markdown: &str) -> Result<Page, Error> {
    let mut sections = markdown.split("+++\n");
    // If there are less than 3 sections, there is no frontmatter,
    // so we just return the whole thing as the body.
    // If there are three or more sections, there is frontmatter,
    // so we parse the frontmatter and the body.
    // any further +++ lines are just part of the body
    let frontmatter: Frontmatter = if sections.clone().count() < 3 {
        toml::from_str("").expect("empty frontmatter should be valid")
    } else {
        // get rid of the first `+++` line
        let before = sections.next().expect("should have at least 3 sections");
        if !before.trim().is_empty() {
            return Err(Error::ContentBeforeFrontmatter(before.to_string()));
        }
        let frontmatter_content = sections.next().unwrap();
        toml::from_str(frontmatter_content).map_err(|e| Error::InvalidFrontmatter(e.to_string()))?
    };
    let body = sections
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("\n+++\n");
    let parser = pulldown_cmark::Parser::new(&body);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    Ok(Page {
        frontmatter,
        body: html,
    })
}
