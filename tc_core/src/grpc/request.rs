use anyhow::Result;
use hyper::{HeaderMap, Method};
use reqwest::Request;
use url::Url;

use super::protobuf::ProtobufOperation;
use crate::http::{HttpClient, Response};

pub fn create_grpc_request(url: Url, headers: HeaderMap, body: Vec<u8>) -> Request {
  let mut req = Request::new(Method::POST, url);
  *req.version_mut() = reqwest::Version::HTTP_2;
  req.headers_mut().extend(headers.clone());
  req.body_mut().replace(body.into());

  req
}

pub async fn execute_grpc_request(
  client: &dyn HttpClient,
  operation: &ProtobufOperation,
  request: Request,
) -> Result<Response> {
  let response = client.execute(request, Some(operation.clone())).await?;
  Ok(response)
  // bail!("Failed to execute request")
}
