use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{self, Read};
use std::time::Duration;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Bind the client socket to a fixed port to maintain a consistent address
    let main_socket = UdpSocket::bind("127.0.0.1:5000").await?; // Use a fixed port for client
    let server_main_addr = "127.0.0.1:8080";
    let image_path = "/home/bavly.remon2004@auc.egy/Downloads/catmeme.jpeg"; // Replace with your image path
    let mut file = File::open(image_path)?;
    let mut buf = Vec::new();

    // Read the image file into a buffer
    file.read_to_end(&mut buf)?;

    // Send a "send request" to the server
    main_socket.send_to(&[1], server_main_addr).await?;
    println!("Sent image transfer request to server");

    // Receive the new port number from the server
    let mut port_buf = [0; 2];
    let (_, _) = main_socket.recv_from(&mut port_buf).await?;
    let new_port = u16::from_be_bytes(port_buf);
    let server_addr = format!("127.0.0.1:{}", new_port);
    println!("Received new port from server: {}", new_port);

    // Now continue to use the same `main_socket` for transferring data on the assigned port
    // Instead of binding a new socket, we reuse `main_socket`

    // Send image chunks with acknowledgment
    let max_packet_size = 1022;
    let mut packet_number: u16 = 0;

    for chunk in buf.chunks(max_packet_size) {
        let mut packet = Vec::with_capacity(2 + chunk.len());
        packet.extend_from_slice(&packet_number.to_be_bytes());
        packet.extend_from_slice(chunk);

        // Retry loop for sending each packet until ACK is received
        loop {
            main_socket.send_to(&packet, &server_addr).await?;
            println!("Sent packet {}", packet_number);

            let mut ack_buf = [0; 2];
            match tokio::time::timeout(Duration::from_secs(1), main_socket.recv_from(&mut ack_buf)).await {
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

    // Send the end-of-transmission signal
    let terminator = [255, 255];
    main_socket.send_to(&terminator, &server_addr).await?;
    println!("All packets sent and end signal sent.");

    Ok(())
}
