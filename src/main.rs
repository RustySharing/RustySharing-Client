use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{self, Read};
use std::time::Duration;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8081").await?;
    let server_addr = "127.0.0.1:8080";
    let image_path = "/home/bavly.remon2004@auc.egy/Downloads/catmeme.jpeg"; // Replace with your image path
    let mut file = File::open(image_path)?;
    let mut buf = Vec::new();

    // Read the image file into a buffer
    file.read_to_end(&mut buf)?;

    let max_packet_size = 1024; // Size of each UDP packet
    let mut packet_number: u16 = 0;

    for chunk in buf.chunks(max_packet_size) {
        let mut packet = Vec::with_capacity(2 + chunk.len());
        packet.extend_from_slice(&packet_number.to_be_bytes());
        packet.extend_from_slice(chunk);

        // Retry loop for sending each packet until ACK is received
        loop {
            // Send the packet
            socket.send_to(&packet, server_addr).await?;
            println!("Sent packet {}", packet_number);

            // Wait for acknowledgment with a timeout
            let mut ack_buf = [0; 2];
            match tokio::time::timeout(Duration::from_secs(1), socket.recv_from(&mut ack_buf)).await {
                Ok(Ok((_, _))) => {
                    let ack_packet_number = u16::from_be_bytes(ack_buf);
                    if ack_packet_number == packet_number {
                        println!("Acknowledgment received for packet {}", packet_number);
                        break;
                    }
                }
                _ => {
                    println!("No acknowledgment received for packet {}, resending...", packet_number);
                }
            }
        }

        packet_number += 1;
    }

    // Send a final packet to indicate the end of the transmission
    let terminator = [255, 255]; // Use a special sequence to indicate the end
    socket.send_to(&terminator, server_addr).await?;
    println!("All packets sent and end signal sent.");

    Ok(())
}

