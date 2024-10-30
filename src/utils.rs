pub fn get_absolute_path(path: &str) -> String {
  let current_dir = std::env::current_dir().unwrap();
  let current_dir = current_dir.to_str().unwrap();
  let absolute_path = format!("{}/{}", current_dir, path);
  absolute_path
}