use rpc_client::simple_request_service::{ connect, send_hello };
use rpc_client::image_encode_service::{ connect, image_encode };
use rpc_client::gui_driver::start_gui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

  let mut client = connect().await;
  let response = image_encode(&mut client, "./test/images/kunst.png", &100, &100).await;

  println!("RESPONSE={:?}", response);

  // Start the GUI
  start_gui();

  Ok(())
}
