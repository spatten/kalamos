use kalamos::markdown;

fn main() {
    let md = r#"
+++
title = "Hello, world!"
draft = true
+++
# Hello, world!
This is my first post.
"#;

    let markdown::Page { frontmatter, body } = markdown::parse(md).expect("should parse");
    println!("frontmatter: {:?}", frontmatter);
    println!("body: {}", body);
}
