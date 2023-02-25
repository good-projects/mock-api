use std::path::PathBuf;

use crate::web_server::types::Request;

/// Returns the path to a project's config file.
pub fn get_project_config_file_path(project_name: &str) -> PathBuf {
  PathBuf::from(format!("database/projects/{}.json", project_name))
}

/// Returns the path to a project's config file from a request.
pub fn config_file_path_from_request(request: &Request) -> PathBuf {
  let project_name = request.params.get("name").unwrap();
  get_project_config_file_path(project_name)
}
