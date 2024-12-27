//! Parse a markdown file with TOML frontmatter

type Frontmatter = toml::Value;

pub fn parse_markdown(markdown: &str) -> (Frontmatter, String) {
    let mut sections = markdown.split("+++");
    // If there are less than 3 sections, there is no frontmatter,
    // so we just return the whole thing as the body.
    if sections.clone().count() < 3 {
        let frontmatter = toml::from_str("").unwrap();
        let body = sections.map(|s| s.to_string()).collect::<String>();
        return (frontmatter, body);
    }
    let _ = sections.next().unwrap();
    let frontmatter = sections.next().unwrap();
    let frontmatter = toml::from_str(frontmatter).unwrap();
    let body = sections.map(|s| s.to_string()).collect::<String>();
    let parser = pulldown_cmark::Parser::new(&body);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    (frontmatter, html)
}
