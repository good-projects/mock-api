mod web_server;
use web_server::{Response, Server, ServerConf};

const SERVER_ADDR: &str = "127.0.0.1:53500";
const MAX_CONNECTIONS: usize = 4;

fn main() {
  let mut server = Server::new(ServerConf {
    max_connections: MAX_CONNECTIONS,
  });

  server.get("/", |request| {
    println!("hello");
    //
    Response
  });

  server.listen(String::from(SERVER_ADDR));
}
