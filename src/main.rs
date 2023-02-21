mod web_server;
use std::fs;

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
  server.get("/projects/:name", |_| {
    let body = Nested::new();
    Response::json(200, body, None)
  });

  // Create a project.
  server.post("/projects/:name", |request| {
    let project_name = request.params.get("name").unwrap();
    let file_path = helpers::get_project_config_file_path(project_name);

    // Return error if the project already exists.
    if file_path.exists() {
      let mut body = Nested::new();
      body.insert_string("error".to_string(), "Project already exists.".to_string());

      return Response::json(400, body, None);
    }

    // Create the project file.
    fs::write(file_path, request.body).unwrap();

    let body = Nested::new();
    Response::json(200, body, None)
  });

  // Update a project.
  // TODO: Implement this.

  server.listen(String::from(SERVER_ADDR));
}
