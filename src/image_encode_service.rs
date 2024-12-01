use crate::utils::get_file_name;
use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::EncodedImageRequest;
use leader_provider::leader_provider_client::LeaderProviderClient;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::fs::{self, File};
use std::net::SocketAddr;
use std::path::Path;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use steganography::util::{bytes_to_file, file_to_bytes};

// TODO: put these in a config file or environment variables
const MAX_SERVICE_ATTEMPTS: u32 = 3;
// maximum number of times a client can connect to a server, and on server connect, that server fails to provide leader
// const MAX_LEADER_PROVIDER_ATTEMPTS: u32 = 3;
// const MAX_LEADER_PROVIDER_AND_SERVICE_ATTEMPTS: u32 = 3;
const MAX_LEADER_ATTEMPTS: u32 = 3;

pub mod image_encoding {
    tonic::include_proto!("image_encoding");
}

pub mod leader_provider {
    tonic::include_proto!("leader_provider");
}

async fn get_leader_provider_client(
    server_list: Vec<&str>,
) -> Option<LeaderProviderClient<tonic::transport::Channel>> {
    for _ in 0..MAX_SERVICE_ATTEMPTS {
        let start = SystemTime::now();
        let duration = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        // Use the current time in nanoseconds as the seed (you can also use seconds or milliseconds)
        let seed = duration.as_nanos() as u64; // Convert nanoseconds to u64

        // Create a seedable RNG from the system time
        let mut rng = StdRng::seed_from_u64(seed);
        let mut random_number = rng.gen_range(0..server_list.len()); // Correcting to use the length of the list
        let prev_random_number = rng.gen_range(0..server_list.len());
        while random_number != prev_random_number {
            random_number = rng.gen_range(0..server_list.len());
        }
        for i in 0..server_list.len() {
            let socket = format!(
                "http://{}:50051",
                server_list[(random_number + i) % server_list.len()]
            );
            match LeaderProviderClient::connect(socket.clone()).await {
                Ok(client) => {
                    println!("Connected to a provider at: {}", socket.clone());
                    return Some(client);
                }
                Err(e) => {
                    println!(
                        "Failed to connect to a provider server at: {}: {}",
                        socket, e
                    );
                }
            }
        }

        println!("All servers failed waiting for 50ms then retrying");
        thread::sleep(std::time::Duration::from_millis(50));
    }

    None
}

async fn persist_leader_provider_client(
    mut leader_provider_client: LeaderProviderClient<tonic::transport::Channel>,
    max_retries: u32,
) -> Option<String> {
    let request_data = leader_provider::LeaderProviderEmptyRequest {};
    let mut retry = 0;

    loop {
        let request = tonic::Request::new(request_data);
        match leader_provider_client.get_leader(request).await {
            Ok(response) => {
                return Some(response.get_ref().leader_socket.to_string());
            }
            Err(e) => {
                println!(
                    "Failed to get leader from the leader provider | Retrying |  Error: {}",
                    e
                );
                if retry >= max_retries {
                    return None;
                }
                retry += 1;
            }
        }
    }
}

// async fn persist_on_service_and_leader_provider(server_list: Vec<&str>) -> Option<String> {
//     for _ in 0..MAX_LEADER_PROVIDER_AND_SERVICE_ATTEMPTS {
//         let leader_provider_client = get_leader_provider_client(server_list.clone()).await?;
//         let leader_socket = persist_leader_provider_client(leader_provider_client, 0).await?;
//         return Some(leader_socket);
//     }

//     None
// }

pub async fn connect() -> ImageEncoderClient<tonic::transport::Channel> {
    let args: Vec<String> = std::env::args().collect();

    let do_random_selection = args.iter().any(|arg| arg == "--random-selection");

    // TODO: server_list replaced with querying service directory
    let server_list: Vec<&str> = vec!["10.7.16.11", "10.7.17.128", "10.7.16.54", "10.7.17.155"];

    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(0..server_list.len()); // Correcting to use the length of the list
    let random_socket = format!("http://{}:50051", server_list[random_number]);
    if do_random_selection {
        // Attempt to connect to the server
        return ImageEncoderClient::connect(random_socket).await.unwrap();
    }

    let leader_provider_client = get_leader_provider_client(server_list)
        .await
        .unwrap_or_else(|| {
            println!("Failed to connect to any leader provider server");
            std::process::exit(1);
        });

    let response = persist_leader_provider_client(leader_provider_client, 0)
        .await
        .unwrap();
    let leader_socket = format!("http://{}", response);
    println!("Leader socket: {}", leader_socket);
    let mut image_encode_client = None;

    for _ in 0..MAX_LEADER_ATTEMPTS {
        match ImageEncoderClient::connect(leader_socket.clone()).await {
            Ok(client) => {
                image_encode_client = Some(client);
                break;
            }
            Err(e) => {
                println!(
                    "Failed to connect to leader server at: {} | Error: {}, Exiting Gracefully",
                    leader_socket, e
                );
                // std::process::exit(1);
            }
        }
    }

    image_encode_client.unwrap()
}

fn create_directory_if_not_exists(dir_path: &str) -> std::io::Result<()> {
    // Convert the dir_path to a Path
    let path = Path::new(dir_path);

    // Create the directory (and any parent directories) if it doesn't exist
    fs::create_dir_all(path)?;

    println!("Directory '{}' created or already exists.", dir_path);

    Ok(())
}
// use crate::image_decode::decode_image;
pub async fn image_encode(
    client: &mut ImageEncoderClient<tonic::transport::Channel>,
    image_path: &str,
    user_name: &str,
    addr: SocketAddr,
) -> String {
    let image_file = File::open(image_path).unwrap();
    let image_data = file_to_bytes(image_file);

    let request = tonic::Request::new(EncodedImageRequest {
        image_data: image_data.clone(),
        file_name: get_file_name(image_path).clone(),
        user_name: user_name.to_string(),
        client_server_socket: addr.to_string(),
    });

    println!("Sending ...");

    let response = match client.image_encode(request).await {
        Ok(response) => {
            //println!("Response: {:?}", response);
            response
        }
        Err(e) => {
            println!("Failed to encode image: {}", e);
            let mut new_client = connect().await;

            let new_request = tonic::Request::new(EncodedImageRequest {
                image_data,
                file_name: get_file_name(image_path),
                user_name: user_name.to_string(),
                client_server_socket: addr.to_string(),
            });
            new_client.image_encode(new_request).await.unwrap()
        }
    };

    println!("Sent!");

    let encoded_data = &response.get_ref().image_data;

    // new file for output
    // let output_file_path = "encoded_image.png";
    // output_file_path will be encoded/{image_name}.png
    let output_file_path = format!("./encoded/{}", get_file_name(image_path));
    create_directory_if_not_exists("./encoded").unwrap();
    let file = File::create(output_file_path).unwrap(); // Unwrap the Result here

    bytes_to_file(encoded_data, &file);

    // let extraction_path = "./extracted";
    // let decode_return = decode_image(output_file_path.to_string(), user_name.to_string());

    // println!("Extracted file saved to: {}", extraction_path);

    // decode_return.unwrap().to_string()
    "Encoded image saved".to_string()
}
