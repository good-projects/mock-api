use std::{
  any::Any,
  collections::HashMap,
  io::{prelude::*, BufReader},
  net::{TcpListener, TcpStream},
  sync::{Arc, Mutex},
};

enum Method {
  Get,
}
impl Method {
  fn to_string(&self) -> String {
    match self {
      Method::Get => String::from("GET"),
    }
  }
}

mod thread_pool;

pub use thread_pool::ThreadPool;

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

  pub fn get<F>(&mut self, path: &str, request_handler: F)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    let mut connection_handler = self.connection_handler.lock().unwrap();
    connection_handler.listeners.push(Listener {
      method: Method::Get,
      route: String::from(path),
      handler: Box::new(request_handler),
    });
  }
}

pub struct Request {
  path: String,
  queries: String,
  params: String,
  body: String,
  headers: String,
}

pub struct Response {
  status: u16,
  body: String,
  headers: HashMap<String, String>,
}

fn map_to_string(map: &HashMap<String, Box<dyn Any>>) -> String {
  let mut result = String::new();
  result.push_str("{");

  for (i, (key, value)) in map.iter().enumerate() {
    match value.downcast_ref::<i32>() {
      Some(v) => result.push_str(&format!("\"{}\":{}", key, v)),
      None => match value.downcast_ref::<String>() {
        Some(v) => result.push_str(&format!("\"{}\":\"{}\"", key, v)),
        None => result.push_str(&format!("\"{}\":null", key)),
      },
    };
    if i < map.len() - 1 {
      result.push_str(",");
    }
  }

  result.push_str("}");
  result
}

impl Response {
  pub fn json(
    status: u16,
    body: HashMap<String, Box<dyn Any>>,
    headers: Option<HashMap<String, String>>,
  ) -> Response {
    let mut headers = headers.unwrap_or(HashMap::new());

    headers.insert(
      String::from("Content-Type"),
      String::from("application/json"),
    );

    Response {
      status,
      body: map_to_string(&body),
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
    // BufReader implements the std::io::BufRead trait, which provides the lines
    // method.
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader
      // The lines method returns an iterator of `Result<String, std::io::Error>`
      // by splitting the stream of data whenever it sees a newline byte.
      .lines()
      // gets the first item from the iterator.
      .next()
      // takes care of the `Option` and stops the program if the iterator has no
      // items
      .unwrap()
      // handles the `Result`.
      .unwrap();

    let mut request_line = request_line.split_whitespace();
    let method = request_line.next().unwrap().to_uppercase();
    let path = request_line.next().unwrap();

    let mut status = 500;
    let mut contents = String::new();
    let mut headers = String::new();

    for listener in self.listeners.iter() {
      let handler = &listener.handler;

      let request = Request {
        path: String::from(path),
        queries: String::from(""),
        params: String::from(""),
        body: String::from(""),
        headers: String::from(""),
      };

      if listener.route == path && listener.method.to_string() == method {
        let response = handler(request);
        status = response.status;
        contents = response.body;

        if response.headers.len() > 0 {
          for (key, value) in response.headers.iter() {
            headers.push_str(&format!("{}: {}\r\n", key, value));
          }
        }
      }
    }

    let length = contents.len();
    let response =
      format!("HTTP/1.1 {status}\r\n{headers}Content-Length: {length}\r\n\r\n{contents}");

    // The write_all method on stream takes a &[u8] and sends those bytes directly
    // down the connection.
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
  }
}
