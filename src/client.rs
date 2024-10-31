use rpc_client::image_encode_service::{ connect, image_encode };
use rpc_client::gui_driver::start_gui;
use std::fs::File;
use std::fs::*;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

  let mut client = connect().await;

 // let mut image_file = File::open("/home/ahmedwaseemr@auc.egy/Downloads/trump7.jpeg").unwrap();
  //let mut image_data = Vec::new();
  //image_data = image_file.bytes().map(|byte| byte.unwrap()).collect();

  let response = image_encode(&mut client, "/home/weso/Documents/Rust/rpc_client/src/test/images/doctor.png", 2000, 3000).await;

  println!("RESPONSE={:?}", response);

  // Start the GUI
  start_gui();

  Ok(())
}
