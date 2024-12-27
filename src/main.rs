use kalamos::markdown;

fn main() {
    println!("Hello, world!");

    let md = r#"
+++
title = "Hello, world!"
+++

# Hello, world!
This is my first post.
"#;

    let (frontmatter, body) = markdown::parser::parse_markdown(md);
    println!("frontmatter: {:?}", frontmatter);
    println!("body: {}", body);
}
