pub mod image_decode;
pub mod image_encode_service;
pub mod utils;

// use image::DynamicImage;
// use std::fs::File;
// use std::io::Write;
// use std::path::Path;
// use stegano_core::media::audio::wav_iter::AudioWavIter;
// use stegano_core::media::image::LsbCodec;
// use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
// use stegano_core::{CodecOptions, Media, Message, RawMessage, SteganoError};

// #[derive(Debug)]
// enum LocalSteganoError {
//     NoSecretData,
//     WriteError { source: std::io::Error },
//     ImageError { source: image::ImageError },
// }

// impl PartialEq for LocalSteganoError {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (LocalSteganoError::NoSecretData, LocalSteganoError::NoSecretData) => true,
//             (LocalSteganoError::WriteError { .. }, LocalSteganoError::WriteError { .. }) => false,
//             (LocalSteganoError::ImageError { .. }, LocalSteganoError::ImageError { .. }) => false,
//             _ => false,
//         }
//     }
// }

// struct Message {
//     text: Option<String>,
//     image_data: Option<Vec<u8>>,
// }

// pub fn unveil(
//     secret_media: &Path,
//     destination: &Path,
//     opts: &CodecOptions,
// ) -> Result<(), LocalSteganoError> {
//     let media = Media::from_file(secret_media)?;

//     let files = match media {
//         Media::Image(image) => {
//             let mut decoder = LsbCodec::decoder(&image, opts);
//             let msg = Message::of(&mut decoder);
//             let mut files = msg.files;

//             if let Some(text) = msg.text {
//                 files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
//             }

//             files
//         }
//         Media::Audio(audio) => {
//             let mut decoder = Decoder::new(AudioWavIter::new(audio.1.into_iter()), OneBitUnveil);

//             let msg = Message::of(&mut decoder);
//             let mut files = msg.files;

//             if let Some(text) = msg.text {
//                 files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
//             }

//             files
//         }
//     };

//     if files.is_empty() {
//         return Err(LocalSteganoError::NoSecretData);
//     }

//     for (file_name, buf) in files.iter().map(|(file_name, buf)| {
//         let file = Path::new(file_name).file_name().unwrap().to_str().unwrap();

//         (file, buf)
//     }) {
//         let target_file = destination.join(file_name);
//         let mut target_file =
//             File::create(target_file).map_err(|source| LocalSteganoError::WriteError { source })?;

//         target_file
//             .write_all(buf.as_slice())
//             .map_err(|source| LocalSteganoError::WriteError { source })?;
//     }

//     Ok(())
// }

// fn extract_files(msg: &Message) -> Result<Vec<(String, Vec<u8>)>, LocalSteganoError> {
//     let mut files = Vec::new();

//     if let Some(text) = &msg.text {
//         files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
//     }

//     if let Some(image_data) = &msg.image_data {
//         files.push(("hidden-image.png".to_owned(), image_data.clone()));
//     }

//     if files.is_empty() {
//         return Err(SteganoError::NoSecretData);
//     }

//     Ok(files)
// }

// fn save_files(destination: &Path, files: Vec<(String, Vec<u8>)>) -> Result<(), LocalSteganoError> {
//     for (file_name, buf) in files.iter().map(|(file_name, buf)| {
//         let file = Path::new(file_name).file_name().unwrap().to_str().unwrap();

//         (file, buf)
//     }) {
//         let target_file = destination.join(file_name);
//         let mut target_file =
//             File::create(target_file).map_err(|source| SteganoError::WriteError { source })?;

//         target_file
//             .write_all(buf.as_slice())
//             .map_err(|source| SteganoError::WriteError { source })?;
//     }

//     Ok(())
// }

// fn get_hidden_image(msg: &Message) -> Result<DynamicImage, LocalSteganoError> {
//     let files = extract_files(msg)?;

//     for (file_name, buf) in files {
//         if file_name.ends_with(".png") || file_name.ends_with(".jpg") {
//             let image = image::load_from_memory(&buf)
//                 .map_err(|source| LocalSteganoError::ImageError { source })?;
//             return Ok(image);
//         }
//     }

//     Err(LocalSteganoError::NoSecretData)
// }

// fn get_hidden_text(msg: &Message) -> Result<String, LocalSteganoError> {
//     let files = extract_files(msg)?;

//     for (file_name, buf) in files {
//         if file_name == "secret-message.txt" {
//             let text = String::from_utf8(buf).map_err(|_| LocalSteganoError::NoSecretData)?;
//             return Ok(text);
//         }
//     }

//     Err(SteganoError::NoSecretData)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use image::DynamicImage;

//     #[test]
//     fn test_get_hidden_text() {
//         let msg = Message {
//             text: Some("This is a secret message".to_string()),
//             image_data: None,
//         };

//         let result = get_hidden_text(&msg);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), "This is a secret message");
//     }

//     #[test]
//     fn test_get_hidden_text_no_data() {
//         let msg = Message {
//             text: None,
//             image_data: None,
//         };

//         let result = get_hidden_text(&msg);
//         assert!(result.is_err());
//         assert_eq!(result.unwrap_err(), LocalSteganoError::NoSecretData);
//     }

//     #[test]
//     fn test_get_hidden_image() {
//         let image_data = include_bytes!("test_image.png").to_vec();
//         let msg = Message {
//             text: None,
//             image_data: Some(image_data),
//         };

//         let result = get_hidden_image(&msg);
//         assert!(result.is_ok());
//         assert!(matches!(result.unwrap(), DynamicImage::ImageRgb8(_)));
//     }

//     #[test]
//     fn test_get_hidden_image_no_data() {
//         let msg = Message {
//             text: None,
//             image_data: None,
//         };

//         let result = get_hidden_image(&msg);
//         assert!(result.is_err());
//         assert_eq!(result.unwrap_err(), SteganoError::NoSecretData);
//     }
// }
