# Kalamos: Static site generator in Rust

## Name
kalamos - https://en.wikipedia.org/wiki/Kalamos
## Syntax highlighting
## commands you need
s new
s serve <--- incrementally generate. Should be fast when you're editing
s generate <--- build the whole thing from scratch
s deploy (?) <--- deploy to S3 by default?
## Configuration
config.toml
## Is this for blogs?
Is there an opinion in the software that biases it towards a blog?
## directory structure
Is there a directory structure that we insist on?
Can you just put plain HTML in?
example from [aurora](https://github.com/capjamesg/aurora):
pages
  _layouts
  posts
  pages
  assets
config.py

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

