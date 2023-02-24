use std::{
  collections::HashMap,
  io::Write,
  net::{TcpListener, TcpStream},
  sync::{Arc, Mutex},
};

enum Method {
  Get,
  Post,
  Put,
}
impl Method {
  fn to_string(&self) -> String {
    match self {
      Method::Get => String::from("GET"),
      Method::Post => String::from("POST"),
      Method::Put => String::from("PUT"),
    }
  }
}

mod helpers;
mod thread_pool;
pub mod types;

use types::{Request, Response};

pub use thread_pool::ThreadPool;

use self::types::Nested;

pub struct Listener {
  route: String,
  method: Method,
  handler: Handler,
}

type Handler = Box<dyn Fn(Request) -> Response + Send + 'static>;

pub struct Server {
  max_connections: usize,
  connection_handler: Arc<Mutex<ConnectionHandler>>,
}

pub struct ServerConf {
  pub max_connections: usize,
}

impl Server {
  pub fn new(conf: ServerConf) -> Server {
    Server {
      max_connections: conf.max_connections,
      connection_handler: Arc::new(Mutex::new(ConnectionHandler::new())),
    }
  }

  pub fn listen(&self, addr: String) {
    // Some possible reasons for binding to fail:
    // - connecting to a port requires administrator privileges.
    // - listening to a port which is occupied.
    let listener = TcpListener::bind(addr).unwrap();

    // Limit the number of threads in the pool to a small number to protect us
    // from Denial of Service (DoS) attacks.
    let pool = ThreadPool::new(self.max_connections);

    for stream in listener.incoming() {
      // The browser signals the end of an HTTP request by sending two newline
      // characters in a row.
      // The reason we might receive errors from the incoming method when a client
      // connects to the server is that we’re not actually iterating over
      // connections. Instead, we’re iterating over connection attempts. The
      // connection might not be successful for a number of reasons, many of them
      // operating system specific. For example, many operating systems have a
      // limit to the number of simultaneous open connections they can support;
      // new connection attempts beyond that number will produce an error until
      // some of the open connections are closed.
      let stream = stream.unwrap();

      let connection_handler = self.connection_handler.clone();

      pool.execute(move || {
        let connection_handler = connection_handler.lock().unwrap();
        connection_handler.handle_connection(stream);
      });
    }
  }

  fn request<F>(&mut self, method: Method, path: &str, request_handler: F)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    let mut connection_handler = self.connection_handler.lock().unwrap();
    connection_handler.listeners.push(Listener {
      method,
      route: String::from(path),
      handler: Box::new(request_handler),
    });
  }

  pub fn get<F>(&mut self, path: &str, request_handler: F)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    self.request(Method::Get, path, request_handler);
  }

  pub fn post<F>(&mut self, path: &str, request_handler: F)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    self.request(Method::Post, path, request_handler);
  }

  pub fn put<F>(&mut self, path: &str, request_handler: F)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    self.request(Method::Put, path, request_handler);
  }
}

impl Response {
  pub fn json(status: u16, body: Nested, headers: Option<HashMap<String, String>>) -> Response {
    let mut headers = headers.unwrap_or(HashMap::new());

    headers.insert(
      String::from("Content-Type"),
      String::from("application/json"),
    );

    Response {
      status,
      body: helpers::stringify_nested(&body),
      headers,
    }
  }

  pub fn ok(body: String, headers: Option<HashMap<String, String>>) -> Response {
    let mut headers = headers.unwrap_or(HashMap::new());

    if headers.get("Content-Type").is_none() {
      headers.insert(String::from("Content-Type"), String::from("text/plain"));
    }

    Response {
      status: 200,
      body,
      headers,
    }
  }
}

struct ConnectionHandler {
  listeners: Vec<Listener>,
}

impl ConnectionHandler {
  pub fn new() -> ConnectionHandler {
    ConnectionHandler {
      listeners: Vec::new(),
    }
  }

  pub fn handle_connection(&self, mut stream: TcpStream) {
    let mut request = helpers::parse_tcp_stream(&mut stream).unwrap();

    let mut response_status = 404;
    let mut response_body = String::new();
    let mut response_headers = String::new();

    for listener in self.listeners.iter() {
      let parsed_path = helpers::parse_request_path(&listener.route[..], &request.path[..]);

      if parsed_path.is_some() && listener.method.to_string() == request.method {
        let handler = &listener.handler;

        let parsed_path = parsed_path.unwrap();
        request.path = parsed_path.path;
        request.queries = parsed_path.queries;
        request.params = parsed_path.params;

        let response = handler(request);
        response_status = response.status;
        response_body = response.body;

        if response.headers.len() > 0 {
          for (key, value) in response.headers.iter() {
            response_headers.push_str(&format!("{}: {}\r\n", key, value));
          }
        }
        break;
      }
    }

    let length = response_body.len();
    let response = format!(
      "HTTP/1.1 {response_status}\r\n{response_headers}Content-Length: {length}\r\n\r\n{response_body}"
    );

    // The write_all method on stream takes a &[u8] and sends those bytes directly
    // down the connection.
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
  }
}
