use crate::web_server;

use std::{fs, path::PathBuf};
use web_server::types::{Nested, Request, Response};

/// Returns the path to a project's config file.
pub fn get_project_config_file_path(project_name: &str) -> PathBuf {
  PathBuf::from(format!("database/projects/{}.json", project_name))
}

/// Returns a closure that saves a project's config.
pub fn save_config() -> impl Fn(Request) -> Response {
  |request: Request| {
    let project_name = request.params.get("name").unwrap();
    let file_path = get_project_config_file_path(project_name);

    if request.method == "POST" && file_path.exists() {
      let mut body = Nested::new();
      body.insert_string("error".to_string(), "Project already exists.".to_string());

      return Response::json(400, body, None);
    } else if request.method == "PUT" && !file_path.exists() {
      let mut body = Nested::new();
      body.insert_string("error".to_string(), "Project does not exist.".to_string());

      return Response::json(400, body, None);
    }

    fs::write(file_path, request.body).unwrap();

    let mut body = Nested::new();
    body.insert_string("result".to_string(), "ok".to_string());
    Response::json(200, body, None)
  }
}
