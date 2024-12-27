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
    let res = markdown::parse(md);
    match res {
        Ok(page) => {
            println!("frontmatter: {:?}", page.frontmatter);
            println!("body:\n======\n{}======\n", page.body);
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
}
