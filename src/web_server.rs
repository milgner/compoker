use axum::body::{Body, BoxBody};
use axum::extract::ws::WebSocket;
use axum::extract::{ws::Message, TypedHeader, WebSocketUpgrade};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use bastion::prelude::*;
use http::{Request, StatusCode, Uri};
use lazy_static::lazy_static;
use simple_error::SimpleError;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tower::ServiceExt;
use tower_http::compression::CompressionLayer;
use tower_http::sensitive_headers::{
    SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer,
};
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

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
                tracing::error!(
                    "Failed to parse port {}; falling back to {}",
                    port,
                    DEFAULT_PORT
                );
                DEFAULT_PORT
            }
        },
        Err(_) => {
            tracing::warn!(
                "No $PORT environment variable set; falling back to {}",
                DEFAULT_PORT
            );
            DEFAULT_PORT
        }
    }
}

fn listen_interface() -> String {
    match std::env::var("LISTEN_INTERFACE") {
        Ok(interface) => interface,
        Err(_) => {
            tracing::warn!(
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

lazy_static! {
    static ref SENSITIVE_HEADERS: Arc<[http::header::HeaderName]> = Arc::new([
        http::header::AUTHORIZATION,
        http::header::PROXY_AUTHORIZATION,
        http::header::COOKIE,
        http::header::SET_COOKIE,
    ]);
    static ref WEBSOCKET_SUPERVISOR: SupervisorRef =
        Bastion::supervisor(|sp| { sp.with_strategy(SupervisionStrategy::OneForOne) })
            .expect("Couldn't create the web supervisor.");
}

/// handle an incoming websocket request
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<axum::headers::UserAgent>>,
    poker_server: ChildrenRef,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        tracing::info!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(move |socket| handle_socket(socket, poker_server))
}

/// receives messages from the websocket and lets the poker server know about them
fn process_websocket_message(
    msg: Result<Message, axum::Error>,
    poker_server: &ChildrenRef,
) -> Result<(), ()> {
    if let Ok(msg) = msg {
        match msg {
            Message::Text(t) => {
                tracing::debug!("client send str: {:?}", t);
                // POKER_SERVER
                //     .as_ref()
                //     .unwrap()
                //     .broadcast(t)
                //     .map_err(|_| ())?;
            }
            Message::Binary(_) => {
                println!("client send binary data");
            }
            Message::Ping(_) => {
                tracing::debug!("socket ping");
            }
            Message::Pong(_) => {
                tracing::debug!("socket pong");
            }
            Message::Close(_) => {
                tracing::info!("client disconnected");
                return Err(());
            }
        }
    } else {
        tracing::info!("client disconnected");
        return Err(());
    }
    Ok(())
}

/// reacts to messages from the poker server and dispatches them over the websocket
fn process_actor_message(msg: SignedMessage, socket: &WebSocket) -> Result<(), ()> {
    Ok(())
}

/// starts an actor which talks to the given websocket and allows it to communicate with the poker server
async fn handle_socket(socket: WebSocket, poker_server: ChildrenRef) {
    WEBSOCKET_SUPERVISOR
        .children(move |children| {
            let socket = Arc::new(Mutex::new(socket));
            children.with_exec(move |context| {
                let socket = Arc::clone(&socket);
                let poker_server = poker_server.clone();

                async move {
                    let mut locked = socket.lock().await;
                    loop {
                        tokio::select! {
                            Some(msg) = locked.recv() => process_websocket_message(msg, &poker_server)?,
                            msg = context.recv() => process_actor_message(msg?, &locked)?,
                        }
                    }
                }
            })
        })
        .expect("Failed to create actor for websocket");
}

/// simple function to return the file from the `public` folder to the client
async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let dummy_request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    match ServeDir::new("./public").oneshot(dummy_request).await {
        Ok(response) => Ok(response.map(axum::body::boxed)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Ouch! {}", err))),
    }
}

/// serve all files from the `public` folder; if none matches, return `index.html` instead of 404
async fn serve_static_files(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        get_static_file(Uri::from_static("/")).await
    } else {
        Ok(res)
    }
}

pub async fn run(poker_server: ChildrenRef) -> Result<ChildrenRef, Box<dyn Error>> {
    let addr = listen_addr()?;

    Bastion::children(|children| {
        children.with_exec(move |ctx| {
            let poker_server = poker_server.clone();
            let addr = addr.clone();
            async move {
                // build our application with some routes
                let app = Router::new()
                    .fallback(get(serve_static_files))
                    // routes are matched from bottom to top, so we have to put `nest` at the
                    // top since it matches all routes
                    .route(
                        "/ws",
                        get({
                            let poker_server = poker_server.clone();
                            move |upgrade, ua| ws_handler(upgrade, ua, poker_server)
                        }),
                    )
                    .layer(CompressionLayer::new())
                    // logging so we can see whats going on (excluding sensitive headers)
                    .layer(SetSensitiveRequestHeadersLayer::from_shared(Arc::clone(
                        &*SENSITIVE_HEADERS,
                    )))
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(DefaultMakeSpan::default().include_headers(true)),
                    )
                    .layer(SetSensitiveResponseHeadersLayer::from_shared(Arc::clone(
                        &*SENSITIVE_HEADERS,
                    )));

                // run it with hyper
                tracing::info!("listening on {}", addr);
                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await
                    .map_err(|_| ())
            }
        })
    })
    .map_err(|_| SimpleError::new("Failed to create actor for web service").into())
}
