mod web_server;

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
  server.post("/projects/:name", helpers::save_config());

  // Update a project.
  server.put("/projects/:name", helpers::save_config());

  server.listen(String::from(SERVER_ADDR));
}
