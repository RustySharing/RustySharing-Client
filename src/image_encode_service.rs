use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::EncodedImageRequest;
use std::fs::File;
use steganography::util::{ bytes_to_file, file_to_bytes };

pub mod image_encoding {
  tonic::include_proto!("image_encoding");
}

pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
  // find my leader service
  // talk to me if ur my leader * 3
  // whoever responds with i am ur leader, continue communicating with him and pass his socket to the connect
  // select a random and send to it if not doing election

  ImageEncoderClient::connect("http://[::1]:50051").await.unwrap()
}

use crate::image_decode::decode_image;
pub async fn image_encode(
  client: &mut ImageEncoderClient<tonic::transport::Channel>,
  image_path: &str
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
  // if let Err(e) = create_directory_if_not_exists(extraction_path) {
  //     eprintln!("Error creating directory: {}", e);
  // }
  // // Extract the hidden file from the image
  // let _ = unveil(
  //     Path::new(output_file_path),
  //     Path::new(extraction_path),
  //     &CodecOptions::default(),
  // );
  let decode_return = decode_image(output_file_path.to_string(), extraction_path.to_string());

  println!("Extracted file saved to: {}", extraction_path);

  decode_return.unwrap().to_string()
}
