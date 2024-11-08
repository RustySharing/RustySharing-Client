use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::EncodedImageRequest;
use std::fs;
use std::fs::File;
use std::path::Path;
use stegano_core::commands::unveil;
use stegano_core::CodecOptions;
use steganography::util::{bytes_to_file, file_to_bytes};

pub mod image_encoding {
    tonic::include_proto!("image_encoding");
}

pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
    ImageEncoderClient::connect("http://[::1]:50051")
        .await
        .unwrap()
}

fn create_directory_if_not_exists(dir_path: &str) -> std::io::Result<()> {
    // Convert the dir_path to a Path
    let path = Path::new(dir_path);

    // Create the directory (and any parent directories) if it doesn't exist
    fs::create_dir_all(path)?;

    println!("Directory '{}' created or already exists.", dir_path);

    Ok(())
}

pub async fn image_encode(
    client: &mut ImageEncoderClient<tonic::transport::Channel>,
    image_path: &str,
) -> String {
    let image_file = File::open(image_path).unwrap();
    let image_data = file_to_bytes(image_file);

    let request = tonic::Request::new(EncodedImageRequest { image_data });

    println!("Sending ...");

    let response = client.image_encode(request).await.unwrap();

    println!("Sent!");

    let encoded_data = &response.get_ref().image_data;

    // new file for output
    let output_file_path = "encoded_image.png";

    let file = File::create(output_file_path).unwrap(); // Unwrap the Result here

    bytes_to_file(encoded_data, &file);

    let extraction_path = "./extracted"; // Path to save extracted image
    if let Err(e) = create_directory_if_not_exists(extraction_path) {
        eprintln!("Error creating directory: {}", e);
    }
    // Extract the hidden file from the image
    let _ = unveil(
        Path::new(output_file_path),
        Path::new(extraction_path),
        &CodecOptions::default(),
    );

    println!("Extracted file saved to: {}", extraction_path);

    ("Extracted file saved to: {}".to_string() + extraction_path).to_string()
}
