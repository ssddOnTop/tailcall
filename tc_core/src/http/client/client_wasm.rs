use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::client::HttpClient;
use crate::grpc::protobuf::ProtobufOperation;
use crate::http::Response;

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, req: reqwest::Request, operation: Option<ProtobufOperation>) -> anyhow::Result<Response> {
    async_std::task::spawn_local(execute(self.client.clone(), req, operation)).await
  }
}

#[derive(Clone)]
pub struct DefaultHttpClient {
  client: ClientWithMiddleware,
}

#[derive(Default)]
pub struct HttpClientOptions {
  pub http2_only: bool,
}

impl DefaultHttpClient {
  pub fn new() -> Self {
    Self::with_options()
  }

  pub fn with_options() -> Self {
    let builder = Client::builder();
    let client = ClientBuilder::new(builder.build().expect("Failed to build client"));
    DefaultHttpClient { client: client.build() }
  }
}

async fn execute(client: ClientWithMiddleware, request: reqwest::Request, operation: Option<ProtobufOperation>) -> anyhow::Result<Response> {
  let response = client.execute(request).await?;
  let response = Response::from_response(response, operation).await?;
  Ok(response)
}
