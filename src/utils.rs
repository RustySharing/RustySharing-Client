use std::path::Path;

pub fn get_absolute_path(path: &str) -> String {
  let current_dir = std::env::current_dir().unwrap();
  let current_dir = current_dir.to_str().unwrap();
  let absolute_path = format!("{}/{}", current_dir, path);
  absolute_path
}

pub fn get_file_name(path: &str) -> String {
  Path::new(path).file_name().unwrap_or_default().to_string_lossy().into_owned()
}
