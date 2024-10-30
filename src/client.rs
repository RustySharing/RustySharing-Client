use rpc_service::simple_request_service::{ connect, send_hello };
use rpc_service::gui_driver::start_gui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

  let mut client = connect().await;
  let response = send_hello(&mut client, "I am a Request from client").await;

  println!("RESPONSE={:?}", response);

  // Start the GUI
  start_gui();

  Ok(())
}
