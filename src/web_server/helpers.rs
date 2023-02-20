use std::{any::Any, collections::HashMap};

/// Converts a HashMap<String, Box<dyn Any>> to a JSON string.
pub fn map_to_string(map: &HashMap<String, Box<dyn Any>>) -> String {
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

#[derive(PartialEq, Debug)]
pub struct RequestPath {
  pub path: String,
  pub queries: Option<HashMap<String, String>>,
  pub params: Option<HashMap<String, String>>,
}

/// Parses the parameters in a path.
pub fn parse_request_path(path_pattern: &str, request_path: &str) -> Option<RequestPath> {
  let mut params = HashMap::new();
  let pattern_segments: Vec<&str> = path_pattern.split("/").collect();
  let request_segments: Vec<&str> = request_path.split("/").collect();

  for (pattern, request) in pattern_segments.iter().zip(request_segments.iter()) {
    if pattern.starts_with(":") {
      let key = pattern[1..].to_string();
      params.insert(key, request.to_string());
    } else if pattern != request {
      return None;
    }
  }

  let params = if params.is_empty() {
    None
  } else {
    Some(params)
  };

  Some(RequestPath {
    path: path_pattern.to_string(),
    queries: None,
    params: params,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn params_is_not_none() {
    let result = parse_request_path("/projects/:name", "/projects/my-project");
    assert_eq!(
      result.unwrap().params.unwrap().get("name").unwrap(),
      "my-project"
    );
  }

  #[test]
  fn params_is_none() {
    let result = parse_request_path("/projects/", "/projects/");
    assert_eq!(result.unwrap().params, None);
  }

  #[test]
  fn request_path_does_not_match() {
    let result = parse_request_path("/projects/:name", "/files/");

    assert_eq!(result, None);
  }
}
