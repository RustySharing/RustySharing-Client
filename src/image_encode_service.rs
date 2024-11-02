use image_encoding::image_encoder_client::ImageEncoderClient;
use image_encoding::EncodedImageRequest;
use std::fs::File;
use std::io::{Read, Write};
use tokio::sync::mpsc;
use tokio::task;

pub mod image_encoding {
    tonic::include_proto!("image_encoding");
}

// List of server IPs for multicasting
const SERVER_IPS: &[&str] = &["http://10.7.17.128:50051", "http://10.7.16.11:50051"];

pub async fn connect(server_ip: &str) -> ImageEncoderClient<tonic::transport::Channel> {
    ImageEncoderClient::connect(server_ip.to_string()).await.unwrap()
}

pub async fn image_encode(image_path: &str, width: i32, height: i32) -> String {
    // Read image data from file
    let mut image_file = File::open(image_path).unwrap();
    let mut image_data = Vec::new();
    image_file.read_to_end(&mut image_data).unwrap();

    // Channel to collect responses
    let (tx, mut rx) = mpsc::channel(SERVER_IPS.len());

    // Send requests to all servers concurrently
    for &server_ip in SERVER_IPS {
        let tx = tx.clone();
        let image_data = image_data.clone();

        // Spawn a task for each server
        task::spawn(async move {
            // Connect to the server and send the request
            let mut client = connect(server_ip).await;
            let request = tonic::Request::new(EncodedImageRequest {
                width,
                height,
                image_data,
            });
            match client.image_encode(request).await {
                Ok(response) => {
                    // Send the response back to the main task
                    let _ = tx.send(response.into_inner().image_data).await;
                }
                Err(e) => eprintln!("Failed to send request to {}: {:?}", server_ip, e),
            }
        });
    }

    // Drop the original sender so the channel closes when all tasks are done
    drop(tx);

    // Wait for the first response and save it to file
    if let Some(encoded_image_data) = rx.recv().await {
        let mut encoded_image_file = File::create("encoded_image.png").unwrap();
        encoded_image_file.write_all(&encoded_image_data).unwrap();
        "Encoded image saved to encoded_image.png".to_string()
    } else {
        "Failed to receive any responses".to_string()
    }
}
