use rpc_client::image_encode_service::{connect, image_encode};
use std::env;
use std::fs::File;
use std::io::{self};
use std::net::TcpListener;
use std::path::Path;
use stegano_core::Message;
use tonic::transport::Server;
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
    Server::builder()
        .add_service(PeerToPeerServer::new(MyPeerToPeer::default()))
        .serve(addr)
        .await?;

    println!("RESPONSE={:?}", response);

    // Start the GUI
    // start_gui();

    Ok(())
}
