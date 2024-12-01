use serde::{Deserialize, Serialize};
use std::{path::Path, str};
use stegano_core::{/*commands::unveil,SteganoCore*/ CodecOptions, Hide, Media, Message, Persist,};

#[derive(Serialize, Deserialize, Debug)]
struct EmbeddedMetaData {
    timestamp: String,
    view_count: i32,
}

use crate::{display_dynamic_image, unveil_image, unveil_txt};

pub fn decode_image(encoded_image_path: String, user_name: String) -> Result<String, String> {
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
        match text[user_name.len() + 1..].parse::<i32>() {
            Ok(count) => view_count = count,
            Err(_) => return Err("Failed to parse view count".to_string()),
        }
    }
    // if view_count is greater than 0, then display the image else return you have no views
    if view_count > 0 {
        display_dynamic_image(result.unwrap());
    } else {
        return Err("You have no views".to_string());
    }

    // Now we need to update view count and add it back to encoded image
    // We will make the text to be user_name + {view_count -1}
    let new_text = format!("{} {}", user_name, view_count - 1);

    // Now we need to encode the new text back to the image
    let mut message = Message::empty();
    message.add_file_data("view_count.txt", new_text.into_bytes());

    let mut media = Media::from_file(Path::new(&encoded_image_path)).unwrap();
    media.hide_message(&message).unwrap();
    media.save_as(Path::new(&encoded_image_path)).unwrap();

    // let read_txt = unveil_txt(Path::new(&encoded_image_path), &CodecOptions::default());
    // if let Ok(text) = read_txt {
    //     println!("Extracted text 2: {}", text);
    // }

    Ok(format!(
        "Extracted correctly. Remaining views: {}",
        view_count - 1
    ))
}
