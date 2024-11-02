use tokio::net::UdpSocket;
use std::fs::File;
use std::io::{self, Read, Write};
use std::time::Duration;
use std::net::{Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() -> io::Result<()> {
    // Multicast address and port where all servers are listening
    let multicast_addr: Ipv4Addr = "239.255.0.1".parse().unwrap();
    let multicast_port = 9001;
    let client_socket = UdpSocket::bind("0.0.0.0:0").await?; // Bind to any available port
    client_socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;

    // Set multicast TTL to ensure packet can propagate
    client_socket.set_multicast_ttl_v4(1)?;

    // Multicast "send request" to all servers
    client_socket.send_to(&[1], (multicast_addr, multicast_port)).await?;
    println!("Sent multicast image transfer request to all servers");

    // Wait for a response with the specific IP and port from the server with the talking stick
    let mut response_buf = [0; 6]; // 4 bytes for IP + 2 bytes for port
    let (len, server_addr) = client_socket.recv_from(&mut response_buf).await?;

    println!("Received response of length {} from {}: {:?}", len, server_addr, &response_buf[..len]);

    if len == 6 {
        let ip = Ipv4Addr::new(response_buf[0], response_buf[1], response_buf[2], response_buf[3]);
        let port = u16::from_be_bytes([response_buf[4], response_buf[5]]);
        let server_image_addr = SocketAddr::new(ip.into(), port);
        println!("Received response from server with IP {} and port {}", ip, port);

        // Proceed to send the image to the server on the provided IP and port using the same socket
        send_image_to_server(&client_socket, server_image_addr).await?;
    } else {
        println!("Invalid response received.");
    }

    Ok(())
}

// Function to send the image to the specified server address using the same socket
async fn send_image_to_server(socket: &UdpSocket, server_addr: SocketAddr) -> io::Result<()> {
    let image_path = "/home/bavly.remon2004@auc.egy/Downloads/orange.png"; // Replace with the path to your image
    let mut file = File::open(image_path)?;
    let mut buf = Vec::new();

    // Read the image file into a buffer
    file.read_to_end(&mut buf)?;

    // Send image chunks with acknowledgment
    let max_packet_size = 1022; // Packet size (1024 - 2 for sequence number)
    let mut packet_number: u16 = 0;

    for chunk in buf.chunks(max_packet_size) {
        let mut packet = Vec::with_capacity(2 + chunk.len());
        packet.extend_from_slice(&packet_number.to_be_bytes()); // Include packet number
        packet.extend_from_slice(chunk); // Include data

        // Retry loop for sending each packet until ACK is received
        loop {
            socket.send_to(&packet, server_addr).await?;
            println!("Sent packet {}", packet_number);

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

        packet_number += 1; // Increment packet number for the next packet
    }

    // Send the end-of-transmission signal
    let terminator = [255, 255];
    socket.send_to(&terminator, server_addr).await?;
    println!("All packets sent and end signal sent.");

    // Step 2: Receive the image sent back from the server
    let mut received_packets = std::collections::HashMap::new();
    let mut total_packets = 0;

    loop {
        let mut buf = [0; 1024 + 2]; // Buffer for incoming data (packet size + 2 bytes for sequence number)
        let (len, _) = socket.recv_from(&mut buf).await?;

        if len == 2 && buf[0] == 255 && buf[1] == 255 {
            println!("End of transmission signal received.");
            break;
        }

        let packet_number = u16::from_be_bytes([buf[0], buf[1]]);
        let data = buf[2..len].to_vec();
        received_packets.insert(packet_number, data);
        total_packets = total_packets.max(packet_number + 1);

        socket.send_to(&packet_number.to_be_bytes(), server_addr).await?;
        println!("Acknowledgment sent for packet {}", packet_number);
    }

    // Reassemble the image data in the correct order
    let mut image_data = Vec::new();
    for i in 0..total_packets {
        if let Some(chunk) = received_packets.remove(&i) {
            image_data.extend(chunk);
        }
    }

    // Save the received image as a new file
    let mut file = File::create("received_image.jpeg")?;
    file.write_all(&image_data)?;
    println!("Image saved as 'received_image.jpeg'.");

    Ok(())
}