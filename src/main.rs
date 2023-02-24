mod web_server;

use std::{collections::HashMap, fs::read_to_string};
use web_server::{
  types::{Nested, Response},
  Server, ServerConf,
};

mod helpers;

const SERVER_ADDR: &str = "127.0.0.1:53500";
const MAX_CONNECTIONS: usize = 4;

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
    let file = helpers::get_project_config_file_path(request.params.get("name").unwrap());

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
  server.post("/projects/:name", helpers::save_config());

  // Update a project.
  server.put("/projects/:name", helpers::save_config());

  server.listen(String::from(SERVER_ADDR));
}
