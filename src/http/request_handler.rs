use std::collections::BTreeSet;
use std::sync::Arc;

use anyhow::Result;
use async_graphql::http::GraphiQLSource;
use async_graphql::ServerError;
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use serde::de::DeserializeOwned;

use super::request_context::RequestContext;
use super::ServerContext;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike, GraphQLResponse};
use crate::parser::de::Parser;

fn graphiql() -> Result<Response<Body>> {
  Ok(Response::new(Body::from(
    GraphiQLSource::build()
      .title("Tailcall - GraphQL IDE")
      .endpoint("/graphql")
      .finish(),
  )))
}

fn not_found() -> Result<Response<Body>> {
  Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty())?)
}

fn create_request_context(req: &Request<Body>, server_ctx: &ServerContext) -> RequestContext {
  let upstream = server_ctx.blueprint.upstream.clone();
  let allowed = upstream.get_allowed_headers();
  let headers = create_allowed_headers(req.headers(), &allowed);
  RequestContext::from(server_ctx).req_headers(headers)
}

fn update_cache_control_header(
  response: GraphQLResponse,
  server_ctx: &ServerContext,
  req_ctx: Arc<RequestContext>,
) -> GraphQLResponse {
  if server_ctx.blueprint.server.enable_cache_control_header {
    let ttl = req_ctx.get_min_max_age().unwrap_or(0);
    let cache_public_flag = req_ctx.is_cache_public().unwrap_or(true);
    return response.set_cache_control(ttl, cache_public_flag);
  }
  response
}

pub fn update_response_headers(resp: &mut hyper::Response<hyper::Body>, server_ctx: &ServerContext) {
  if !server_ctx.blueprint.server.response_headers.is_empty() {
    resp
      .headers_mut()
      .extend(server_ctx.blueprint.server.response_headers.clone());
  }
}
pub async fn graphql_request<T: DeserializeOwned + GraphQLRequestLike>(
  req: Request<Body>,
  server_ctx: &ServerContext,
) -> Result<Response<Body>> {
  let req_ctx = Arc::new(create_request_context(&req, server_ctx));
  let bytes = hyper::body::to_bytes(req.into_body()).await?;
  let request = serde_json::from_slice::<T>(&bytes);
  match request {
    Ok(request) => {
      let response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;
      let mut response = update_cache_control_header(response, server_ctx, req_ctx).to_response()?;
      update_response_headers(&mut response, server_ctx);
      Ok(response)
    }
    Err(err) => {
      log::error!(
        "Failed to parse request: {}",
        String::from_utf8(bytes.to_vec()).unwrap()
      );

      let mut response = async_graphql::Response::default();
      let server_error = ServerError::new(format!("Unexpected GraphQL Request: {}", err), None);
      response.errors = vec![server_error];

      Ok(GraphQLResponse::from(response).to_response()?)
    }
  }
}
async fn url_req<T: DeserializeOwned + GraphQLRequestLike>(
  req: Request<Body>,
  server_ctx: &ServerContext,
) -> Result<Response<Body>> {
  let req_ctx = Arc::new(create_request_context(&req, server_ctx));
  if let Some(query) = req.uri().path_and_query() {
    let query = query.as_str();
    let mut parser = Parser::from_path(query)?;
    let request = parser.parse::<T>();
    match request {
      Ok(request) => {
        let response = request.data(req_ctx.clone()).execute(&server_ctx.schema).await;
        let mut response = update_cache_control_header(response, server_ctx, req_ctx).to_response()?;
        update_response_headers(&mut response, server_ctx);
        Ok(response)
      }
      Err(err) => {
        log::error!("Failed to parse request: {query}",);

        let mut response = async_graphql::Response::default();
        let server_error = ServerError::new(format!("Unexpected GraphQL Request: {}", err), None);
        response.errors = vec![server_error];

        Ok(GraphQLResponse::from(response).to_response()?)
      }
    }
  } else {
    log::error!("Failed to parse request, invalid url",);
    let mut response = async_graphql::Response::default();
    let server_error = ServerError::new("Unexpected GraphQL Request.".to_string(), None);
    response.errors = vec![server_error];

    Ok(GraphQLResponse::from(response).to_response()?)
  }
}

async fn custom_url_req(req: Request<Body>, server_ctx: &ServerContext) -> Result<Response<Body>> {
  url_req::<GraphQLRequest>(req, server_ctx).await
}
async fn graphql_single_request(req: Request<Body>, server_ctx: &ServerContext) -> Result<Response<Body>> {
  graphql_request::<GraphQLRequest>(req, server_ctx).await
}

async fn graphql_batch_request(req: Request<Body>, server_ctx: &ServerContext) -> Result<Response<Body>> {
  graphql_request::<GraphQLBatchRequest>(req, server_ctx).await
}

fn create_allowed_headers(headers: &HeaderMap, allowed: &BTreeSet<String>) -> HeaderMap {
  let mut new_headers = HeaderMap::new();
  for (k, v) in headers.iter() {
    if allowed.contains(k.as_str()) {
      new_headers.insert(k, v.clone());
    }
  }

  new_headers
}

pub async fn handle_single_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
  match *req.method() {
    hyper::Method::POST if req.uri().path() == "/graphql" => graphql_single_request(req, state.as_ref()).await,
    hyper::Method::GET if req.uri().path().starts_with("/api") => custom_url_req(req, state.as_ref()).await,
    hyper::Method::GET if state.blueprint.server.enable_graphiql => graphiql(),
    _ => not_found(),
  }
}

pub async fn handle_batch_request(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>> {
  match *req.method() {
    hyper::Method::POST if req.uri().path() == "/graphql" => graphql_batch_request(req, state.as_ref()).await,
    hyper::Method::GET if req.uri().path().starts_with("/api") => custom_url_req(req, state.as_ref()).await,
    hyper::Method::GET if state.blueprint.server.enable_graphiql => graphiql(),
    _ => not_found(),
  }
}
