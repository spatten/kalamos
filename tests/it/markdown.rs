//! Tests for the markdown module
use kalamos::markdown;
use simple_test_case::test_case;

#[test_case("+++\ntitle = \"Hello, world!\"\n+++\n# Hello, world!", ("title = \"Hello, world!\"", "<h1>Hello, world!</h1>\n"); "simple frontmatter and post")]
#[test_case(
  r#"
  +++
  title = "Hello, world!"
  date = 2024-01-01
  draft = false
  +++
  # Hello, world!
"#,
  (
    r#"
    title = "Hello, world!"
    date = 2024-01-01
    draft = false
    "#,
    "<h1>Hello, world!</h1>\n");
    "moderate frontmatter and post"
  )]
#[test_case(
  r#"




  +++
  title = "Hello, world!"
  +++
  # Hello, world!
"#,
  (
    r#"title = "Hello, world!""#,
    "<h1>Hello, world!</h1>\n");
    "whitespace before frontmatter"
  )]
#[test_case("# Hello, world!", ("", "<h1>Hello, world!</h1>\n"); "no frontmatter")]
#[test_case("+++\ntitle = \"Hello, world!\"\n+++\n# Hello, world!\n+++\n\ncontinuing", ("title = \"Hello, world!\"", "<h1>Hello, world!</h1>\n<p>+++</p>\n<p>continuing</p>\n"); "multiple plus-plus-plus lines")]
#[test]
fn test_parse_with_valid_frontmatter(markdown: &str, expected: (&str, &str)) {
    let parser::FrontmatterAndBody { frontmatter, body } =
        parser::parse_markdown(markdown).expect("should parse");
    assert_eq!(frontmatter, toml::from_str(expected.0).unwrap());
    assert_eq!(body, expected.1);
}

#[test_case(
    "+++\ntitle+++\n# Hello, world!\n+++\ncontinuing"; "invalid toml in frontmatter") ]
#[test_case("before the frontmatter\n+++\ntitle = \"Hello, world!\"\n+++\n# Hello, world!\n+++\ncontinuing"; "content before frontmatter")]
#[test]
fn test_parse_with_invalid_frontmatter(markdown: &str) {
    let res = parser::parse_markdown(markdown);
    assert!(res.is_err());
}
