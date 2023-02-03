//! # Arpy
//!
//! Define RPC call signatures for use with Arpy providers. See the `examples`
//! folder in this repo for various client/server provider examples.
use std::{error::Error, fmt::Debug, str::FromStr};

/// Derive an [`id::RpcId`].
///
/// It will use the kebab cased type name without any generics or module path.
pub use arpy_macros::RpcId;
use async_trait::async_trait;
use futures::Stream;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// A remote procedure.
///
/// This defines the signature of an RPC call, which can then be used by the
/// client or the server.
pub trait FnRemote: id::RpcId + Serialize + DeserializeOwned + Debug {
    /// The return type.
    type Output: Serialize + DeserializeOwned + Debug;
}

/// Allow an `FnRemote` to be called like a method.
///
/// A blanket implementation is provided for any `T: FnRemote`.
#[async_trait(?Send)]
pub trait FnClient: FnRemote {
    /// The default implementation defers to [`RpcClient::call`].
    ///
    /// You shouldn't need to implement this.
    async fn call<C>(self, connection: &C) -> Result<Self::Output, C::Error>
    where
        C: RpcClient,
    {
        connection.call(self).await
    }
}

impl<T: FnRemote> FnClient for T {}

/// Allow a fallible `FnRemote` to be called like a method.
///
/// A blanket implementation is provided for any `T: FnRemote`.
#[async_trait(?Send)]
pub trait FnTryClient<Success, Error>: FnRemote<Output = Result<Success, Error>> {
    /// The default implementation defers to [`RpcClient::try_call`].
    ///
    /// You shouldn't need to implement this.
    async fn try_call<C>(self, connection: &mut C) -> Result<Success, ErrorFrom<C::Error, Error>>
    where
        C: RpcClient,
    {
        connection.try_call(self).await
    }
}

impl<Success, Error, T> FnTryClient<Success, Error> for T where
    T: FnRemote<Output = Result<Success, Error>>
{
}

/// An RPC client.
///
/// Implement this to provide an RPC client. It uses [`async_trait`] to provide
/// `async` methods. See the `arpy_reqwest` crate for an example.
///
/// [`async_trait`]: async_trait::async_trait
#[async_trait(?Send)]
pub trait RpcClient {
    /// A transport error
    type Error: Error + Debug + Send + Sync + 'static;

    /// Make an RPC call.
    async fn call<F>(&self, function: F) -> Result<F::Output, Self::Error>
    where
        F: FnRemote;

    /// Make a fallible RPC call.
    ///
    /// You shouldn't need to implement this. It just flattens any errors sent
    /// from the server into an [`ErrorFrom`].
    async fn try_call<F, Success, Error>(
        &mut self,
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

pub trait Subscription: id::RpcId + Serialize + DeserializeOwned + Debug {
    type Output: Serialize + DeserializeOwned + Debug;
}

#[async_trait(?Send)]
pub trait SubscriptionClient {
    /// A transport error
    type Error: Error + Debug + Send + Sync + 'static;
    type Output<Item: DeserializeOwned>: Stream<Item = Result<Item, Self::Error>>;

    async fn subscribe<T>(&self, event_type: &str) -> Result<Self::Output<T>, Self::Error>
    where
        T: DeserializeOwned;
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

/// Uniquely identify an RPC call.
pub mod id {
    /// This should be `derive`d with [`crate::RpcId`].
    pub trait RpcId {
        /// `ID` should be a short identifier to uniquely identify an RPC call
        /// on a server.
        const ID: &'static str;
    }
}

/// Some common mime types supported by Arpy providers.
#[derive(Copy, Clone)]
pub enum MimeType {
    Cbor,
    Json,
    XwwwFormUrlencoded,
}

impl MimeType {
    /// The mime type, for example `"application/cbor"`.
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
