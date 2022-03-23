use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::{Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get},
    Router,
};
use axum::body::{Body, BoxBody};
use axum::http::{Request, Uri};
use axum::response::Response;
use tower::ServiceExt;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tower_http::compression::CompressionLayer;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
use tracing_subscriber::filter::LevelFilter;
use tracing_tree::HierarchicalLayer;

// use crate::poker_server::*;
// mod poker_server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_INTERFACE: &str = "127.0.0.1";

fn listen_port() -> u16 {
    match std::env::var("PORT") {
        Ok(port) => match u16::from_str(port.as_str()) {
            Ok(port) => port,
            Err(_) => {
                println!("Failed to parse port {}", port);
                DEFAULT_PORT
            }
        },
        Err(_) => {
            println!("No $PORT environment variable set");
            DEFAULT_PORT
        }
    }
}

fn listen_interface() -> String {
    match std::env::var("LISTEN_INTERFACE") {
        Ok(interface) => interface,
        Err(_) => {
            println!(
                "No $LISTEN_INTERFACE set, falling back to {}",
                DEFAULT_INTERFACE
            );
            DEFAULT_INTERFACE.to_string()
        }
    }
}

fn listen_addr() -> Result<SocketAddr, std::net::AddrParseError> {
    format!("{}:{}", listen_interface(), listen_port()).parse()
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<axum::headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    println!("client send str: {:?}", t);
                }
                Message::Binary(_) => {
                    println!("client send binary data");
                }
                Message::Ping(_) => {
                    println!("socket ping");
                }
                Message::Pong(_) => {
                    println!("socket pong");
                }
                Message::Close(_) => {
                    println!("client disconnected");
                    return;
                }
            }
        } else {
            println!("client disconnected");
            return;
        }
    }

    loop {
        if socket
            .send(Message::Text(String::from("Hi!")))
            .await
            .is_err()
        {
            println!("client disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}


async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let dummy_request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    match ServeDir::new("./public").oneshot(dummy_request).await {
        Ok(response) => Ok(response.map(axum::body::boxed)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Ouch! {}", err)))
    }
}

async fn serve_static_files(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        get_static_file(Uri::from_static("/")).await
    } else {
        Ok(res)
    }
}

async fn run_server() -> Result<(), Box<dyn Error>> {
    // build our application with some routes
    let app = Router::new()
        .fallback(get(serve_static_files))
        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        .route("/ws", get(ws_handler))
        .layer(CompressionLayer::new())
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Registry::default()
        .with(tracing_subscriber::EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into()).with_env_var("POKER_LOG").from_env()?)
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(console_subscriber::spawn())
        .with(tracing_subscriber::fmt::layer().json()).init();

    run_server().await
}
