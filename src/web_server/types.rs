use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct RequestPath {
  pub path: String,
  pub queries: HashMap<String, String>,
  pub params: HashMap<String, String>,
}

/// A data structure that represents a request.
pub struct Request {
  pub method: String,
  pub path: String,
  pub version: String,
  pub headers: HashMap<String, String>,
  pub body: String,
  pub queries: HashMap<String, String>,
  pub params: HashMap<String, String>,
}

pub enum Method {
  Get,
  Post,
  Put,
}

impl Method {
  pub fn to_string(&self) -> String {
    match self {
      Method::Get => String::from("GET"),
      Method::Post => String::from("POST"),
      Method::Put => String::from("PUT"),
    }
  }
}

pub enum RequestPathPattern {
  Exact(String),
  Match(String),
}

pub struct RequestOption {
  pub method: Method,
  pub path: RequestPathPattern,
}

/// A data structure that represents a response.
pub struct Response {
  pub status: u16,
  pub body: String,
  pub headers: HashMap<String, String>,
}

/// A data structure that similar to a [HashMap].
pub struct Nested {
  values: Vec<(String, NestedValue)>,
}

impl Nested {
  pub fn new() -> Self {
    Self { values: Vec::new() }
  }

  fn insert(&mut self, key: String, value: NestedValue) {
    self.values.push((key, value));
  }

  pub fn insert_string(&mut self, key: String, value: String) {
    self.insert(key, NestedValue::Str(value));
  }

  pub fn iter(&self) -> std::slice::Iter<(String, NestedValue)> {
    self.values.iter()
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }
}

/// A value that can be stored in a [Nested].
pub enum NestedValue {
  // Map(Nested),
  Str(String),
  // Bool(bool),
  // Int(i32),
  // Float(f32),
}
