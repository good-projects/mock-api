mod web_server;

use std::{collections::HashMap, fs::read_to_string};
use web_server::{
  types::{Method, Nested, RequestOption, Response},
  Server, ServerConf,
};

mod handlers;
mod helpers;

const SERVER_ADDR: &str = "127.0.0.1:53500";
const MAX_CONNECTIONS: usize = 1000;

fn main() {
  let mut server = Server::new(ServerConf {
    max_connections: MAX_CONNECTIONS,
  });

  server.get("/", |_| {
    let status = 200;
    let headers = None;
    let mut body = Nested::new();
    body.insert_string("name".to_string(), "Hello world!".to_string());

    Response::json(status, body, headers)
  });

  // Get a project.
  server.get("/projects/:name", |request| {
    let file = helpers::config_file_path_from_request(&request);

    if file.exists() {
      let content = read_to_string(file).unwrap();
      let mut headers = HashMap::new();
      headers.insert(
        String::from("Content-Type"),
        String::from("application/json"),
      );
      Response::ok(content, Some(headers))
    } else {
      let mut body = Nested::new();
      body.insert_string("error".to_string(), "Project does not exist.".to_string());
      Response::json(404, body, None)
    }
  });

  // Create a project.
  server.post("/projects/:name", handlers::save_config());

  // Update a project.
  server.put("/projects/:name", handlers::save_config());

  // A mock request of a given project.
  server.request(
    handlers::mock_request(),
    RequestOption {
      path: web_server::types::RequestPathPattern::Match(r"^/projects/([^/]+)/([^?]+)".to_string()),
      method: Method::Get,
    },
  );

  server.listen(String::from(SERVER_ADDR));
}
