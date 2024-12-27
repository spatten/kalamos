//! Tests for the markdown module
use kalamos::markdown;
use simple_test_case::test_case;

#[test_case("+++ title = \"Hello, world!\" +++ # Hello, world!", ("title = \"Hello, world!\"", "<h1>Hello, world!</h1>\n"); "simple frontmatter and post")]
#[test_case("# Hello, world!", ("", "<h1>Hello, world!</h1>\n"); "no frontmatter")]
#[test_case("+++ title = \"Hello, world!\" +++ # Hello, world!\n+++\ncontinuing", ("title = \"Hello, world!\"", "<h1>Hello, world!</h1>\n<p>+++</p>\n<p>continuing</p>\n"); "multiple plus-plus-plus lines")]
#[test]
fn test_parse(markdown: &str, expected: (&str, &str)) {
    let (frontmatter, body) = markdown::parse(markdown);
    assert_eq!(frontmatter, toml::from_str(expected.0).unwrap());
    assert_eq!(body, expected.1);
}
