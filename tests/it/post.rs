use chrono::NaiveDate;
use kalamos::{
    post::{Post, PostFile},
    render::Render,
};
use simple_test_case::test_case;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;

macro_rules! post_file {
    ($date:expr, $slug:expr, $extension:expr, $filename:expr, $url:expr, $input_path:expr, $output_path:expr ) => {
        PostFile {
            date: NaiveDate::parse_from_str($date, "%Y-%m-%d").expect("should parse date"),
            slug: $slug.to_string(),
            extension: $extension.to_string(),
            url: PathBuf::from($url),
            input_path: PathBuf::from($input_path),
            output_path: PathBuf::from($output_path),
        }
    };
}

#[test_case("posts/2024-12-01-first.md", post_file!("2024-12-01", "first", "md", "first.md", "/2024/12/first.html", "posts/2024-12-01-first.md", "2024/12/first.html"); "2024-12-01-first.md")]
#[test]
fn test_post_from_file(input_path: &str, expected_post_file: PostFile) {
    let input_path = PathBuf::from(input_path);
    let post_file = PostFile::try_from(input_path).expect("should parse");
    assert_eq!(post_file, expected_post_file)
}

#[test_case(
  r#"
  <h1>{{title}}</h1>
  <div class="post">
  {{body|safe}}
  </div>
  "#,
  Path::new("posts/2024-12-01-first.md"),
  r#"
  +++
  title = "First Post"
  +++
  This is my first post.
  "#,
  r#"
  <h1>First Post</h1>
  <div class="post">
  <p>This is my first post.</p>

  </div>
  "#
; "simple frontmatter and post")]
#[test]
fn test_post_from_content(layout: &str, input_path: &Path, content: &str, expected: &str) {
    let mut tera = Tera::default();
    let output_dir = env::temp_dir();
    tera.add_raw_template("post.html", layout)
        .expect("should be able to add template");
    let page_file = PostFile::try_from(input_path.to_path_buf()).expect("should parse");
    let page = Post::from_content(page_file, content).expect("should parse");
    let posts = vec![];
    page.render(&tera, &output_dir, &posts)
        .expect("should render");
    let output_path = output_dir.join("2024/12/first.html");
    println!("output_path: {:?}", output_path);
    let rendered = fs::read_to_string(&output_path).expect("should read");

    assert_eq!(rendered, expected);
}
