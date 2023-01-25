# RPC for Rust

This is very much a work in progress, but the general idea is that you could define some RPC functions like this:

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct Add(pub i32, pub i32);

#[async_trait]
impl RemoteFn for Add {
    type ResultType = i32;

    async fn run(&self) -> Self::ResultType {
        self.0 + self.1
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TryMultiply(pub i32, pub i32);

#[async_trait]
impl RemoteFn for TryMultiply {
    type ResultType = Result<i32, ()>;

    async fn run(&self) -> Self::ResultType {
        Ok(self.0 * self.1)
    }
}
```

Then set up a server, using one of various implementations. For exampe, using `axum`:

```rust
let app = Router::new()
    .route("/api/add", handle_rpc::<Add>())
    .route("/api/multiply", handle_rpc::<TryMultiply>())
    .layer(CorsLayer::permissive());

Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), PORT))
    .serve(app.into_make_service())
    .await
    .unwrap();
```

And then call them with various client implementations, for example, using `reqwasm`:

```rust
let mut connection = http::Connection::new(&format!("http://127.0.0.1:9090/api/add"));
let result = connection.call(&Add(1, 2)).await?;

assert_eq!(3, result);
```