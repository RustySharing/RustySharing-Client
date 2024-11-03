use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use csv::Writer;
use tokio::net::UdpSocket;
use tokio::task;

#[derive(Serialize, Deserialize, Debug)]
struct EmbeddedData {
    message: String,
    timestamp: String,
}

const CLIENT_COUNT: usize = 10; // Number of clients
const IMAGES_PER_CLIENT: usize = 100; // Images per client

#[tokio::main]
async fn main() -> io::Result<()> {
    // Load all image paths from the specified folder
    let all_images: Vec<PathBuf> = std::fs::read_dir("/home/bavly.remon2004@auc.egy/Downloads/jpeg_images/jpeg_images")
        .unwrap()
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();

    // Ensure there are enough images for each client
    assert!(all_images.len() >= CLIENT_COUNT * IMAGES_PER_CLIENT, "Not enough images for each client.");

    // Divide images among clients
    let images_per_client = all_images.chunks(IMAGES_PER_CLIENT)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();

    // Spawn multiple client tasks
    let mut client_handles = Vec::new();
    for i in 0..CLIENT_COUNT {
        let images = images_per_client[i].clone();
        client_handles.push(task::spawn(async move {
            simulate_client(i, images).await
        }));
    }

    // Wait for all clients to complete
    for handle in client_handles {
        handle.await??;
    }

    println!("All clients finished sending images.");
    Ok(())
}

// Function to simulate a single client
async fn simulate_client(client_id: usize, images: Vec<PathBuf>) -> io::Result<()> {
    let client_socket = UdpSocket::bind("0.0.0.0:0").await?;
    client_socket.set_multicast_ttl_v4(1)?;
    client_socket.send_to(&[1], (Ipv4Addr::new(239,255,0,1),9001)).await?;
    println!("Client {} sent multicast image transfer request", client_id);

    let mut response_buf = [0; 6];
    let (len, server_addr) = client_socket.recv_from(&mut response_buf).await?;

    if len == 6 {
        let ip = Ipv4Addr::new(response_buf[0], response_buf[1], response_buf[2], response_buf[3]);
        let port = u16::from_be_bytes([response_buf[4], response_buf[5]]);
        let server_image_addr = SocketAddr::new(ip.into(), port);
        println!("Client {} received server IP {} and port {}", client_id, ip, port);

        // Create a CSV writer to log results
        let mut csv_writer = Writer::from_path(format!("client_{}_log.csv", client_id))?;
        csv_writer.write_record(&["Request Number", "Response Time (ms)", "Failure"])?;

        // Track failures
        let mut failure_count = 0;

        for (index, image_path) in images.iter().enumerate() {
            let (response_time, success) = send_image_to_server(&client_socket, server_image_addr, client_id, image_path, index + 1).await?;
            csv_writer.write_record(&[
                (index + 1).to_string(),
                response_time.as_millis().to_string(),
                if success { "0".to_string() } else { "1".to_string() },
            ])?;

            if !success {
                failure_count += 1;
            }
        }

        csv_writer.flush()?;
        println!("Client {} finished with {} failures.", client_id, failure_count);
    } else {
        println!("Client {} received invalid response.", client_id);
    }

    Ok(())
}

// Function to send an image to the server and measure response time
async fn send_image_to_server(socket: &UdpSocket, server_addr: SocketAddr, client_id: usize, image_path: &PathBuf, image_number: usize) -> io::Result<(Duration, bool)> {
    let mut file = File::open(image_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let max_packet_size = 1022;
    let mut packet_number: u16 = 0;
    let mut success = true;
    let start_time = Instant::now();

    for chunk in buf.chunks(max_packet_size) {
        let mut packet = Vec::with_capacity(2 + chunk.len());
        packet.extend_from_slice(&packet_number.to_be_bytes()); // Include packet number
        packet.extend_from_slice(chunk); // Include data

        loop {
            socket.send_to(&packet, server_addr).await?;
            println!("Client {} sent packet {} for image {}", client_id, packet_number, image_number);

            let mut ack_buf = [0; 2];
            match tokio::time::timeout(Duration::from_secs(1), socket.recv_from(&mut ack_buf)).await {
                Ok(Ok((_, _))) => {
                    let ack_packet_number = u16::from_be_bytes(ack_buf);
                    if ack_packet_number == packet_number {
                        println!("Client {} received ack for packet {} of image {}", client_id, packet_number, image_number);
                        break;
                    }
                }
                _ => {
                    println!("Client {} no ack for packet {}, resending...", client_id, packet_number);
                    success = false;
                }
            }
        }

        packet_number += 1;
    }

    // Send end-of-transmission signal
    let terminator = [255, 255];
    socket.send_to(&terminator, server_addr).await?;
    println!("Client {} sent end signal for image {}", client_id, image_number);

    let response_time = start_time.elapsed();
    Ok((response_time, success))
}