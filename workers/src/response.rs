use serde_json::{Map, Value};

use crate::conv_err;

#[derive(Clone, Debug, Default)]
pub struct Response {
  pub status: u16,
  pub headers: Value,
  pub body: String,
}

impl Response {
  /* pub async fn hyper_to_response(mut resp: worker::Response) -> worker::Result<Self> {
      let status = resp.status();
      let headers = resp.headers().to_owned();
      let body = resp.text().await?;
      Ok(Response { status, headers, body })
  }*/
  pub async fn hyper_to_response(response: hyper::Response<hyper::Body>) -> worker::Result<Self> {
    let status = response.status().as_u16();
    let hyper_headers = response.headers();
    let mut headers = Map::new();
    for (k, v) in hyper_headers {
      headers.insert(
        k.as_str().to_string(),
        Value::String(v.to_str().map_err(conv_err)?.to_string()),
      );
    }
    let headers = Value::Object(headers);
    let buf = hyper::body::to_bytes(response).await.map_err(conv_err)?;
    let body = std::str::from_utf8(&buf).map_err(conv_err)?.to_string();
    Ok(Response { status, headers, body })
  }
  pub fn response_to_wrk(self) -> worker::Result<worker::Response> {
    let body = self.body;
    let headers = self.headers;
    let status = self.status;
    let mut resp = worker::Response::from_bytes(body.as_bytes().to_vec())?.with_status(status);
    let mut header_mp = worker::Headers::new();
    for (k, v) in headers.as_object().unwrap() {
      header_mp.append(k, v.as_str().unwrap())?;
    }
    *resp.headers_mut() = header_mp;
    Ok(resp)
  }
  /*    pub async fn hyper_to_wrk(response: hyper::Response<hyper::Body>) -> worker::Result<worker::Response> {
      let buf = hyper::body::to_bytes(response).await.map_err(conv_err)?;
      let mut headers = worker::Headers::new();
      let text = std::str::from_utf8(&buf).map_err(conv_err)?;
      let mut response = worker::Response::ok(text).map_err(conv_err)?;
      let headers = response.headers();
      let wheaders = response.headers_mut();
      for (k,v) in headers {
          wheaders.append(&k,&v)?;
      }
      Ok(response)
  }*/
}
