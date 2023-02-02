use std::{
  io::{prelude::*, BufReader},
  net::{TcpListener, TcpStream},
  sync::Arc,
};

enum Method {
  get,
}

mod thread_pool;

pub use thread_pool::ThreadPool;

pub struct Listener {
  route: String,
  method: Method,
  handler: Handler,
}

type Handler = Box<dyn FnOnce(Request) -> Response + 'static>;

pub struct Server {
  max_connections: usize,
  listeners: Vec<Listener>,
  connection_handler_reference: Arc<ConnectionHandler>,
}

pub struct ServerConf {
  pub max_connections: usize,
}

impl Server {
  pub fn new(conf: ServerConf) -> Server {
    Server {
      max_connections: conf.max_connections,
      listeners: Vec::new(),
      connection_handler_reference: Arc::new(ConnectionHandler {}),
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

      let handler = self.connection_handler_reference.clone();

      pool.execute(move || {
        handler.handle_connection(stream);
      });
    }
  }

  pub fn get<F>(&mut self, path: &str, f: F)
  where
    F: FnOnce(Request) -> Response + 'static,
  {
    let handler = Box::new(f);
    self.listeners.push(Listener {
      method: Method::get,
      route: String::from(path),
      handler,
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

pub struct Response;

struct ConnectionHandler {}

impl ConnectionHandler {
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
    let method = request_line.next().unwrap();
    let path = request_line.next().unwrap();

    let (status_line, contents) = if path == "/" {
      ("HTTP/1.1 200 OK", "Hello")
    } else {
      ("HTTP/1.1 404 NOT FOUND", "Not found")
    };

    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // The write_all method on stream takes a &[u8] and sends those bytes directly
    // down the connection.
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
  }
}
