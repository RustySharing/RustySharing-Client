use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::{ EncodedImageRequest };
use std::fs::File;
use std::io::{ Read, Write };

pub mod image_encoding {
  tonic::include_proto!("image_encoding");
}

pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
  ImageEncoderClient::connect("http://10.7.17.128:50051").await.unwrap()
}

pub async fn image_encode(
  client: &mut ImageEncoderClient<tonic::transport::Channel>,
  image_path: &str,
  width: i32,
  height: i32
) -> String {
  let mut image_file = File::open(image_path).unwrap();
  let mut image_data = Vec::new();
  image_data = image_file.bytes().map(|byte| byte.unwrap()).collect();

  let request = tonic::Request::new( EncodedImageRequest {
    width: width,
    height: height,
    image_data: image_data,
  });

  let response = client.image_encode(request).await.unwrap();

  // write encoded image to path
  let mut encoded_image_file = File::create("encoded_image.png").unwrap();
  encoded_image_file.write_all(&response.get_ref().image_data).unwrap();

  "Encoded image saved to encoded_image.png".to_string()
}