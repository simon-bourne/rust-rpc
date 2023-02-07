//! These tests are run from `provide_server`
use arpy::{ConcurrentRpcClient, FnRemote, FnTryRemote};
use arpy_reqwasm::{http, websocket};
use arpy_test::{Add, Counter, TryMultiply, PORT};
use futures::StreamExt;
use reqwasm::websocket::futures::WebSocket;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn simple_http() {
    let connection = http::Connection::new(&server_url("http", "http"));

    assert_eq!(3, Add(1, 2).call(&connection).await.unwrap());
}

#[wasm_bindgen_test]
async fn simple_websocket() {
    let connection = websocket();

    assert_eq!(3, Add(1, 2).call(&connection).await.unwrap());
    assert_eq!(12, TryMultiply(3, 4).try_call(&connection).await.unwrap());
}

#[wasm_bindgen_test]
async fn out_of_order_websocket() {
    let connection = websocket();

    let result1 = Add(1, 2).begin_call(&connection).await.unwrap();
    let result2 = TryMultiply(3, 4).try_begin_call(&connection).await.unwrap();

    // Await in reverse order
    assert_eq!(12, result2.await.unwrap());
    assert_eq!(3, result1.await.unwrap());
}

#[wasm_bindgen_test]
async fn websocket_subscription() {
    let connection = websocket();

    let stream = connection.subscribe(Counter(5)).await.unwrap();

    assert_eq!(
        stream
            .take(10)
            .map(Result::unwrap)
            .collect::<Vec<i32>>()
            .await,
        (5..15).collect::<Vec<i32>>()
    )
}

fn websocket() -> websocket::Connection {
    websocket::Connection::new(WebSocket::open(&server_url("ws", "ws")).unwrap())
}

fn server_url(scheme: &str, route: &str) -> String {
    let port_str = format!("{PORT}");
    let port = option_env!("TCP_PORT").unwrap_or(&port_str);
    format!("{scheme}://127.0.0.1:{port}/{route}")
}
