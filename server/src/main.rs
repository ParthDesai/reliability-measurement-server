#[macro_use]
extern crate log;

#[macro_use]
mod utils;

mod measurements;
mod types;

use http::HeaderValue;
use types::Storage;
use uuid::Uuid;
use warp::reply::Reply;
use warp::ws::WebSocket;
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let storage: Storage = Default::default();

    let state = warp::any().map(move || storage.clone());

    let routes = warp::path("ws")
        .and(warp::ws())
        .and(state)
        .map(|ws: warp::ws::Ws, storage| {
            let mut response = ws
                .on_upgrade(move |socket| handle_connection(socket, storage))
                .into_response();
            response
                .headers_mut()
                .insert("access-control-allow-origin", HeaderValue::from_static("*"));
            response
        });

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

async fn handle_connection(ws: WebSocket, storage: Storage) {
    let client_id = Uuid::new_v4().as_u128();
    if let Err(e) = measurements::perform_all(ws, storage, client_id).await {
        error!("Error during measurements client[{}]: {:?}", client_id, e);
    }
}
