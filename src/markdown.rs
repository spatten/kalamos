//! Parse a markdown file with TOML frontmatter
use thiserror::Error;
type Frontmatter = toml::Value;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid frontmatter")]
    InvalidFrontmatter(String),
}

pub fn parse(markdown: &str) -> Result<(Frontmatter, String), Error> {
    let mut sections = markdown.split("+++");
    // If there are less than 3 sections, there is no frontmatter,
    // so we just return the whole thing as the body.
    // If there are more than 3 sections, there is frontmatter,
    // so we parse the frontmatter and the body.
    let frontmatter: Frontmatter = if sections.clone().count() < 3 {
        toml::from_str("").expect("empty frontmatter should be valid")
    } else {
        // get rid of the first `+++` line
        let _ = sections.next().unwrap();
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
    Ok((frontmatter, html))
}
