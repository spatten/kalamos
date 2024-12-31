use crate::support;
use insta::assert_yaml_snapshot;
use kalamos::render;
use std::{env, fs, path::Path};

#[test]
fn test_render_dir() {
    let temp_dir = env::temp_dir();
    let root_dir = Path::new("tests/it/testdata/simple_site");
    let output_dir = temp_dir.join("kalamos_test_output");
    fs::remove_dir_all(&output_dir).expect("should remove output dir");
    render::render_dir(root_dir, &output_dir).expect("should render");
    let output_content = support::dir_to_yaml(&output_dir).expect("should generate yaml");
    assert_yaml_snapshot!(output_content);
}
