//@ # A complexity poker server in Rust
//@
//@ This is another learning project for experimentation which, through a series
//@ of happpy little accidents happens to result in usable software.
//@
//@ It's written in the [Rust programming language](https://www.rust-lang.org/)
//@ using [Literate Programming](https://en.wikipedia.org/wiki/Literate_programming)
//@ style.
//@
//@ We're going to use two main libraries:
//@
//@ [Riker](https://riker.rs/) for an actor-based approach
use riker::actors::*;

//@ and [warp](https://github.com/seanmonstar/warp) for the webserver part.
//@ And yes, it looks like there are a lot of Star Trek fans in the Rust
//@ community. 🤓

extern crate warp;
use crate::warp::Filter;

//@ Let's get some basics out of the way...
//@
//@ ## Logging
//@ We're also going to need to do some logging. Fern is an established logging 
//@ library - let's use that.

extern crate fern;
#[macro_use]
extern crate log;

fn init_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

//@ ## The actor hierarchy
//@
//@ At the top of our application-specific actor hierarchy we have the poker
//@ server, which is responsible for coordinating the creation of sessions.
//@ Below the sessions are the actors for every participant.
//@
//@ Since the individual actors must be free to move between threads and
//@ support re-instantiation through the system, the web server and the
//@ websocket connections will have to live outside of the actor hierarchy.
//@
//@ For now. I plan to revisit this part once I have further familiarized myself
//@ with the intricacies of the language. (I'm sure there's a way 😉)
//@
//@ ```
//@ compoker
//@ └─ user
//@    └─ poker-server
//@       ├─ session 23482384
//@       │  ├─ participant 1
//@       │  ├─ participant 2
//@       │  └─ ...
//@       └─ session 93483432
//@          └─ ...
//@ ```
//@
//@ ## Actor: The Poker Server


struct PokerServer {}

impl Default for PokerServer {
    fn default() -> Self {
        PokerServer{}
    }
}

impl Actor for PokerServer {
    type Msg = serde_json::Value;

    fn recv(&mut self,
            ctx: &Context<Self::Msg>,
            msg: Self::Msg,
            sender: Sender) {
    }
}

//@ ## The web server
//@
//@
//@ ### Accepting websockets
//@
//@ Whenever a client connects to a websocket, we'll have to accept that
//@ connection and wire it up with the poker server so that they can send messages
//@ back and forth. When the connection is established, it will have to talk to
//@ the poker server itself. Afterwards it will have to talk to the session and
//@ finally to the actor that represents the participant in that session.
//@
//@ For this scenario to work, we'll just have to keep an `ActorRef` that can
//@ receive the JSON messages our clients send. Every message we send can be
//@ replied to with a new `ActorRef` which will then replace the destination of
//@ the next message. 

struct ClientConnection {
    websocket: warp::ws::WebSocket,
    actor: ActorRef<serde_json::Value>,
}

impl ClientConnection {
    fn start(ws: warp::ws::WebSocket, poker_server: ActorRef<serde_json::Value>) {
        let mut instance = ClientConnection {
            websocket: ws,
            actor: poker_server,
        };
        instance.run();
    }
    
    fn run(&mut self) {
        println!("Established client connection!");
    }
}

//@ The function which accepts the client connection must be able to instantiate
//@ the `ClientConnection` struct for which it needs the `ActorRef`. But the warp
//@ code won't pass it in. A higher-order function (a.k.a. currying) helps us
//@ here:

pub fn curry_websocket_handler<F, I, U>(poker_server: ActorRef<serde_json::Value>) -> F 
where
    F: FnOnce(warp::ws::Ws) -> I + Send + 'static,
    I: FnOnce(warp::ws::WebSocket) -> U + Send + 'static,
    U: std::future::Future<Output = ()> + Send + 'static
{
    return move |ws: warp::ws::Ws| {
        Ok(async {
            ws.on_upgrade(move |websocket: warp::ws::WebSocket| {
                ClientConnection::start(websocket, poker_server.clone());
            })
        }) 
    }
}

struct WebServer {
    listen_on: std::net::SocketAddr,
    poker_server: ActorRef<serde_json::Value>
}

use serde::{Serialize, Deserialize};

impl WebServer {
    fn create(listen_on: std::net::SocketAddr, poker_server: ActorRef<serde_json::Value>) -> Self {
        WebServer {
            listen_on,
            poker_server
        }
    }

    async fn start(&mut self) {
        let websocket_handler = curry_websocket_handler(self.poker_server.clone());
        let ws_route = warp::path("ws").and(warp::ws()).and_then(websocket_handler)
            .with(warp::cors().allow_origin("http://localhost"));
        let static_route = warp::path::end().and(warp::fs::dir("public"));
        let routes = ws_route.or(static_route);
        warp::serve(routes).run(self.listen_on).await
    }
}


//@ ## Starting the webserver
//@
//@ Before trying to start the web server, we'll need to determine on what port
//@ and interface to start it. Let's use the environment variables `PORT` and
//@ `LISTEN_ON` for that. `127.0.0.1:8080` seems like a sensible default.
//@ If you're on Docker, you'll want to set `LISTEN_ON` to `0.0.0.0`.

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_INTERFACE: &str = "127.0.0.1";

use std::str::FromStr;

fn listen_port() -> u16 {
    match std::env::var("PORT") {
        Ok(port) => {
            match u16::from_str(port.as_str()) {
                Ok(port) => port,
                Err(_) => {
                    error!("Failed to parse port {}", port);
                    DEFAULT_PORT
                }
            }
        },
        Err(_) => {
            info!("No $PORT environment variable set, falling back to {}", DEFAULT_PORT);
            DEFAULT_PORT
        }
    }
}

fn listen_interface() -> String {
    match std::env::var("LISTEN_ON") {
        Ok(interface) => interface,
        Err(_) => {
            info!("No $LISTEN_ON set, falling back to {}", DEFAULT_INTERFACE);
            DEFAULT_INTERFACE.to_string()
        }
    }
}

fn listen_addr() -> Result<std::net::SocketAddr, std::net::AddrParseError> {
    format!("{}:{}", listen_interface(), listen_port()).parse()
}

//@ As warp uses [Tokio](https://tokio.rs/) for its concurrent functionality,
//@ we'll annotate  our `main` function accordingly:

#[tokio::main]
async fn main() {
    init_logging().expect("Failed to initialize logging");
    let listen_on = listen_addr().expect("Cannot parse interface & port configuration");

    let sys = SystemBuilder::new()
        .name("compoker")
        .create()
        .unwrap();

    let poker_server = sys.actor_of::<PokerServer>("poker-server").expect("Failed to start poker server");
    let mut web_server = WebServer::create(listen_on, poker_server);
    web_server.start().await;
}
