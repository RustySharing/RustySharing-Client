// pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
//     let args: Vec<String> = std::env::args().collect();

//     let do_random_selection = args.iter().any(|arg| arg == "--random-selection");

//     // TODO: server_list replaced with querying service directory
//     let server_list: Vec<&str> = vec!["10.7.16.11", "10.7.17.128", "10.7.16.54", "10.7.17.155"];

//     let mut rng = rand::thread_rng();
//     let random_number = rng.gen_range(0..server_list.len()); // Correcting to use the length of the list
//     let random_socket = format!("http://{}:50051", server_list[random_number]);
//     if do_random_selection {
//         // Attempt to connect to the server
//         return ImageEncoderClient::connect(random_socket).await.unwrap();
//     }

//     let leader_provider_client = get_leader_provider_client(server_list)
//         .await
//         .unwrap_or_else(|| {
//             println!("Failed to connect to any leader provider server");
//             std::process::exit(1);
//         });

//     let response = persist_leader_provider_client(leader_provider_client, 0)
//         .await
//         .unwrap();
//     let leader_socket = format!("http://{}", response);
//     println!("Leader socket: {}", leader_socket);
//     let mut image_encode_client = None;

//     for _ in 0..MAX_LEADER_ATTEMPTS {
//         match ImageEncoderClient::connect(leader_socket.clone()).await {
//             Ok(client) => {
//                 image_encode_client = Some(client);
//                 break;
//             }
//             Err(e) => {
//                 println!(
//                     "Failed to connect to leader server at: {} | Error: {}, Exiting Gracefully",
//                     leader_socket, e
//                 );
//                 // std::process::exit(1);
//             }
//         }
//     }

//     image_encode_client.unwrap()
// }

// // use crate::image_decode::decode_image;
// pub async fn image_encode(
//     client: &mut ImageEncoderClient<tonic::transport::Channel>,
//     image_path: &str,
//     user_name: &str,
//     addr: SocketAddr,
// ) -> String {
//     let image_file = File::open(image_path).unwrap();
//     let image_data = file_to_bytes(image_file);

//     let request = tonic::Request::new(EncodedImageRequest {
//         image_data: image_data.clone(),
//         file_name: get_file_name(image_path).clone(),
//         user_name: user_name.to_string(),
//         client_server_socket: addr.to_string(),
//     });

//     println!("Sending ...");

//     let response = match client.image_encode(request).await {
//         Ok(response) => {
//             //println!("Response: {:?}", response);
//             response
//         }
//         Err(e) => {
//             println!("Failed to encode image: {}", e);
//             let mut new_client = connect().await;

//             let new_request = tonic::Request::new(EncodedImageRequest {
//                 image_data,
//                 file_name: get_file_name(image_path),
//                 user_name: user_name.to_string(),
//                 client_server_socket: addr.to_string(),
//             });
//             new_client.image_encode(new_request).await.unwrap()
//         }
//     };

//     println!("Sent!");

//     let encoded_data = &response.get_ref().image_data;

//     // new file for output
//     let output_file_path = "encoded_image.png";

//     let file = File::create(output_file_path).unwrap(); // Unwrap the Result here

//     bytes_to_file(encoded_data, &file);

//     // let extraction_path = "./extracted";
//     // let decode_return = decode_image(output_file_path.to_string(), user_name.to_string());

//     // println!("Extracted file saved to: {}", extraction_path);

//     // decode_return.unwrap().to_string()
//     "Encoded image saved".to_string()
// }
