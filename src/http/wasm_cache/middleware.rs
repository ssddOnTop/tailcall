use anyhow::Result;
use lazy_static::lazy_static;
use reqwest_middleware::ClientWithMiddleware;

use crate::http::req_resp::error::map_anyh_err;
use crate::http::req_resp::request::Request;
use crate::http::req_resp::response::Response;
use crate::http::wasm_cache::cache::Cache;

pub struct WasmMiddleware;

lazy_static! {
  static ref CACHE: Cache = Cache::new();
}

impl WasmMiddleware {
  pub async fn execute(client: ClientWithMiddleware, req: reqwest::Request) -> Result<Response> {
    let cust_req = Request::from_request(&req).await.map_err(map_anyh_err)?;
    let resp = CACHE.get(&cust_req);
    if let Some(resp) = resp {
      return Ok(resp.clone());
    }
    let resp = client.execute(req).await.map_err(map_anyh_err)?;
    let resp = Response::from_response(resp).await?;
    CACHE.insert(cust_req, resp.clone());
    Ok(resp)
  }
}
