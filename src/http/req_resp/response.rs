use std::str::FromStr;

use anyhow::Result;
use derive_setters::Setters;
use http::{HeaderMap, StatusCode};
use http_cache_semantics::ResponseLike;

use crate::http::req_resp::error::map_anyh_err;

#[derive(Clone, Debug, Default, Setters)]
pub struct Response {
  pub status: StatusCode,
  pub headers: HeaderMap,
  pub body: async_graphql::Value,
}

impl Response {
  pub async fn from_response(resp: reqwest::Response) -> Result<Self> {
    let status = StatusCode::from_u16(resp.status().as_u16()).map_err(map_anyh_err)?;
    let mut headers = HeaderMap::new();
    for (k, v) in resp.headers() {
      headers.insert(
        http::HeaderName::from_str(k.as_str()).map_err(map_anyh_err)?,
        http::HeaderValue::from_str(v.to_str().map_err(map_anyh_err)?).map_err(map_anyh_err)?,
      );
    }
    let body = resp.bytes().await?;
    let json = serde_json::from_slice(&body)?;
    Ok(Response { status, headers, body: json })
  }
}

impl ResponseLike for Response {
  fn status(&self) -> StatusCode {
    self.status
  }

  fn headers(&self) -> &HeaderMap {
    &self.headers
  }
}
