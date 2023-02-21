use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct RequestPath {
  pub path: String,
  pub queries: HashMap<String, String>,
  pub params: HashMap<String, String>,
}

pub struct Request {
  pub method: String,
  pub path: String,
  pub version: String,
  pub headers: HashMap<String, String>,
  pub body: String,
  pub queries: HashMap<String, String>,
  pub params: HashMap<String, String>,
}

pub struct Response {
  pub status: u16,
  pub body: String,
  pub headers: HashMap<String, String>,
}
