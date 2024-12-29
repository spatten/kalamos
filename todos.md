
# Kalamos: Static site generator in Rust
## TODO Makefile
### install
cargo install --path .
## TODO Use pandoc to convert current textile to md
## DONE Name
kalamos - https://en.wikipedia.org/wiki/Kalamos
## Syntax highlighting
## commands you need
kalamos new
kalamos serve <--- incrementally generate. Should be fast when you're editing
kalamos generate <--- build the whole thing from scratch
kalamos deploy (?) <--- deploy to S3 by default?
## Configuration
config.toml

- site-wide variables
- deploy method and config
-
## Is this for blogs?
Is there an opinion in the software that biases it towards a blog?
## directory structure
Is there a directory structure that we insist on?
Can you just put plain HTML in?
example from [aurora](https://github.com/capjamesg/aurora):
pages
  layouts
  posts
  pages
  assets
config.py

Everything in layouts is a layout used to generate posts and pages.
Everything in posts has a type of Post. Only .md files are supported. Default layout is post.
Everything in pages is a type of Page. .md, .html, .xhtml and .xml files are supported. Default layout is default.
Everything in assets is just copied to the assets folder in the generated site

Differences between posts and pages:
- default layout
- post requires date, title, url
- page requires title
- post can have an excerpt
- posts are sorted and returned in the posts variable, which is sent to all other pages and can be used in templates

Post and Page implement the Render trait

Render:
- to_context (gets variables, sets default template if required)
- render
- path (has date in it for posts, just the relative path for pages)

## first steps
generate a single page with maud template pulling in a post body
## Markdown syntax
Do we need something like Jekyll's template syntax?

Yes, let's do that. The current standard seems to be like this. That's TOML syntax inside of `+++`. [Zola](https://www.getzola.org/documentation/getting-started/overview/) and [Hugo](https://gohugo.io/getting-started/quick-start/) use this.

```
+++
title = "List of blog posts"
sort_by = "date"
template = "blog.html"
page_template = "blog-page.html"
+++
```
## Markdown parser
See last example of [Maud's render-trait page](https://maud.lambda.xyz/render-trait.html) for an example of using pulldown-cmark with Maud
### Comrak
https://github.com/kivikakk/comrak
### pulldown-cmark
https://crates.io/crates/pulldown-cmark
## template language
https://www.reddit.com/r/rust/comments/1fc2mic/best_templating_engine_for_rust/
I'm going to go with Maud or hypertext (using the Maud syntax).

The only real difference is, supposedly, that hypertext is more efficient with nested templates

Actually, it looks like there's no way to pass a string into maude, so this does not work
### hypertext
https://github.com/vidhanio/hypertext
### rsx
https://github.com/victorporof/rsx
### Maud
https://maud.lambda.xyz/
### markup.rs
https://github.com/utkarshkukreti/markup.rs
### Liquid
https://github.com/cobalt-org/liquid-rust
### Handlebars
https://github.com/sunng87/handlebars-rust
### Jinja
#### Tera
https://keats.github.io/tera/
#### Askama
https://github.com/rinja-rs/askama
requires templates during compile time

