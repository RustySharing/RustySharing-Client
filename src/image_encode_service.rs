use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::EncodedImageRequest;
use leader_provider::leader_provider_client::LeaderProviderClient;
use std::fs::File;
use rand::Rng;
use steganography::util::{ bytes_to_file, file_to_bytes };
use crate::utils::get_file_name;
use std::thread;

pub mod image_encoding {
  tonic::include_proto!("image_encoding");
}

pub mod leader_provider {
  tonic::include_proto!("leader_provider");
}

pub async fn connect() -> LeaderProviderClient<tonic::transport::Channel> {}

pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
  let args: Vec<String> = std::env::args().collect();

  let do_random_selection = args.iter().any(|arg| arg == "--random-selection");
  // random server selection
  // TODO: replaced with querying service directory
  let server_list: Vec<&str> = vec!["10.7.16.11", "10.7.17.128", "10.7.16.54"];

  if do_random_selection {
    // Initialize random number generator
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(0..server_list.len()); // Correcting to use the length of the list

    // Format the connection string with the chosen server
    let address = format!("http://{}:50051", server_list[random_number]);

    // Attempt to connect to the server
    ImageEncoderClient::connect(address).await.unwrap();
  }

  // connect to all servers on different threads
  for server in server_list {
    thread::spawn(|| {
      let address = format!("http://{}:50051", server);
      LeaderProviderClient::connect(address).await.unwrap();
    });
  }

  // multicast to server list, 50052 will be port of leader election
  let address = format!("http://{}:50052".server_list[random_number]);
}

use crate::image_decode::decode_image;
pub async fn image_encode(
  client: &mut ImageEncoderClient<tonic::transport::Channel>,
  image_path: &str
) -> String {
  let image_file = File::open(image_path).unwrap();
  let image_data = file_to_bytes(image_file);

  let request = tonic::Request::new(EncodedImageRequest {
    image_data: image_data,
    file_name: get_file_name(image_path),
  });

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
