//! Parse a markdown file with TOML frontmatter
use tera::Context;
use thiserror::Error;
type Frontmatter = toml::Value;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum Error {
    #[error("invalid frontmatter")]
    InvalidFrontmatter(String),
    #[error("content before frontmatter")]
    ContentBeforeFrontmatter(String),
}

#[derive(Debug)]
pub struct FrontmatterAndBody {
    pub frontmatter: Frontmatter,
    pub body: String,
    pub excerpt: String,
}

/// Convert to a Tera Context
impl From<FrontmatterAndBody> for Context {
    fn from(page: FrontmatterAndBody) -> Self {
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

pub fn extract_frontmatter(markdown: &str) -> Result<(Frontmatter, String), Error> {
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
    Ok((frontmatter, body))
}

pub fn parse_markdown(markdown: &str) -> Result<FrontmatterAndBody, Error> {
    let ts = ThemeSet::load_defaults();
    let theme = ts.themes.get("InspiredGitHub").unwrap();
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let (frontmatter, body) = extract_frontmatter(markdown)?;
    let events = pulldown_cmark::Parser::new(&body);
    let mut highlighted_events = vec![];
    let mut excerpt_events = vec![];
    let mut still_excerpting = true;
    let mut in_codeblock = false;
    let mut codeblock_contents = String::new();
    let mut syntax_extension = String::new();
    let default_syntax = syntax_set.find_syntax_plain_text();

    for event in events {
        println!("{:?}", event);
        match event.clone() {
            pulldown_cmark::Event::Html(html) => {
                if &html.to_string() == "<!--more-->\n" {
                    println!("EXCERPT ENDED!!");
                    still_excerpting = false;
                }
            }
            // Start collecting codeblock contents
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(kind)) => {
                match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(language) => {
                        println!("found codeblock. language = {:?}", language);
                        syntax_extension = language.to_string();
                    }
                    pulldown_cmark::CodeBlockKind::Indented => {
                        println!("found indented codeblock");
                    }
                }
                in_codeblock = true;
                codeblock_contents = String::new();
            }
            // End of a codeblock. Highlight the codeblock and add it to the highlighted events
            pulldown_cmark::Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                in_codeblock = false;
                let syntax = syntax_set
                    .find_syntax_by_token(&syntax_extension)
                    .unwrap_or(default_syntax);
                let highlighted = syntect::html::highlighted_html_for_string(
                    &codeblock_contents,
                    &syntax_set,
                    syntax,
                    theme,
                )
                .unwrap_or(codeblock_contents.clone());
                highlighted_events.push(pulldown_cmark::Event::Html(highlighted.clone().into()));
                if still_excerpting {
                    excerpt_events.push(pulldown_cmark::Event::Html(highlighted.into()));
                }
            }
            pulldown_cmark::Event::Text(text) => {
                if in_codeblock {
                    codeblock_contents.push_str(&text);
                } else {
                    highlighted_events.push(event.clone());
                    if still_excerpting {
                        excerpt_events.push(event.clone());
                    }
                }
            }
            _ => {
                highlighted_events.push(event.clone());
                if still_excerpting {
                    excerpt_events.push(event.clone());
                }
            }
        }
    }
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, highlighted_events.into_iter());
    let mut excerpt_html = String::new();
    pulldown_cmark::html::push_html(&mut excerpt_html, excerpt_events.into_iter());
    Ok(FrontmatterAndBody {
        frontmatter,
        body: html,
        excerpt: excerpt_html,
    })
}
