use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::{ EncodedImageResponse, EncodedImageRequest };
use std::fs::File;

pub mod image_encoding {
  tonic::include_proto!("image_encoding");
}

pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
  ImageEncoderClient::connect("http://[::1]:50051").await.unwrap()
}

pub async fn image_encode(
  client: &mut ImageEncoderClient<tonic::transport::Channel>,
  image_path: &str,
  width: &i32,
  height: &i32
) -> String {
  let mut image_file = File::open(image_path).unwrap();
  let mut image_data = Vec::new();
  image_file.read_to_end(&mut image_data).unwrap();

  let request = tonic::Request::new( EncodedImageRequest {
    width: width,
    height: height,
    image_data: image_data,
  });

  let response = client.encode_image(request).await.unwrap();

  // write encoded image to path
  let mut encoded_image_file = File::create("encoded.png").unwrap();
  encoded_image_file.write_all(&response.into_inner().encoded_image).unwrap();

  response.into_inner().encoded_image
}