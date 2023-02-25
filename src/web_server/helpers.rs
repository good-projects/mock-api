use regex::Regex;

use std::{
  collections::HashMap,
  io::{BufRead, BufReader, Error as IoError, Read},
  net::TcpStream,
};

use super::types::{Nested, NestedValue, Request, RequestPath, RequestPathPattern};

/// Converts a [Nested] to a JSON string.
pub fn stringify_nested(nested: &Nested) -> String {
  let mut result = String::new();
  result.push_str("{ ");
  for (i, (key, value)) in nested.iter().enumerate() {
    result.push_str(&format!("\"{}\": {}", key, stringfy_nested_value(value)));
    if i < nested.len() - 1 {
      result.push_str(", ");
    }
  }
  result.push_str(" }");
  result
}

fn stringfy_nested_value(nested: &NestedValue) -> String {
  match nested {
    // NestedValue::Map(nested) => stringify_nested(nested),
    NestedValue::Str(value) => format!("\"{}\"", value),
    // NestedValue::Bool(value) => format!("{}", value),
    // NestedValue::Int(value) => format!("{}", value),
    // NestedValue::Float(value) => format!("{}", value),
  }
}

/// Parses the parameters in a path.
pub fn parse_request_path(
  path_pattern: &RequestPathPattern,
  request_path: &str,
) -> Option<RequestPath> {
  match path_pattern {
    RequestPathPattern::Exact(path_pattern) => {
      let mut params = HashMap::new();
      let pattern_segments: Vec<&str> = path_pattern.split("/").collect();
      let request_segments: Vec<&str> = request_path.split("/").collect();

      if pattern_segments.len() != request_segments.len() {
        return None;
      }

      for (pattern, request) in pattern_segments.iter().zip(request_segments.iter()) {
        if pattern.starts_with(":") {
          let key = pattern[1..].to_string();
          params.insert(key, request.to_string());
        } else if pattern != request {
          return None;
        }
      }

      Some(RequestPath {
        path: request_path.to_string(),
        queries: HashMap::new(),
        params,
        matches: Vec::new(),
      })
    }
    RequestPathPattern::Match(path_pattern) => {
      let regexp = Regex::new(path_pattern).unwrap();

      if let Some(captures) = regexp.captures(request_path) {
        let mut matches = Vec::new();
        for (i, capture) in captures.iter().enumerate() {
          if i > 0 {
            matches.push(capture.unwrap().as_str().to_string());
          }
        }
        Some(RequestPath {
          path: request_path.to_string(),
          queries: HashMap::new(),
          params: HashMap::new(),
          matches,
        })
      } else {
        None
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn params_is_not_none() {
    let result = parse_request_path(
      &RequestPathPattern::Exact(String::from("/projects/:name")),
      "/projects/my-project",
    );
    assert_eq!(result.unwrap().params.get("name").unwrap(), "my-project");
  }

  #[test]
  fn params_is_none() {
    let result = parse_request_path(
      &RequestPathPattern::Exact(String::from("/projects/")),
      "/projects/",
    );
    assert!(result.unwrap().params.is_empty());
  }

  #[test]
  fn request_path_does_not_match() {
    let result = parse_request_path(
      &RequestPathPattern::Exact(String::from("/projects/:name")),
      "/files/",
    );

    assert_eq!(result, None);
  }
}

pub fn parse_tcp_stream(stream: &mut TcpStream) -> Result<Request, IoError> {
  let mut buf_reader = BufReader::new(stream);
  let mut start_line = String::new();
  buf_reader.read_line(&mut start_line)?;

  let mut start_line_parts = start_line.split_whitespace();
  let method = start_line_parts.next().unwrap().to_uppercase();
  let path = start_line_parts.next().unwrap().to_owned();
  let version = start_line_parts.next().unwrap().to_owned();

  // Read the headers.
  let mut headers = HashMap::new();
  loop {
    let mut line = String::new();
    buf_reader.read_line(&mut line)?;
    if line.trim().is_empty() {
      break;
    }
    if let Some(pos) = line.find(':') {
      let key = line[..pos].trim().to_owned();
      let value = line[pos + 1..].trim().to_owned();
      headers.insert(key, value);
    }
  }

  // Read the body.
  let mut body = String::new();
  if method == "POST" || method == "PUT" {
    let content_length = headers
      .get("Content-Length")
      .and_then(|v| v.parse::<usize>().ok())
      .unwrap_or(0);

    if content_length > 0 {
      let mut buffer = vec![0; content_length];
      buf_reader.read_exact(&mut buffer)?;
      body = String::from_utf8(buffer).unwrap();
    }
  }

  Ok(Request {
    path,
    version,
    method,
    headers,
    body,
    queries: HashMap::new(),
    params: HashMap::new(),
    matches: Vec::new(),
  })
}
