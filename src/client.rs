use rpc_client::image_encode_service::{connect, image_encode};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let user_name = &args[1];
    //let image_path = &args[2];

    let mut client = connect().await.max_decoding_message_size(100 * 1024 * 1024); // Set to 10 MB
    let response = image_encode(&mut client, "./input_image.png", user_name).await;

    println!("RESPONSE={:?}", response);

    // Start the GUI
    // start_gui();

    Ok(())
}
