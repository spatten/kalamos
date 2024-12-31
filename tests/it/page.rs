use kalamos::{page, render::Render};
use simple_test_case::test_case;
use std::env;
use std::fs;
use std::path::Path;
use tera::Tera;

#[test_case(
  r#"
  <h1>{{title}}</h1>
  <div class="page">
  {{body|safe}}
  </div>
  "#,
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

  </div>
  "#
; "simple frontmatter and post")]
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
    println!("output_path: {:?}", output_path);
    let rendered = fs::read_to_string(&output_path).expect("should read");

    assert_eq!(rendered, expected);
}
