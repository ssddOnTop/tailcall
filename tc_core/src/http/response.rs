use anyhow::Result;
use derive_setters::Setters;

use crate::grpc::protobuf::ProtobufOperation;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response {
  pub status: reqwest::StatusCode,
  pub headers: reqwest::header::HeaderMap,
  pub body: async_graphql::Value,
}

impl Response {
  pub async fn from_response(resp: reqwest::Response, operation: Option<ProtobufOperation>) -> Result<Self> {
    let status = resp.status();
    let headers = resp.headers().to_owned();
    let body = resp.text().await?;
    let body = match operation {
      Some(operation) => operation.convert_output(&string_to_bytes(&body))?,
      None => serde_json::from_slice::<async_graphql::Value>(body.as_bytes())?,
    };
    Ok(Response { status, headers, body })
  }
}

fn string_to_bytes(input: &str) -> Vec<u8> {
  let mut bytes = Vec::new();
  let mut chars = input.chars().peekable();

  while let Some(c) = chars.next() {
    match c {
      '\\' => match chars.next() {
        Some('0') => bytes.push(0),
        Some('n') => bytes.push(b'\n'),
        Some('t') => bytes.push(b'\t'),
        Some('r') => bytes.push(b'\r'),
        Some('\\') => bytes.push(b'\\'),
        Some('\"') => bytes.push(b'\"'),
        Some('x') => {
          let mut hex = chars.next().unwrap().to_string();
          hex.push(chars.next().unwrap());
          let byte = u8::from_str_radix(&hex, 16).unwrap();
          bytes.push(byte);
        }
        _ => std::panic!("Unsupported escape sequence"),
      },
      _ => bytes.push(c as u8),
    }
  }

  bytes
}
