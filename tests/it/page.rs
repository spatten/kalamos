use kalamos::{page, page::PageFile, render::Render};
use simple_test_case::test_case;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;

macro_rules! page_file {
    ($slug:expr, $extension:expr, $filename:expr, $url:expr, $input_path:expr, $output_path:expr ) => {
        PageFile {
            slug: $slug.to_string(),
            extension: $extension.to_string(),
            filename: $filename.to_string(),
            url: PathBuf::from($url),
            input_path: PathBuf::from($input_path),
            output_path: PathBuf::from($output_path),
        }
    };
}

#[test_case("pages/about.md", page_file!("about", "md", "about.md", "/about.html", "pages/about.md", "about.html"); "about.md")]
#[test_case("pages/index.html", page_file!("index", "html", "index.html", "/index.html", "pages/index.html", "index.html"); "index.html")]
#[test]
fn test_page_file_from_path(input_path: &str, expected_page_file: PageFile) {
    let input_path = PathBuf::from(input_path);
    let page_file = PageFile::try_from(input_path).expect("should parse");
    assert_eq!(page_file, expected_page_file)
}

#[test_case(
  r#"
  <h1>{{title}}</h1>
  <div class="page">
  {% block content %}{{body|safe}}{% endblock content %}
  </div>"#,
  Path::new("pages/index.md"),
  r#"
  +++
  title = "Home Page"
  +++
  This is my home page.
  "#,
  r#"
  <h1>Home Page</h1>
  <div class="page">
  <p>This is my home page.</p>
</div>"#
; "simple frontmatter and md contents")]
#[test_case(
  r#"
  <h1>{{title}}</h1>
  <div class="page">{% block content %}{{body|safe}}{% endblock content %}</div>"#,
  Path::new("pages/index.html"),
  r#"
  +++
  title = "Home Page"
  +++
{% extends "default.html" %}
{% block content %}
  <p>This is my home page.</p>
{% endblock content %}
  "#,
  r#"
  <h1>Home Page</h1>
  <div class="page">
  <p>This is my home page.</p>
</div>"#; "simple frontmatter and html contents")]
#[test]
fn test_page_from_content(layout: &str, input_path: &Path, content: &str, expected: &str) {
    let mut tera = Tera::default();
    let output_dir = env::temp_dir();
    tera.add_raw_template("default.html", layout)
        .expect("should be able to add template");
    let page_file = page::PageFile::try_from(input_path.to_path_buf()).expect("should parse");
    let page = page::Page::from_content(page_file, content).expect("should parse");
    let posts = vec![];
    page.render(&tera, &output_dir, &posts)
        .expect("should render");
    let output_path = output_dir.join("index.html");
    let rendered = fs::read_to_string(&output_path).expect("should read");

    assert_eq!(rendered, expected);
}
