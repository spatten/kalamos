use kalamos::{markdown, page};
use simple_test_case::test_case;
use tera::{Context, Tera};

#[test_case(
  r#"
  <h1>{{title}}</h1>
  <div class="post">
  {{body|safe}}
  </div>
  "#,
  r#"
  +++
  title = "Hello, world!"
  +++
  This is a *test* post.
  "#,
  r#"
  <h1>Hello, world!</h1>
  <div class="post">
  <p>This is a <em>test</em> post.</p>

  </div>
  "#
; "simple frontmatter and post")]
#[test]
fn test_page_from_content(layout: &str, content: &str, expected: &str) {
    let mut tera = Tera::default();
    tera.add_raw_template("hello.html", layout)
        .expect("should be able to add template");

    let parser::FrontmatterAndBody { frontmatter, body } =
        parser::parse_markdown(content).expect("should parse");
    let context = Context::from(parser::FrontmatterAndBody { frontmatter, body });
    let rendered = page::render_layout(&tera, "hello.html", &context).expect("should render");
    assert_eq!(rendered, expected);
}
