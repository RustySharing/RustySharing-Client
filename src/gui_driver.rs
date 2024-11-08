use crate::simple_request_service::{connect, send_hello};

use gtk::gio::File;
// TODO:: migrate gui_driver into its own file
use gtk::glib::clone;
use gtk::{glib, ApplicationWindow};
use gtk::{prelude::*, Picture};
use gtk::{Box as GtkBox, Button, Entry, Label, Orientation};
const APP_ID: &str = "org.gtk_rs.HelloClient";

pub fn start_gui() -> glib::ExitCode {
    // Create a new application
    let app = adw::Application::builder().application_id(APP_ID).build();
    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);
    app.run()
}

fn image_sender_widget() -> GtkBox {
    let entry = Entry::builder()
        .placeholder_text("Enter your image's Path")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let submit_entry = Button::builder()
        .label("Submit")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let picture = Picture::new();
    picture.set_size_request(500, 500);

    let path_label = Label::builder()
        .label("")
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

    box_.append(&entry);
    box_.append(&submit_entry);
    box_.append(&picture);
    box_.append(&path_label);

    submit_entry.connect_clicked(move |_| {
        let path = entry.text();
        let file = File::for_path(path.as_str());
        picture.set_file(Some(&file));
        path_label.set_label(path.as_str());
    });

    box_
}

fn build_ui(app: &adw::Application) {
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

    let image_sender = image_sender_widget();

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
    box_.append(&image_sender);

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
        .title("RustySharing")
        .child(&box_)
        .build();

    // Present window
    window.present();
}
