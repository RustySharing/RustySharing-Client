// use rand::{thread_rng, Rng};
use reqwest::Client;
use rpc_client::image_encode_service::{connect, image_encode};
use serde::{Deserialize, Serialize};
// use serde_json;
use std::env;
use std::error::Error;
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
use steganography::util::file_to_bytes;
use tonic::{Request, Response, Status};

pub mod peer_to_peer {
    tonic::include_proto!("peer_to_peer");
}

#[derive(Debug, Default)]
pub struct MyPeerToPeer {}

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
        let mut message = Message::empty();
        message.add_file_data("view_count.txt", new_text.into_bytes());
        let mut media = Media::from_file(Path::new(&request.requested_image_name)).unwrap();
        media.hide_message(&message).unwrap();
        media
            .save_as(Path::new(&request.requested_image_name))
            .unwrap();

        let file = File::open(request.requested_image_name.clone())?;
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
            for (i, (image, image_id)) in images.iter().enumerate() {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let user_name = &args[1];
    //let image_path = &args[2];
    // start the client's server
    let ip = local_ip::get().unwrap();
    let mut myport: u16 = 50051;
    // Get a not busy port
    match find_available_port() {
        Ok(port) => {
            println!("Found available port: {}", port);
            myport = port;
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    let addr = format!("{}:{}", ip, myport).parse()?;
    let mut client = connect().await.max_decoding_message_size(100 * 1024 * 1024); // Set to 10 MB
    let response = image_encode(&mut client, "./input_image.png", user_name, addr).await;
    tokio::spawn(async move {
        Server::builder()
            .add_service(PeerToPeerServer::new(MyPeerToPeer::default()))
            .serve(addr)
            .await
            .unwrap();
    });
    println!("Server is running at {}", addr);
    let client = Client::new();
    println!("Initial database state:");
    display_ownership_hierarchy(&client).await?;
    println!("RESPONSE={:?}", response);

    // Start the GUI
    // start_gui();

    Ok(())
}
