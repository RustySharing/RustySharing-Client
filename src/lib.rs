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
    // let rgba_image = image.to_rgba();
    // let (image_width, image_height) = rgba_image.dimensions();
    // let buffer: Vec<u32> = rgba_image
    //     .pixels()
    //     .map(|p| {
    //         let [r, g, b, a] = p.data;
    //         ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    //     })
    //     .collect();

    // let mut window = Window::new(
    //     "Display Image - [Draggable]",
    //     800,
    //     600,
    //     WindowOptions {
    //         resize: false,
    //         ..WindowOptions::default()
    //     },
    // )
    // .unwrap();

    // let mut image_x = 0;
    // let mut image_y = 0;
    // let mut dragging = false;
    // let mut last_mouse_pos = (0, 0);

    // while window.is_open() && !window.is_key_down(Key::Escape) {
    //     if let Some(mouse_pos) = window.get_mouse_pos(MouseMode::Clamp) {
    //         let mouse_x = mouse_pos.0 as i32;
    //         let mouse_y = mouse_pos.1 as i32;

    //         if window.get_mouse_down(minifb::MouseButton::Left) {
    //             if !dragging {
    //                 dragging = true;
    //                 last_mouse_pos = (mouse_x, mouse_y);
    //             } else {
    //                 let dx = mouse_x - last_mouse_pos.0;
    //                 let dy = mouse_y - last_mouse_pos.1;
    //                 image_x += dx;
    //                 image_y += dy;
    //                 last_mouse_pos = (mouse_x, mouse_y);
    //             }
    //         } else {
    //             dragging = false;
    //         }
    //     }

    //     let mut display_buffer = vec![0u32; 800 * 600];

    //     for y in 0..image_height as usize {
    //         for x in 0..image_width as usize {
    //             let window_x = x as i32 + image_x;
    //             let window_y = y as i32 + image_y;

    //             if (0..800).contains(&window_x) && (0..600).contains(&window_y) {
    //                 let buffer_index = y * image_width as usize + x;
    //                 let display_index = window_y as usize * 800 + window_x as usize;
    //                 display_buffer[display_index] = buffer[buffer_index];
    //             }
    //         }
    //     }

    //     let _ = window.update_with_buffer(&display_buffer, 800, 600);
    // }
    let viewer = ImageViewer::new("My Image", &image);
    match viewer {
        Ok(mut viewer) => viewer.display(&image).unwrap(),
        Err(e) => eprintln!("Failed to create image viewer: {}", e),
    }
}

use image::GenericImageView;
use std::error::Error;

pub struct ImageViewer {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
}

impl ImageViewer {
    pub fn new(title: &str, image: &DynamicImage) -> Result<Self, Box<dyn Error>> {
        // Get the image dimensions
        let (img_width, img_height) = image.dimensions();

        // Get the primary monitor resolution
        // Default to 1920x1080 if we can't get the monitor info
        let (screen_width, screen_height) = (1920, 1080);

        // Calculate the optimal window size
        // Use 80% of screen size as maximum
        let max_width = (screen_width as f32 * 0.8) as u32;
        let max_height = (screen_height as f32 * 0.8) as u32;

        // Calculate scaling factor to fit within max dimensions while preserving aspect ratio
        let scale_width = max_width as f32 / img_width as f32;
        let scale_height = max_height as f32 / img_height as f32;
        let scale = scale_width.min(scale_height).min(1.0);

        // Calculate final dimensions
        let width = (img_width as f32 * scale) as usize;
        let height = (img_height as f32 * scale) as usize;

        // Ensure minimum window size
        let width = width.max(320);
        let height = height.max(240);

        let mut window = Window::new(
            title,
            width,
            height,
            WindowOptions {
                resize: true,
                scale: minifb::Scale::X1,
                // Disable copy/paste and other system operations
                borderless: false,
                title: true,
                ..WindowOptions::default()
            },
        )?;

        // Disable right-click menu and other system shortcuts
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Ok(ImageViewer {
            window,
            buffer: vec![0; width * height],
            width,
            height,
        })
    }

    fn update_buffer_from_image(&mut self, image: &DynamicImage) {
        let resized = image.resize_exact(
            self.width as u32,
            self.height as u32,
            image::imageops::FilterType::Lanczos3,
        );

        self.buffer.clear();
        self.buffer.reserve(self.width * self.height);

        for pixel in resized.pixels() {
            let rgba = pixel.2.data;
            let pixel_value: u32 = ((rgba[3] as u32) << 24)
                | ((rgba[0] as u32) << 16)
                | ((rgba[1] as u32) << 8)
                | rgba[2] as u32;
            self.buffer.push(pixel_value);
        }
    }

    pub fn display(&mut self, image: &DynamicImage) -> Result<(), Box<dyn Error>> {
        self.update_buffer_from_image(image);

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.window
                .update_with_buffer(&self.buffer, self.width, self.height)?;

            // Handle window resize
            let (new_width, new_height) = self.window.get_size();
            if new_width != self.width || new_height != self.height {
                self.width = new_width;
                self.height = new_height;
                self.buffer = vec![0; new_width * new_height];
                self.update_buffer_from_image(image);
            }

            // Intercept and block common save shortcuts
            if (self.window.is_key_down(Key::LeftCtrl) || self.window.is_key_down(Key::RightCtrl))
                && (self.window.is_key_down(Key::S))
            {
                println!("Save operation blocked");
            }
        }

        Ok(())
    }
}
