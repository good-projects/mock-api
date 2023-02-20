mod web_server;
use std::{any::Any, collections::HashMap};

use web_server::{Response, Server, ServerConf};

const SERVER_ADDR: &str = "127.0.0.1:53500";
const MAX_CONNECTIONS: usize = 4;

fn main() {
  let mut server = Server::new(ServerConf {
    max_connections: MAX_CONNECTIONS,
  });

  server.get("/", |request| {
    let status = 200;
    let headers = None;
    let mut body = HashMap::new();

    body.insert(
      "name".to_owned(),
      Box::new("Hello World".to_owned()) as Box<dyn Any>,
    );

    Response::json(status, body, headers)
  });

  // Get a project.
  server.get("/projects/:name", |request| {
    let body = HashMap::new();
    Response::json(200, body, None)
  });

  // Create a project.
  server.post("/projects/:name", |request| {
    let body = HashMap::new();
    Response::json(200, body, None)
  });

  // Update a project.
  // TODO: Implement this.

  server.listen(String::from(SERVER_ADDR));
}
