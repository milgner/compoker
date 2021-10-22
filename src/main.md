# A complexity poker server in Rust

This is another learning project for experimentation which, through a series
of happpy little accidents happens to result in usable software.

It's written in the [Rust programming language](https://www.rust-lang.org/)
using [Literate Programming](https://en.wikipedia.org/wiki/Literate_programming)
style.

We're going to use two main libraries:

[Riker](https://riker.rs/) for an actor-based approach
```rust
use riker::actors::*;
```

and [warp](https://github.com/seanmonstar/warp) for the webserver part.
And yes, it looks like there are a lot of Star Trek fans in the Rust
community. 🤓

```rust
extern crate warp;
use crate::warp::Filter;
```

Let's get some basics out of the way...

## Logging
We're also going to need to do some logging. Fern is an established logging 
library - let's use that.

```rust
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
```

## The actor hierarchy

At the top of our application-specific actor hierarchy we have two elements:
the poker server, which is responsible for coordinating the creation of
sessions and the web server which accepts websocket connections and connects
them with the poker server.

The full hierarchy should look something like this:

```
compoker
└─ user
   ├─ poker-server
   │  ├─ session 23482384
   │  │  ├─ participant 1
   │  │  ├─ participant 2
   │  │  └─ ...
   │  └─ session 93483432
   │     └─ ...
   └─ web-server
      ├─ connection 1
      ├─ connection 2
      └─ ...
```

## Actor: The Poker Server


```rust
struct PokerServer {}

impl Default for PokerServer {
    fn default() -> Self {
        PokerServer{}
    }
}

impl Actor for PokerServer {
    type Msg = ();

    fn recv(&mut self,
            ctx: &Context<Self::Msg>,
            msg: Self::Msg,
            sender: Sender) {
    }
}
```

## Actor: Web server

Let's start by implementing an actor for the web server, based on the
aforementioned warp library.

```rust
struct WebServer {
    listen_on: std::net::SocketAddr,
    poker_server: ActorRef<()>
}

use serde::{Serialize, Deserialize};

impl WebServer {
    fn create(listen_on: std::net::SocketAddr, poker_server: ActorRef<()>) -> Self {
        WebServer {
            listen_on,
            poker_server
        }
    }

    async fn start(&mut self) {
        let ws_route = warp::path("ws").and(warp::ws()).and_then(handle_websocket)
            .with(warp::cors().allow_origin("http://localhost"));
        let static_route = warp::path::end().and(warp::fs::dir("public"));
        let routes = ws_route.or(static_route);
        warp::serve(routes).run(self.listen_on).await
    }
}
```


## Accepting websockets

Whenever a client connects to a websocket, we'll have to accept that
connection:


```rust
pub async fn handle_websocket(ws: warp::ws::Ws) -> Result<impl warp::Reply, std::convert::Infallible> {
    Ok(ws.on_upgrade(move |socket| accept_client_connection(socket)))
}

async fn accept_client_connection(ws: warp::ws::WebSocket) {
    println!("Established client connection!");
}
```

## Starting the webserver

Before trying to start the web server, we'll need to determine on what port
and interface to start it. Let's use the environment variables `PORT` and
`LISTEN_ON` for that. `127.0.0.1:8080` seems like a sensible default.
If you're on Docker, you'll want to set `LISTEN_ON` to `0.0.0.0`.

```rust
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
```

As warp uses [Tokio](https://tokio.rs/) for its concurrent functionality,
we'll annotate  our `main` function accordingly:

```rust
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
```
