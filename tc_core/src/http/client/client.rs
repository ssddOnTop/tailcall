#[cfg(not(target_arch = "wasm32"))]
pub use super::client_cli::*;
#[cfg(target_arch = "wasm32")]
pub use super::client_wasm::*;
use crate::grpc::protobuf::ProtobufOperation;
use crate::http::Response;

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: reqwest::Request, option: Option<ProtobufOperation>) -> anyhow::Result<Response>;
}
