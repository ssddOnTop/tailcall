use std::hash::{Hash, Hasher};
use std::str::FromStr;

use anyhow::Result;
use derive_setters::Setters;
use http_cache_semantics::RequestLike;
use hyper::body::Bytes;

use crate::http::req_resp::error::map_anyh_err;

#[derive(Clone, Debug, Default, Setters, PartialEq, Eq)]
pub struct Request {
  pub method: http::method::Method,
  pub url: http::uri::Uri,
  pub headers: http::header::HeaderMap,
  pub body: Vec<u8>,
}

impl Hash for Request {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.method.hash(state);
    self.url.hash(state);
    format!("{:?}", self.headers).hash(state);
    self.body.hash(state);
  }
}

impl Request {
  pub async fn from_request(req: &reqwest::Request) -> Result<Self> {
    let url = req.url().as_str().to_lowercase();
    let url = http::uri::Uri::from_str(&url).map_err(map_anyh_err)?;
    let method = req.method().as_str().to_lowercase();
    let method = http::method::Method::from_str(&method).map_err(map_anyh_err)?;
    // let req_headers = req.headers().into();
    let body = match req.body() {
      Some(b) => b.as_bytes().ok_or(Bytes::new()).map_err(map_anyh_err)?,
      None => &[0; 0],
    }
    .to_vec();
    let mut headers = http::header::HeaderMap::new();
    for (k, v) in req.headers() {
      headers.insert(
        http::HeaderName::from_str(k.as_str()).map_err(map_anyh_err)?,
        http::HeaderValue::from_str(v.to_str().map_err(map_anyh_err)?).map_err(map_anyh_err)?,
      );
    }
    /*
    for (k,v) in req_headers {
        headers.insert(k.to_string().to_lowercase(), Value::String(v.to_str().map_err(map_anyh_err)?.to_lowercase()));
    }*/
    // let headers = Value::Object(headers);
    Ok(Request { method, url, headers, body })
  }
}

impl RequestLike for Request {
  fn uri(&self) -> http::uri::Uri {
    self.url.clone()
  }

  fn is_same_uri(&self, other: &http::uri::Uri) -> bool {
    self.url.eq(other)
  }

  fn method(&self) -> &http::method::Method {
    &self.method
  }

  fn headers(&self) -> &http::header::HeaderMap {
    &self.headers
  }
}
