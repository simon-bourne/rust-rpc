use arpy::{FnRemote, RpcId};
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[derive(RpcId, Serialize, Deserialize, Debug)]
pub struct Add(pub i32, pub i32);

impl FnRemote for Add {
    type Output = i32;
}

#[derive(RpcId, Serialize, Deserialize, Debug)]
pub struct TryMultiply(pub i32, pub i32);

impl FnRemote for TryMultiply {
    type Output = Result<i32, ()>;
}

pub const PORT: u16 = 9090;
