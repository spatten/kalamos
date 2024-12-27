//! Parse a markdown file with TOML frontmatter
type Frontmatter = toml::Value;

pub fn parse(markdown: &str) -> (Frontmatter, String) {
    let mut sections = markdown.split("+++");
    let frontmatter: Frontmatter;
    let body: String;
    // If there are less than 3 sections, there is no frontmatter,
    // so we just return the whole thing as the body.
    // If there are more than 3 sections, there is frontmatter,
    // so we parse the frontmatter and the body.
    if sections.clone().count() < 3 {
        frontmatter = toml::from_str("").unwrap();
        body = sections.map(|s| s.to_string()).collect::<String>();
        return (frontmatter, body);
    } else {
        let _ = sections.next().unwrap();
        let frontmatter_content = sections.next().unwrap();
        frontmatter = toml::from_str(frontmatter_content).unwrap();
        body = sections.map(|s| s.to_string()).collect::<String>();
    }
    let parser = pulldown_cmark::Parser::new(&body);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    (frontmatter, html)
}
