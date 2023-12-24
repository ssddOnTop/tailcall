use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::RwLock;

use http::HeaderMap;
use http_cache_semantics::CachePolicy;
use lru::LruCache;

use crate::http::{Request, Response};

#[derive(Clone, Debug)]
pub struct CacheEntry {
  response: Response,
  // policy: CachePolicy,
  cached_at: u64,
}
pub struct Cache {
  storage: RwLock<LruCache<Request, CacheEntry>>,
}

impl Cache {
  pub fn new() -> Cache {
    Cache { storage: RwLock::new(LruCache::new(NonZeroUsize::new(100).unwrap())) }
  }
  pub fn insert(&self, request: Request, response: Response) {
    // ttl remaining
    /*let (storable, policy) = self.is_cachable(&request, &response);
    if storable {

    }*/
    let entry = CacheEntry { response, cached_at: 0 };
    let mut storage = self.storage.write().unwrap();
    storage.put(request, entry);
  }
  pub fn get(&self, request: &Request) -> Option<Response> {
    let mut storage = self.storage.write().unwrap();
    return storage.get(request).map(|entry| entry.response.clone());
  }
  fn is_cachable(&self, request: &Request, response: &Response) -> (bool, CachePolicy) {
    let policy = CachePolicy::new(request, response);
    (policy.is_storable(), policy)
  }
}
impl From<&Response> for HeaderMap {
  fn from(response: &Response) -> Self {
    response.headers.clone()
  }
}

impl From<&Request> for HeaderMap {
  fn from(request: &Request) -> Self {
    request
      .headers
      .iter()
      .map(|(k, v)| (k.as_str().parse().unwrap(), v.to_str().unwrap().parse().unwrap()))
      .collect()
  }
}
