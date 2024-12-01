use serde::{Deserialize, Serialize};
use std::{fs, path::Path, str};
use stegano_core::{commands::unveil, CodecOptions, SteganoCore};

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

use crate::{display_dynamic_image, unveil_image, unveil_txt};

pub fn decode_image(
    encoded_image_path: String,
    extraction_path: String,
    user_name: String,
) -> Result<String, String> {
    //let extraction_path = "./extracted"; // Path to save extracted image
    // if let Err(e) = create_directory_if_not_exists(&extraction_path) {
    //     eprintln!("Error creating directory: {}", e);
    // }
    // Extract the hidden file from the image
    // let _ = unveil(
    //   Path::new(&encoded_image_path),
    //   Path::new(&extraction_path),
    //   &CodecOptions::default()
    // );

    let text = unveil_txt(Path::new(&encoded_image_path), &CodecOptions::default());

    let result = unveil_image(Path::new(&encoded_image_path), &CodecOptions::default());
    let mut view_count = 0;
    println!("Extraction complete.");
    // Print the extracted text
    if let Ok(text) = text {
        println!("Extracted text: {}", text);
        // check that text starts with user_name
        if !text.starts_with(&user_name) {
            return Err("User name does not match".to_string());
        }
        // Read the number that follows user_name
        view_count = text[user_name.len()..].parse::<i32>().unwrap();
    }
    // if view_count is greater than 0, then display the image else return you have no views
    if view_count > 0 {
        display_dynamic_image(result.unwrap());
    } else {
        return Err("You have no views".to_string());
    }

    // Now we need to update view count and add it back to encoded image
    // We will make the text to be user_name + {view_count -1}
    let new_text = format!("{}{}", user_name, view_count - 1);

    // Now we need to encode the new text back to the image

    Ok(format!("Extracted file saved to: {}", extraction_path))
}
