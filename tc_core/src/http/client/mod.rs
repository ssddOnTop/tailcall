pub mod client;
#[cfg(not(target_arch = "wasm32"))]
mod client_cli;
#[cfg(target_arch = "wasm32")]
mod client_wasm;

pub use client::*;
