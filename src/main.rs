use kalamos::markdown;

fn main() {
    println!("Hello, world!");

    let md = r#"
# Hello, world!
This is my first post.
"#;

    let (frontmatter, body) = markdown::parse(md);
    println!("frontmatter: {:?}", frontmatter);
    println!("body: {}", body);
}
