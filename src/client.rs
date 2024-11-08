use rpc_client::image_encode_service::{ connect, image_encode };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = connect().await.max_decoding_message_size(100 * 1024 * 1024); // Set to 10 MB
  let response = image_encode(&mut client, "./input_image.png").await;

  println!("RESPONSE={:?}", response);

  // Start the GUI
  // start_gui();

  Ok(())
}
