use hello::hello_client::HelloClient;
use hello::HelloRequest;

pub mod hello {
  tonic::include_proto!("hello");
}

pub async fn connect() -> HelloClient<tonic::transport::Channel> {
  HelloClient::connect("http://[::1]:50051").await.unwrap()
}

// send hello request to server, and return the response as string
pub async fn send_hello(client: &mut HelloClient<tonic::transport::Channel>, name: &str) -> String {
  let request = tonic::Request::new(HelloRequest {
    name: name.into(),
  });

  let response = client.send_hello(request).await.unwrap();

  response.into_inner().message
}