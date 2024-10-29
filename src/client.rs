use hello::hello_client::HelloClient;
use hello::HelloRequest;

pub mod hello {
  tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

  let mut client = connect().await;
  let response = send_hello(&mut client, "I am a Request from client").await;

  println!("RESPONSE={:?}", response);

  // Start the GUI
  gui_driver();

  Ok(())
}

//TODO: modify this as were it gets the port of the service through directory discovery service
async fn connect() -> HelloClient<tonic::transport::Channel> {
  HelloClient::connect("http://[::1]:50051").await.unwrap()
}

// send hello request to server, and return the response as string
async fn send_hello(client: &mut HelloClient<tonic::transport::Channel>, name: &str) -> String {
  let request = tonic::Request::new(HelloRequest {
    name: name.into(),
  });

  let response = client.send_hello(request).await.unwrap();

  response.into_inner().message
}

// TODO:: migrate gui_driver into its own file
use crate::glib::clone;
use gtk::prelude::*;
use gtk::{ glib, Application, ApplicationWindow };
use gtk::{ Button, Label, Box as GtkBox, Orientation};

const APP_ID: &str = "org.gtk_rs.HelloClient";

fn gui_driver() -> glib::ExitCode {
  // Create a new application
  let app = Application::builder().application_id(APP_ID).build();
  // Connect to "activate" signal of `app`
  app.connect_activate(build_ui);
  app.run()
}

fn build_ui(app: &Application) {

    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let text_label = Label::builder()
        .label("Hello World!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .spacing(12)
        .build();

    box_.append(&text_label);
    box_.append(&button);

    // Connect to "clicked" signal of `button`
    button.connect_clicked(move |button| {
        text_label.set_label("Request Sent!, Awaiting Response...");
        // The main loop executes the asynchronous block
        glib::spawn_future_local(clone!(
            #[weak]
            button,
            #[weak]
            text_label,
            async move {
                // Deactivate the button until the operation is done
                button.set_sensitive(false);
                let response = send_hello(&mut connect().await, "I am a Request from client").await;
                button.set_sensitive(!response.is_empty());
                button.set_label("Response Received!");
                text_label.set_label(response.as_str());
            }
        ));
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&box_)
        .build();

    // Present window
    window.present();
}
