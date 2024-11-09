use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::EncodedImageRequest;
use leader_provider::leader_provider_client::LeaderProviderClient;
use std::fs::File;
use rand::Rng;
use steganography::util::{ bytes_to_file, file_to_bytes };
use crate::utils::get_file_name;
use std::thread;

// TODO: put these in a config file or environment variables
const MAX_SERVICE_ATTEMPTS: u32 = 3;
// maximum number of times a client can connect to a server, and on server connect, that server fails to provide leader
//const MAX_LEADER_PROVIDER_ATTEMPTS: u32 = 3;

pub mod image_encoding {
  tonic::include_proto!("image_encoding");
}

pub mod leader_provider {
  tonic::include_proto!("leader_provider");
}

async fn get_leader_provider_client(
  server_list: Vec<&str>
) -> Option<LeaderProviderClient<tonic::transport::Channel>> {
  for _ in 0..MAX_SERVICE_ATTEMPTS {
    for server in &server_list {
      let socket = format!("http://{}:50051", server);

      match LeaderProviderClient::connect(socket.clone()).await {
        Ok(client) => {
          println!("Connected to a provider at: {}", socket.clone());
          return Some(client);
        }
        Err(e) => {
          println!("Failed to connect to a provider server at: {}: {}", socket, e);
        }
      }
    }

    println!("All servers failed waiting for 50ms then retrying");
    thread::sleep(std::time::Duration::from_millis(50));
  }

  None
}

// async fn persist_leader_provider_client(
//   leader_provider_client: LeaderProviderClient<tonic::transport::Channel>
// ) -> LeaderProviderClient<tonic::transport::Channel> {
//   let request = tonic::Request::new(leader_provider::LeaderRequest {});
//   let response = leader_provider_client.get_leader(request).await.unwrap();
// }

pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
  let args: Vec<String> = std::env::args().collect();

  let do_random_selection = args.iter().any(|arg| arg == "--random-selection");

  // TODO: server_list replaced with querying service directory
  // let server_list: Vec<&str> = vec!["10.7.16.11", "10.7.17.128", "10.7.16.54"];
  let server_list: Vec<&str> = vec!["[::1]"];

  let mut rng = rand::thread_rng();
  let random_number = rng.gen_range(0..server_list.len()); // Correcting to use the length of the list
  let random_socket = format!("http://{}:50051", server_list[random_number]);

  if do_random_selection {
    // Attempt to connect to the server
    return ImageEncoderClient::connect(random_socket).await.unwrap();
  }

  let mut leader_provider_client = get_leader_provider_client(server_list).await.unwrap_or_else(|| {
    panic!("Failed to connect to any leader provider server");
  });
  let request = tonic::Request::new(leader_provider::LeaderProviderEmptyRequest {});
  let response = leader_provider_client.get_leader(request).await.unwrap();
  let leader_socket = format!("http://{}:50051", response.get_ref().leader_socket);
  println!("Leader socket: {}", leader_socket);
  ImageEncoderClient::connect(leader_socket).await.unwrap()
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
