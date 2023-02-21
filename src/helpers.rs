use std::path::PathBuf;

pub fn get_project_config_file_path(project_name: &str) -> PathBuf {
  PathBuf::from(format!("database/projects/{}.json", project_name))
}
