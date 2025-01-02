use log::info;
use mime_guess;
use mime_guess::mime::Mime;
use regex::Regex;
use simple_server::{Server, StatusCode};
use std::path::Path;
use std::str::FromStr;

struct RequestInfo {
    content: Vec<u8>,
    status_code: StatusCode,
    mime_type: Mime,
}

const NOT_FOUND_PATH: &str = "404.html";

pub fn serve(input_dir: &Path, port: u16) -> Result<(), simple_server::Error> {
    // remove leading slash from request path, so that we can use it as a relative path
    let slash_remover = Regex::new(r"^/").expect("should be able to parse regex");

    let host = "127.0.0.1";
    info!("Serving from {:?}...", input_dir);
    let input_dir = input_dir.to_path_buf();
    let server = Server::new(move |request, mut response| {
        let request_path = request.uri().path();
        let request_path = slash_remover.replace(request_path, "").to_string();
        let RequestInfo {
            content,
            status_code,
            mime_type,
        } = file_content(&input_dir, &request_path)?;
        info!("Serving file: {}", &request_path);
        response.header("content_type", mime_type.essence_str());
        response.status(status_code);
        Ok(response.body(content)?)
    });

    server.listen(host, port.to_string().as_str());
}

fn file_content(root_path: &Path, path: &str) -> Result<RequestInfo, simple_server::Error> {
    let path = root_path.join(path);
    let path_with_index = path.join("index.html");
    match (&path.is_file(), &path_with_index.is_file()) {
        (true, _) => {
            let content = std::fs::read(&path)?;
            let mime_type = mime_guess::from_path(&path).first_or_text_plain();
            Ok(RequestInfo {
                content,
                status_code: StatusCode::OK,
                mime_type,
            })
        }
        (_, true) => {
            let content = std::fs::read(&path_with_index)?;
            let mime_type = mime_guess::from_path(&path_with_index).first_or_text_plain();
            Ok(RequestInfo {
                content,
                status_code: StatusCode::OK,
                mime_type,
            })
        }
        (_, _) => {
            let not_found_path = root_path.join(NOT_FOUND_PATH);
            let content = if not_found_path.exists() {
                std::fs::read(not_found_path)?
            } else {
                "<h1>404</h1><p>Not found!<p>".as_bytes().to_vec()
            };
            let mime_type = Mime::from_str("text/html").expect("should be able to parse mime type");
            Ok(RequestInfo {
                content,
                status_code: StatusCode::NOT_FOUND,
                mime_type,
            })
        }
    }
}
