use serde::{ Deserialize, Serialize };
use std::{ fs, path::Path, str };
use stegano_core::{ commands::unveil, CodecOptions };

#[derive(Serialize, Deserialize, Debug)]
struct EmbeddedMetaData {
  timestamp: String,
  view_count: i32,
}

fn create_directory_if_not_exists(dir_path: &str) -> std::io::Result<()> {
  // Convert the dir_path to a Path
  let path = Path::new(dir_path);

  // Create the directory (and any parent directories) if it doesn't exist
  fs::create_dir_all(path)?;

  println!("Directory '{}' created or already exists.", dir_path);

  Ok(())
}

pub fn decode_image(encoded_image_path: String, extraction_path: String) -> Result<String, String> {
  //let extraction_path = "./extracted"; // Path to save extracted image
  if let Err(e) = create_directory_if_not_exists(&extraction_path) {
    eprintln!("Error creating directory: {}", e);
  }
  // Extract the hidden file from the image
  let _ = unveil(
    Path::new(&encoded_image_path),
    Path::new(&extraction_path),
    &CodecOptions::default()
  );

  Ok(format!("Extracted file saved to: {}", extraction_path))
}
