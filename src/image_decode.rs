use image::{ GenericImageView, imageops, DynamicImage };
use serde::{ Serialize, Deserialize };
use serde_json::to_vec;
use steganography::{ encoder, decoder };
use std::str;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
struct EmbeddedMetaData {
  timestamp: String,
  view_count: i32,
}

use crate::utils::get_absolute_path;

pub fn load_image_from_file(file_path: &str) -> Result<DynamicImage, String> {
  // if the file path is relative, convert it to an absolute path
  // check if path begins with '/' or '~'
  let file_path = if file_path.starts_with('/') || file_path.starts_with('~') {
    file_path.to_string()
  } else {
    get_absolute_path(file_path)
  };

  let img = image::open(file_path);
  match img {
    Ok(img) => Ok(img),
    Err(e) => Err(format!("Error loading image: {}", e)),
  }
}

pub fn decode_image(image: DynamicImage) -> Result<(DynamicImage, EmbeddedMetaData), String> {
  let my_decoder = decoder::Decoder::new(image.to_rgba());
  let decoded_data = my_decoder.decode_alpha();

  // Find the position of the JSON content
  let start = decoded_data
    .iter()
    .position(|&b| b == b'{')
    .expect("Opening brace not found");
  let end = decoded_data
    .iter()
    .position(|&b| b == b'}')
    .expect("Closing brace not found");

  let json_part = &decoded_data[start..=end]; // Include the closing brace
  let original_image_part = &decoded_data[end + 1..]; // Skip past the closing brace

  let decoded_json: EmbeddedMetaData = serde_json
    ::from_slice(json_part)
    .expect("Failed to parse JSON data");

  let original_image = image::ImageBuffer
    ::from_vec(image.width(), image.height(), original_image_part.to_vec())
    .expect("Failed to create image buffer");

  Ok((DynamicImage::ImageRgba8(original_image), decoded_json))
}
