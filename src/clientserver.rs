// use std::fs::File;
// use std::path::Path;

// use stegano_core::{Hide, Media, Message, Persist};
// use steganography::util::file_to_bytes;
// use tonic::{Request, Response, Status};

// use hello_world::greeter_server::{Greeter, GreeterServer};
// use hello_world::{HelloReply, HelloRequest};

// pub mod hello_world {
//     tonic::include_proto!("helloworld"); // The string specified here must match the proto package name
// }
// use peer_to_peer::peer_to_peer_server::{PeerToPeer, PeerToPeerServer};
// use peer_to_peer::{PeerToPeerRequest, PeerToPeerResponse};

// pub mod peer_to_peer {
//     tonic::include_proto!("peer_to_peer");
// }

// #[derive(Debug, Default)]
// pub struct MyPeerToPeer {}

// #[tonic::async_trait]
// impl PeerToPeer for MyPeerToPeer {
//     async fn peer_to_peer(
//         &self,
//         request: Request<PeerToPeerRequest>,
//     ) -> Result<Response<PeerToPeerResponse>, Status> {
//         let request = request.into_inner();
//         println!("Got a request: {:?}", request);

//         // encode the image with requester_user_name and requested_views
//         let new_text = format!(
//             "{} {}",
//             request.requester_user_name, request.requested_views
//         );
//         let mut message = Message::empty();
//         message.add_file_data("view_count.txt", new_text.into_bytes());
//         let mut media = Media::from_file(Path::new(&request.requested_image_name)).unwrap();
//         media.hide_message(&message).unwrap();
//         media
//             .save_as(Path::new(&request.requested_image_name))
//             .unwrap();

//         let file = File::open(request.requested_image_name.clone())?;
//         let encoded_bytes = file_to_bytes(file);

//         let reply = PeerToPeerResponse {
//             image_data: encoded_bytes,
//         };
//         Ok(Response::new(reply))
//     }
// }
