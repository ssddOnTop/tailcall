mod request;
mod response;

use std::sync::{Arc, RwLock};

use cached::proc_macro::cached;
use cached::TimedSizedCache;
use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::config::Config;
use tailcall::http::{handle_request, DefaultHttpClient, ServerContext};
use worker::*;

lazy_static! {
  static ref SERV_CTX: RwLock<Option<Arc<ServerContext>>> = RwLock::new(None);
}

async fn make_req() -> Result<Config> {
  let reader = ConfigReader::init(
    [
      "https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql", // add/edit the SDL links to this list
    ]
    .iter(),
  );
  reader.read().await.map_err(conv_err)
}

#[event(fetch)]
async fn main(wrk_req: Request, _: Env, _: Context) -> Result<Response> {
  let server_ctx = get_option().await;
  if server_ctx.is_none() {
    let cfg = make_req().await.map_err(conv_err)?;
    let blueprint = Blueprint::try_from(&cfg).map_err(conv_err)?;
    let http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
    let serv_ctx = Arc::new(ServerContext::new(blueprint, http_client));
    *SERV_CTX.write().unwrap() = Some(serv_ctx.clone());
  }
  let resp = mkreq(wrk_req).await?;
  Ok(resp)
}

async fn mkreq(wrk_req: Request) -> Result<Response> {
  let response = make_internal(request::Request::wrk_to_req(wrk_req).await?).await;
  response.response_to_wrk()
}
#[cached(
  type = "TimedSizedCache<request::Request, response::Response>",
  create = "{ TimedSizedCache::with_size_and_lifespan(1000,10) }"
)]
async fn make_internal(wrk_req: request::Request) -> response::Response {
  let req = request::Request::req_to_hyper(wrk_req).await.unwrap();
  let hyper_resp = handle_request::<GraphQLRequest>(req, get_option().await.unwrap().clone())
    .await
    .unwrap();

  response::Response::hyper_to_response(hyper_resp).await.unwrap()
}

async fn get_option() -> Option<Arc<ServerContext>> {
  SERV_CTX.read().unwrap().clone()
}
fn conv_err<T: std::fmt::Display>(e: T) -> Error {
  Error::from(format!("{}", e))
}
