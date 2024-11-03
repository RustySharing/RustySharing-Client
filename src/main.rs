use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;

#[derive(Serialize, Deserialize, Debug)]
struct EmbeddedData {
    message: String,
    timestamp: String,
}

pub async fn client_send_image(image_path: &str) -> io::Result<()> {
    let multicast_addr: Ipv4Addr = "239.255.0.1".parse().unwrap();
    let multicast_port = 9001;
    let client_socket = UdpSocket::bind("0.0.0.0:0").await?;
    client_socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;

    client_socket.set_multicast_ttl_v4(1)?;
    client_socket.send_to(&[1], (multicast_addr, multicast_port)).await?;
    println!("Sent multicast image transfer request to all servers");

    let mut response_buf = [0; 6];
    let (len, server_addr) = client_socket.recv_from(&mut response_buf).await?;

    if len == 6 {
        let ip = Ipv4Addr::new(
            response_buf[0],
            response_buf[1],
            response_buf[2],
            response_buf[3],
        );
        let port = u16::from_be_bytes([response_buf[4], response_buf[5]]);
        let server_image_addr = SocketAddr::new(ip.into(), port);
        println!("Received response from server with IP {} and port {}", ip, port);

        send_image_to_server(&client_socket, server_image_addr, image_path).await?;
    } else {
        println!("Invalid response received.");
    }

    Ok(())
}

async fn send_image_to_server(socket: &UdpSocket, server_addr: SocketAddr, image_path: &str) -> io::Result<()> {
    let mut buf = Vec::new();
    let mut file = File::open(image_path)?;
    file.read_to_end(&mut buf)?;

    let max_packet_size = 1022;
    let mut packet_number: u16 = 0;

    for chunk in buf.chunks(max_packet_size) {
        let mut packet = Vec::with_capacity(2 + chunk.len());
        packet.extend_from_slice(&packet_number.to_be_bytes());
        packet.extend_from_slice(chunk);

        loop {
            socket.send_to(&packet, server_addr).await?;
            println!("Sent packet {}", packet_number);

            let mut ack_buf = [0; 2];
            if let Ok(Ok((_, _))) = tokio::time::timeout(Duration::from_secs(1), socket.recv_from(&mut ack_buf)).await {
                let ack_packet_number = u16::from_be_bytes(ack_buf);
                if ack_packet_number == packet_number {
                    println!("Acknowledgment received for packet {}", packet_number);
                    break;
                }
            }
        }

        packet_number += 1;
    }

    let terminator = [255, 255];
    socket.send_to(&terminator, server_addr).await?;
    println!("All packets sent and end signal sent.");

    receive_image(socket).await
}

async fn receive_image(socket: &UdpSocket) -> io::Result<()> {
    let mut received_packets = HashMap::new();
    let mut total_packets = 0;

    loop {
        let mut buf = [0; 1026];
        let (len, server_addr) = socket.recv_from(&mut buf).await?;
        if len == 2 && buf[0] == 255 && buf[1] == 255 {
            println!("End of transmission signal received.");
            break;
        }

        let packet_number = u16::from_be_bytes([buf[0], buf[1]]);
        let data = buf[2..len].to_vec();
        received_packets.insert(packet_number, data);
        total_packets = total_packets.max(packet_number + 1);

        socket.send_to(&packet_number.to_be_bytes(), server_addr).await?;
    }

    let mut image_data = Vec::new();
    for i in 0..total_packets {
        if let Some(chunk) = received_packets.remove(&i) {
            image_data.extend(chunk);
        }
    }

    let output_path = "received_image.png";
    let mut file = File::create(output_path)?;
    file.write_all(&image_data)?;
    println!("Image saved as 'received_image.png'.");

    decode_embedded_data(output_path)?;

    Ok(())
}

fn decode_embedded_data(image_path: &str) -> io::Result<()> {
    let decoded_img = image::open(image_path).expect("Failed to open encoded image");
    let my_decoder = steganography::decoder::Decoder::new(decoded_img.to_rgba());
    let decoded_data = my_decoder.decode_alpha();

    let start = decoded_data.iter().position(|&b| b == b'{').expect("Opening brace not found");
    let end = decoded_data.iter().position(|&b| b == b'}').expect("Closing brace not found");

    let json_part = &decoded_data[start..=end];
    let original_image_part = &decoded_data[end + 1..];

    let decoded_json: EmbeddedData = serde_json::from_slice(json_part).expect("Failed to parse JSON data");
    println!("Decoded Data: {:?}", decoded_json);

    let original_image_output_path = "extracted_original_image.png";
    std::fs::write(original_image_output_path, original_image_part)
        .expect("Failed to save the extracted original image");
    println!("Extracted original image saved as: {}", original_image_output_path);

    Ok(())
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let image_path = "orange.png"; // Replace with the path to your test image

    match client_send_image(image_path).await {
        Ok(_) => println!("Image successfully sent and received."),
        Err(e) => eprintln!("Error occurred: {}", e),
    }

    Ok(())
}
