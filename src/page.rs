//! A page is a maud template and its markdown content
use tera::{self, Context, Tera};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("tera error")]
    Tera(tera::Error),
}
/// pass in a path containing glob patterns for the pages
/// Eg. load_templates("layouts/*.html")
pub fn load_templates(path: &str) -> Result<Tera, Error> {
    Tera::new(path).map_err(Error::Tera)
}

pub fn render_layout(templates: &Tera, layout: &str, context: &Context) -> Result<String, Error> {
    templates.render(layout, context).map_err(Error::Tera)
}
