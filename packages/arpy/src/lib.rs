use std::{error::Error, fmt::Debug, str::FromStr};

pub use arpy_macros::RpcId;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

pub trait FnRemote: id::RpcId + Serialize + DeserializeOwned + Debug {
    type Output: Serialize + DeserializeOwned + Debug;
}

#[async_trait(?Send)]
pub trait FnClient: FnRemote {
    async fn call<C>(self, connection: &C) -> Result<Self::Output, C::Error>
    where
        C: RpcClient,
    {
        connection.call(self).await
    }
}

impl<T: FnRemote> FnClient for T {}

#[async_trait(?Send)]
pub trait FnTryCient<Success, Error>: FnRemote<Output = Result<Success, Error>> {
    async fn try_call<C>(self, connection: &C) -> Result<Success, ErrorFrom<C::Error, Error>>
    where
        C: RpcClient,
    {
        connection.try_call(self).await
    }
}

impl<Success, Error, T> FnTryCient<Success, Error> for T where
    T: FnRemote<Output = Result<Success, Error>>
{
}

/// An error from a fallible RPC call.
/// 
/// A fallible RPC call is one where `FnRemote::Output = Result<_, _>`.
#[derive(Error, Debug)]
pub enum ErrorFrom<C, S> {
    /// A transport error.
    #[error("Connection: {0}")]
    Connection(C),
    /// An error from `FnRemote::Output`.
    #[error("Server: {0}")]
    Server(S),
}

#[async_trait(?Send)]
pub trait RpcClient {
    type Error: Error + Debug + Send + Sync + 'static;

    async fn call<F>(&self, function: F) -> Result<F::Output, Self::Error>
    where
        F: FnRemote;

    async fn try_call<F, Success, Error>(
        &self,
        function: F,
    ) -> Result<Success, ErrorFrom<Self::Error, Error>>
    where
        Self: Sized,
        F: FnRemote<Output = Result<Success, Error>>,
    {
        match self.call(function).await {
            Ok(Ok(ok)) => Ok(ok),
            Ok(Err(e)) => Err(ErrorFrom::Server(e)),
            Err(e) => Err(ErrorFrom::Connection(e)),
        }
    }
}

pub mod id {
    pub trait RpcId {
        const ID: &'static str;
    }
}

#[derive(Copy, Clone)]
pub enum MimeType {
    Cbor,
    Json,
    XwwwFormUrlencoded,
}

impl MimeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cbor => "application/cbor",
            Self::Json => "application/json",
            Self::XwwwFormUrlencoded => "application/x-www-form-urlencoded",
        }
    }
}

impl FromStr for MimeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(Self::Cbor.as_str()) {
            Ok(Self::Cbor)
        } else if s.starts_with(Self::Json.as_str()) {
            Ok(Self::Json)
        } else if s.starts_with(Self::XwwwFormUrlencoded.as_str()) {
            Ok(Self::XwwwFormUrlencoded)
        } else {
            Err(())
        }
    }
}
