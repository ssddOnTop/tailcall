use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use serde_json::{Map, Value};

use crate::conv_err;

#[derive(Clone, Debug, Default, Eq)]
pub struct Request {
  pub method: String,
  pub headers: String,
  pub url: String,
  pub body: String,
}
impl Hash for Request {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.url.hash(state);
    consume_spaces(&self.body).hash(state);
  }
}
impl PartialEq for Request {
  fn eq(&self, other: &Self) -> bool {
    consume_spaces(&self.body).eq(&consume_spaces(&other.body)) && self.url.eq(&other.url)
  }
}

impl Request {
  pub async fn wrk_to_req(mut req: worker::Request) -> worker::Result<Self> {
    let wrk_headers = req.headers().clone();
    let mut headers = Map::new();
    for (k, v) in &wrk_headers {
      headers.insert(k, Value::String(v));
    }
    let headers = Value::Object(headers).to_string();
    let url = req.url()?.as_str().to_string();
    let method = req.method().to_string();
    let body = req.text().await?;
    Ok(Self { method, headers, url, body })
  }

  pub async fn req_to_hyper(self) -> worker::Result<hyper::Request<hyper::Body>> {
    let method = self.method;
    let uri = self.url;
    let headers = serde_json::from_str::<Value>(&self.headers).map_err(conv_err)?;
    let mut builder = hyper::Request::builder().method(convert_method(&method)).uri(uri);
    for (k, v) in headers.as_object().unwrap() {
      builder = builder.header(k, v.as_str().unwrap());
    }
    builder.body(hyper::body::Body::from(self.body)).map_err(conv_err)
  }
}

fn consume_spaces(s: &str) -> String {
  s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn convert_method(method_str: &str) -> hyper::Method {
  match method_str {
    "GET" => Ok(hyper::Method::GET),
    "POST" => Ok(hyper::Method::POST),
    "PUT" => Ok(hyper::Method::PUT),
    "DELETE" => Ok(hyper::Method::DELETE),
    "HEAD" => Ok(hyper::Method::HEAD),
    "OPTIONS" => Ok(hyper::Method::OPTIONS),
    "PATCH" => Ok(hyper::Method::PATCH),
    "CONNECT" => Ok(hyper::Method::CONNECT),
    "TRACE" => Ok(hyper::Method::TRACE),
    _ => Err("Unsupported HTTP method"),
  }
  .unwrap()
}
