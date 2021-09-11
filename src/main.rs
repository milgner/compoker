use std::time::{Duration, Instant};
use std::collections::HashMap;
use actix::prelude::*;
use actix_files::{Files, NamedFile};
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web_actors::ws;
use serde::{ Serialize, Deserialize };

mod poker_server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize, Deserialize, Debug)]
enum Vote {
    Unknown,
    One,
    Two,
    Three,
    Five,
    Eight,
    Thirteen,
    TwentyOne,
    Infinite
}



#[derive(Debug, Serialize, Deserialize)]
enum VotingState {
    Opening,
    Voting,
    Closing
}

struct VotingParticipant {
    id: String,
    name: String,
    // TODO: add reference to session or so...
}

#[derive(Serialize, Deserialize, Debug)]
struct VotingIssue {
    id: String,
    state: VotingState,
    trello_card: Option<String>, // further abstraction TBD
    outcome: Option<Vote>,
    // participant id to votes
    votes: HashMap<String, Vote>,
}

struct VotingSession {
    id: String,
    participants: Vec<VotingParticipant>,
    current_issue: VotingIssue,
}

struct ClientConnection {
    hb: std::time::Instant,
    server: Addr<poker_server::Server>,
    participant: Option<VotingParticipant>,
}

impl ClientConnection {
    pub fn new(server: Addr<poker_server::Server>) -> ClientConnection {
        ClientConnection {
            hb: Instant::now(),
            participant: None,
            server,
        }
    }
}

// fn serialize_message(message: SocketMessage) -> Option<String> {
//     let serialized = serde_json::to_string(&message);
//     if serialized.is_ok() {
//         Option::Some(serialized.unwrap())
//     } else {
//         println!("Failed to serialize message {:?}", message);
//         Option::None
//     }
// }

impl Actor for ClientConnection {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
    }
}

impl ClientConnection {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn start_heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }

    fn process_message(&self, message: poker_server::PokerMessage) {
        println!("deserialized message = {:?}", message);
    }
}


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClientConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let deserialized = serde_json::from_str(&text);
                match deserialized {
                    Ok(message) => {
                        self.process_message(message)
                    }
                    Err(e) => {
                        println!("failed to deserialize: {}, {}", text, e);
                    }
                }
            },
            Ok(ws::Message::Binary(_bin)) => {
                println!("Unexpected binary message received. What's going on?!")
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

struct SessionCoordinator {
    sessions: HashMap<String, VotingSession>,
}

async fn websocket(req: HttpRequest, stream: web::Payload, srv: web::Data<Addr<poker_server::Server>>) -> Result<HttpResponse, Error> {
    let connection = ClientConnection::new(srv.get_ref().clone());
    ws::start(connection, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let poker_server = poker_server::Server::new().start();

    let http_server = HttpServer::new(move || {
        App::new()
            .data(poker_server.clone())
            .route("/ws", web::get().to(websocket))
            .service(Files::new("/", "./public")
                .prefer_utf8(true)
                .index_file("index.html")
                // for SPA behaviour: unknown/dynamic paths will be resolved through app routing mechanism
                .default_handler(|req: ServiceRequest| {
                    let (http_req, _payload) = req.into_parts();

                    async {
                        let response = NamedFile::open("./public/index.html")?.into_response(&http_req)?;
                        Ok(ServiceResponse::new(http_req, response))
                    }
                }))
    })
        .bind("127.0.0.1:8080")?
        .run();
    println!("Server now running at 127.0.0.1:8080");
    http_server.await
}
