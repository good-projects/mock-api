use crate::{helpers, web_server};
use serde_json::Value;
use std::{collections::HashMap, fs, fs::read_to_string};
use web_server::types::{Nested, Request, Response};

/// Returns a closure that saves a project's config.
pub fn save_config() -> impl Fn(Request) -> Response {
  |request: Request| {
    let file_path = helpers::config_file_path_from_request(&request);

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

/// Returns a closure that mocks a request of a given project.
pub fn mock_request() -> impl Fn(Request) -> Response {
  |request: Request| {
    let config = helpers::get_project_config_file_path(request.matches.get(0).unwrap());
    if !config.exists() {
      let mut body = Nested::new();
      body.insert_string("error".to_string(), "Project does not exist.".to_string());
      return Response::json(400, body, None);
    }

    let config = read_to_string(config).unwrap();
    let value: Value = serde_json::from_str(&config).unwrap();
    let endpoints = &value["endpoints"];
    let actual_path = request.matches.get(1).unwrap();
    let actual_method = request.method.to_uppercase();

    // TODO: Implement these matching.
    // let actual_queries = &request.queries;
    // let actual_headers = &request.headers;
    // let actual_body = &request.body;

    for endpoint in endpoints.as_array().unwrap() {
      let expected_path = &endpoint["path"].as_str().unwrap();
      if expected_path != actual_path {
        continue;
      }

      let when = &endpoint["when"];
      for condition in when.as_array().unwrap() {
        let expected_method = &condition["method"];
        if actual_method != expected_method.as_str().unwrap().to_uppercase() {
          continue;
        }
        let expected_delay = &condition["delay"].as_u64().unwrap();
        let expected_response = &condition["response"];
        let expected_headers = &expected_response["headers"];
        let mut expected_headers_map = HashMap::new();
        for (key, value) in expected_headers.as_object().unwrap() {
          expected_headers_map.insert(key.to_string(), value.to_string());
        }

        if expected_delay > &0 {
          std::thread::sleep(std::time::Duration::from_millis(*expected_delay));
        }

        return Response {
          status: expected_response["status"].as_u64().unwrap() as u16,
          body: expected_response["body"].to_string(),
          headers: expected_headers_map,
        };
      }
    }

    Response {
      status: 400,
      body: "Not implemented.".to_string(),
      headers: HashMap::new(),
    }
  }
}
