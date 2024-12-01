// use rand::{thread_rng, Rng};
use reqwest::Client;
use rpc_client::image_decode::decode_image;
use rpc_client::image_encode_service::{connect, image_encode};
use rpc_client::unveil_image;
use serde::{Deserialize, Serialize};
// use serde_json;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self};
use std::net::TcpListener;
use std::path::Path;
use stegano_core::Message;
use tonic::transport::Server;

const FIREBASE_URL: &str =
    "https://firestore.googleapis.com/v1/projects/rustysharing-eb44c/databases/(default)/documents";

#[derive(Debug, Deserialize, Serialize)]
struct Owner {
    socket: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Image {
    ownerSocket: String,
    imgID: String,
    description: String,
}

#[derive(Debug)]
struct Share {
    imgID: String,
    userSocket: String,
    viewCount: i64,
}

fn find_available_port() -> Result<u16, io::Error> {
    for port in 8000..9000 {
        let addr = format!("127.0.0.1:{}", port);
        match TcpListener::bind(&addr) {
            Ok(_) => {
                return Ok(port); // Successfully bound, port is available
            }
            Err(_) => {
                continue; // Port is in use, try the next one
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No available ports found",
    ))
}
use peer_to_peer::peer_to_peer_server::{PeerToPeer, PeerToPeerServer};
use peer_to_peer::{PeerToPeerRequest, PeerToPeerResponse};
use stegano_core::{Hide, Media, Persist};
use steganography::util::{bytes_to_file, file_to_bytes};
use tonic::{Request, Response, Status};

pub mod peer_to_peer {
    tonic::include_proto!("peer_to_peer");
}

use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

fn dynamic_image_to_bytes(img: &DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), format)
        .expect("Failed to write image to bytes");
    bytes
}

#[derive(Debug, Default)]
pub struct MyPeerToPeer {}
use stegano_core::{CodecOptions, SteganoError};
#[tonic::async_trait]
impl PeerToPeer for MyPeerToPeer {
    async fn peer_to_peer(
        &self,
        request: Request<PeerToPeerRequest>,
    ) -> Result<Response<PeerToPeerResponse>, Status> {
        let request = request.into_inner();
        println!("Got a request: {:?}", request);

        // encode the image with requester_user_name and requested_views
        let new_text = format!(
            "{} {}",
            request.requester_user_name, request.requested_views
        );
        let image_path: String = format!("./encoded/{}", request.requested_image_name);

        let mut message = Message::empty();
        message.add_file_data("view_count.txt", new_text.into_bytes());
        let image = unveil_image(Path::new(&image_path), &CodecOptions::default());
        let image = image.unwrap();
        message.add_file_data(
            "image.png",
            dynamic_image_to_bytes(&image, ImageFormat::PNG),
        );
        create_directory_if_not_exists("./encoded").unwrap();
        let mut media = Media::from_file(Path::new(&image_path)).unwrap();
        media.hide_message(&message).unwrap();
        // also save the image that was inside

        media.save_as(Path::new(&image_path)).unwrap();

        let file = File::open(image_path.clone())?;
        let encoded_bytes = file_to_bytes(file);

        let reply = PeerToPeerResponse {
            image_data: encoded_bytes,
        };
        Ok(Response::new(reply))
    }
}

fn extract_doc_id(path: &str) -> String {
    path.split('/').last().unwrap_or("").to_string()
}

async fn display_ownership_hierarchy(client: &Client) -> Result<(), Box<dyn Error>> {
    println!("\n=== Current Ownership Hierarchy ===\n");
    let owners = get_all_owners(&client).await?;
    // println!("Owners: {:?}", owners);
    if owners.is_empty() {
        println!("No owners found in the database.");
        return Ok(());
    }

    for (user_name, owner_id, client_socket_addr) in owners {
        println!("👤 Owner: {} {}", user_name, client_socket_addr);
        let images = get_images_for_owner(&client, &owner_id).await?;
        if images.is_empty() {
            println!(" └─ No images owned\n");
        } else {
            for (i, (image, _image_id)) in images.iter().enumerate() {
                let is_last = i == images.len() - 1;
                let prefix = if is_last { " └─ " } else { " ├─ " };
                println!("{}📷 Image doc ID: {}", prefix, image.imgID);
                println!(
                    "{} Owner's local file name: {}",
                    if is_last { " " } else { " │ " },
                    image.description
                );
            }
            println!(); // Add spacing between owners
        }
    }

    println!("=== End of Hierarchy ===\n");
    Ok(())
}

async fn get_all_owners(client: &Client) -> Result<Vec<(String, String, String)>, Box<dyn Error>> {
    let response = client.get(format!("{}/users", FIREBASE_URL)).send().await?;
    if !response.status().is_success() {
        return Err("Failed to fetch owners".into());
    }
    let data: serde_json::Value = response.json().await?;
    // println!("data: {:?}", data);
    let owners = data["documents"]
        .as_array()
        .ok_or("No documents found")?
        .iter()
        .map(|doc| {
            let user_name = doc["fields"]["user_name"]["stringValue"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let client_server_socket = doc["fields"]["client_server_socket"]["stringValue"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let id = extract_doc_id(doc["name"].as_str().unwrap_or(""));
            (user_name, id, client_server_socket)
        })
        .collect::<Vec<_>>();
    Ok(owners)
}

async fn get_images_for_owner(
    client: &Client,
    owner_id: &str,
) -> Result<Vec<(Image, String)>, Box<dyn Error>> {
    let response = client
        .get(format!("{}/users/{}/images", FIREBASE_URL, owner_id))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err("Failed to fetch images".into());
    }
    let data: serde_json::Value = response.json().await?;
    let images = data["documents"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|doc| {
            let img_id = doc["fields"]["img_id"]["stringValue"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let description = doc["fields"]["description"]["stringValue"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let doc_id = extract_doc_id(doc["name"].as_str().unwrap_or(""));
            let image = Image {
                ownerSocket: owner_id.to_string(),
                imgID: img_id,
                description,
            };
            (image, doc_id)
        })
        .collect();
    Ok(images)
}

fn create_directory_if_not_exists(dir_path: &str) -> std::io::Result<()> {
    // Convert the dir_path to a Path
    let path = Path::new(dir_path);

    // Create the directory (and any parent directories) if it doesn't exist
    fs::create_dir_all(path)?;

    println!("Directory '{}' created or already exists.", dir_path);

    Ok(())
}

// Client for peer to peer communication
use peer_to_peer::peer_to_peer_client::PeerToPeerClient;
// use peer_to_peer::PeerToPeerRequest;

// pub mod peer_to_peer {
//     tonic::include_proto!("peer_to_peer");
// }

async fn user_interaction(client: &mut Client, user_name: String) -> Result<(), Box<dyn Error>> {
    loop {
        let owners = get_all_owners(&client).await?;
        let mut images = Vec::new();
        // read images from ./images directory
        let img_dir = "./images";
        let entries = fs::read_dir(img_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let image_data = fs::read(path.clone())?;
            images.push((file_name.to_string(), image_data));
        }

        println!("Select an option:");
        println!("1. View an image you have");
        println!("2. Request a new image");
        println!("3. Exit");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        match choice {
            "1" => {
                if images.is_empty() {
                    println!("You have no images.");
                    continue;
                }

                println!("Select an image to view:");
                for (i, (name, _)) in images.iter().enumerate() {
                    println!("{}. {}", i + 1, name);
                }

                let mut image_choice = String::new();
                io::stdin().read_line(&mut image_choice)?;
                let image_choice: usize = image_choice.trim().parse().unwrap_or(0);

                if image_choice == 0 || image_choice > images.len() {
                    println!("Invalid choice.");
                    continue;
                }

                let (image_name, _) = &images[image_choice - 1];
                let image_path = format!("{}/{}", img_dir, image_name);
                match decode_image(image_path, user_name.clone()) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => println!("Failed to display image: {}", e),
                }
            }
            "2" => {
                println!("Select an owner:");
                for (i, owner) in owners.iter().enumerate() {
                    println!("{}. {}", i + 1, owner.0);
                }

                let mut owner_choice = String::new();
                io::stdin().read_line(&mut owner_choice)?;
                let owner_choice: usize = owner_choice.trim().parse().unwrap_or(0);

                if owner_choice == 0 || owner_choice > owners.len() {
                    println!("Invalid choice.");
                    continue;
                }

                let owner = &owners[owner_choice - 1];

                // Get owner's images
                let owner_images = get_images_for_owner(&client, &owner.1).await?;
                if owner_images.is_empty() {
                    println!("This owner has no images.");
                    continue;
                }

                println!("Select an image to request:");
                for (i, (image, _)) in owner_images.iter().enumerate() {
                    println!("{}. {}", i + 1, image.description);
                }

                let mut image_choice = String::new();
                io::stdin().read_line(&mut image_choice)?;
                let image_choice: usize = image_choice.trim().parse().unwrap_or(0);

                if image_choice == 0 || image_choice > owner_images.len() {
                    println!("Invalid choice.");
                    continue;
                }

                let (selected_image, _) = &owner_images[image_choice - 1];

                println!("Enter the number of views to request:");
                let mut view_count = String::new();
                io::stdin().read_line(&mut view_count)?;
                let view_count = view_count.trim();

                let owner_socket = &owner.2;
                let owner_socket = format!("http://{}", owner_socket);
                let mut client = PeerToPeerClient::connect(owner_socket.to_string()).await?;
                client = client.max_decoding_message_size(100 * 1024 * 1024);
                let request = tonic::Request::new(PeerToPeerRequest {
                    requester_user_name: user_name.clone(),
                    requested_image_name: selected_image.description.clone(),
                    requested_views: view_count.to_string(),
                });

                let response = client.peer_to_peer(request).await?;
                let encoded_data = &response.get_ref().image_data;

                let img_dir = "./images";
                create_directory_if_not_exists(img_dir)?;
                let image_path = format!("{}/{}", img_dir, selected_image.description);
                let file = File::create(image_path.clone())?;
                bytes_to_file(encoded_data, &file);
            }
            "3" => {
                println!("Exiting.");
                break;
            }
            _ => {
                println!("Invalid choice.");
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <user_name> <img_dir>", args[0]);
        return Ok(());
    }

    let user_name = &args[1];
    let img_dir = &args[2];

    // Initialize networking
    let ip = local_ip::get().unwrap();
    let myport = find_available_port().unwrap_or(50051);
    let addr = format!("{}:{}", ip, myport).parse()?;

    // Set up server
    let mut client = connect().await.max_decoding_message_size(100 * 1024 * 1024);

    // Process all images in the directory
    let entries = fs::read_dir(img_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "png" || extension == "jpg" || extension == "jpeg" {
                    println!("Encoding image: {:?}", path);
                    let path_str = path.to_str().unwrap();
                    let response = image_encode(&mut client, path_str, user_name, addr).await;
                    println!("RESPONSE={:?}", response);
                }
            }
        }
    }

    // Start server
    tokio::spawn(async move {
        Server::builder()
            .add_service(PeerToPeerServer::new(MyPeerToPeer::default()))
            .serve(addr)
            .await
            .unwrap();
    });

    println!("Server is running at {}", addr);
    let mut client = Client::new();
    println!("Initial database state:");
    display_ownership_hierarchy(&client).await?;

    create_directory_if_not_exists("./images")?;
    user_interaction(&mut client, user_name.to_string()).await?;

    Ok(())
}
