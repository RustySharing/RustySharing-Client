pub mod clientserver;
pub mod image_decode;
pub mod image_encode_service;
pub mod utils;

use image::DynamicImage;
use std::path::Path;
use stegano_core::media::audio::wav_iter::AudioWavIter;
use stegano_core::media::image::LsbCodec;
use stegano_core::universal_decoder::{Decoder, OneBitUnveil};
use stegano_core::{CodecOptions, Media, Message, SteganoError};

pub fn unveil_image(
    secret_media: &Path,
    opts: &CodecOptions,
) -> Result<DynamicImage, SteganoError> {
    let media = Media::from_file(secret_media)?;

    let files = match media {
        Media::Image(image) => {
            let mut decoder = LsbCodec::decoder(&image, opts);
            let msg = Message::of(&mut decoder);
            let mut files = msg.files;

            if let Some(text) = msg.text {
                files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
            }

            files
        }
        Media::Audio(audio) => {
            let mut decoder = Decoder::new(AudioWavIter::new(audio.1.into_iter()), OneBitUnveil);

            let msg = Message::of(&mut decoder);
            let mut files = msg.files;

            if let Some(text) = msg.text {
                files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
            }

            files
        }
    };

    if files.is_empty() {
        return Err(SteganoError::NoSecretData);
    }

    // print file names

    for (file_name, buf) in files.iter().map(|(file_name, buf)| {
        let file = Path::new(file_name).file_name().unwrap().to_str().unwrap();

        (file, buf)
    }) {
        println!("file_name: {}", file_name);
        if file_name.ends_with(".png")
            || file_name.ends_with(".jpg")
            || file_name.ends_with(".jpeg")
        {
            return image::load_from_memory(buf).map_err(|_| SteganoError::InvalidImageMedia);
        }
    }

    println!("No image found in the media.");

    Err(SteganoError::InvalidImageMedia)
}

pub fn unveil_txt(secret_media: &Path, opts: &CodecOptions) -> Result<String, SteganoError> {
    let media = Media::from_file(secret_media)?;

    let files = match media {
        Media::Image(image) => {
            let mut decoder = LsbCodec::decoder(&image, opts);
            let msg = Message::of(&mut decoder);
            let mut files = msg.files;

            if let Some(text) = msg.text {
                files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
            }

            files
        }
        Media::Audio(audio) => {
            let mut decoder = Decoder::new(AudioWavIter::new(audio.1.into_iter()), OneBitUnveil);

            let msg = Message::of(&mut decoder);
            let mut files = msg.files;

            if let Some(text) = msg.text {
                files.push(("secret-message.txt".to_owned(), text.as_bytes().to_vec()));
            }

            files
        }
    };

    if files.is_empty() {
        return Err(SteganoError::NoSecretData);
    }

    for (file_name, buf) in files.iter().map(|(file_name, buf)| {
        let file = Path::new(file_name).file_name().unwrap().to_str().unwrap();

        (file, buf)
    }) {
        println!("file_name: {}", file_name);
        if file_name.ends_with(".txt") {
            return String::from_utf8(buf.clone()).map_err(|_| SteganoError::NoSecretData);
        }
    }

    println!("No secret message found in the media.");

    Err(SteganoError::NoSecretData)
}

use minifb::{Key, MouseMode, Window, WindowOptions};

pub fn display_dynamic_image(image: DynamicImage) {
    let rgba_image = image.to_rgba();
    let (image_width, image_height) = rgba_image.dimensions();
    let buffer: Vec<u32> = rgba_image
        .pixels()
        .map(|p| {
            let [r, g, b, a] = p.data;
            ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        })
        .collect();

    // Create the window
    let mut window = Window::new(
        "Display Image - [Draggable]",
        800, // Window width (bigger than image for movement)
        600, // Window height
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create window");

    // Variables to track image position and dragging state
    let mut image_x = 0;
    let mut image_y = 0;
    let mut dragging = false;
    let mut last_mouse_pos = (0, 0);

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle mouse input
        if let Some(mouse_pos) = window.get_mouse_pos(MouseMode::Clamp) {
            let mouse_x = mouse_pos.0 as i32;
            let mouse_y = mouse_pos.1 as i32;

            if window.get_mouse_down(minifb::MouseButton::Left) {
                if !dragging {
                    // Start dragging
                    dragging = true;
                    last_mouse_pos = (mouse_x, mouse_y);
                } else {
                    // Update image position while dragging
                    let dx = mouse_x - last_mouse_pos.0;
                    let dy = mouse_y - last_mouse_pos.1;
                    image_x += dx;
                    image_y += dy;
                    last_mouse_pos = (mouse_x, mouse_y);
                }
            } else {
                // Stop dragging
                dragging = false;
            }
        }

        // Create a blank buffer to clear the window
        let mut display_buffer = vec![0u32; 800 * 600]; // Match window size

        // Draw the image at the current position
        for y in 0..image_height as usize {
            for x in 0..image_width as usize {
                let window_x = x as i32 + image_x;
                let window_y = y as i32 + image_y;

                if (0..800).contains(&window_x) && (0..600).contains(&window_y) {
                    let buffer_index = y * image_width as usize + x;
                    let display_index = window_y as usize * 800 + window_x as usize;

                    display_buffer[display_index] = buffer[buffer_index];
                }
            }
        }

        // Update the window with the display buffer
        window
            .update_with_buffer(&display_buffer, 800, 600)
            .expect("Failed to update window buffer");
    }
}
