use image::{ImageBuffer, Rgba};
use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::EncodedImageRequest;
use std::fs::File;
use std::io::Read;

pub mod image_encoding {
    tonic::include_proto!("image_encoding");
}

pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
    ImageEncoderClient::connect("http://10.7.16.54:50051")
        .await
        .unwrap()
}

pub async fn image_encode(
    client: &mut ImageEncoderClient<tonic::transport::Channel>,
    image_path: &str,
    width: i32,
    height: i32,
) -> String {
    let image_file = File::open(image_path).unwrap();
    let mut image_data = Vec::new();
    image_data = image_file.bytes().map(|byte| byte.unwrap()).collect();

    let request = tonic::Request::new(EncodedImageRequest {
        width,
        height,
        image_data,
    });

    println!("Sending ...");

    let response = client.image_encode(request).await.unwrap();

    println!("Sent!");

    let encoded_data = &response.get_ref().image_data;
    let width = response.get_ref().width;
    let height = response.get_ref().height;
    let decoded_image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        encoded_data.clone(),
    )
    .expect("Failed to create ImageBuffer from raw data");
    // write encoded image to path
    let output_file_path = "encoded_image.png";
    decoded_image
        .save(output_file_path)
        .expect("Failed to save encoded image");

    "Encoded image saved to encoded_image.png".to_string()
}
