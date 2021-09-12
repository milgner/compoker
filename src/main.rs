use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_files::{Files, NamedFile};
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web_actors::ws;

use crate::poker_server::*;

mod poker_server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


struct ClientConnection {
    hb: std::time::Instant,
    server: Addr<Server>,
    participant_id: u32,
    session_id: u32, // ensure that the client will only ever be in one session - keinen Quatsch machen!
}

impl ClientConnection {
    pub fn new(server: Addr<Server>) -> ClientConnection {
        ClientConnection {
            hb: Instant::now(),
            participant_id: 0,
            session_id: 0,
            server,
        }
    }
}

impl Actor for ClientConnection {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
        let addr = ctx.address();
        self.server.send(Connect { addr: addr.recipient() })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.participant_id = res,
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if self.participant_id > 0 {
            self.server.do_send(Disconnect {
                participant_id: self.participant_id,
                session_id: self.session_id });
        } else {
            println!("Something is fishy: stopping before participant_id was set");
        }
        Running::Stop
    }
}

impl ClientConnection {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                // TODO: check whether this triggers the stopping() method and sends Disconnect
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }

    // invoked when a message from the browser has been received
    fn process_message(&self, message: PokerMessage) {
        let message = match message {
            PokerMessage::CreateSessionRequest { participant_name, .. } =>
                PokerMessage::CreateSessionRequest { participant_id: self.participant_id, participant_name },
            PokerMessage::VoteRequest { issue_id, vote, .. } =>
                PokerMessage::VoteRequest { participant_id: self.participant_id, issue_id, vote },
            PokerMessage::JoinSessionRequest { session_id, participant_name, .. } =>
                PokerMessage::JoinSessionRequest { participant_id: self.participant_id, session_id, participant_name },
            _ => message
        };
        self.server.do_send(message);
    }
}

// invoked when the server sends back a message -> forward it through the socket
impl Handler<PokerMessage> for ClientConnection {
    type Result = ();

    fn handle(&mut self, msg: PokerMessage, ctx: &mut Self::Context) {
        match msg {
            // if the server sends back a session id, jot it down so we can use it for Disconnect
            PokerMessage::SessionInfoResponse { session_id, .. } => {
                self.session_id = session_id;
            },
            _ => ()
        }
        let serialized = serde_json::to_string(&msg).unwrap_or("Shit!".to_string());
        ctx.text(serialized);
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

async fn websocket(req: HttpRequest, stream: web::Payload, srv: web::Data<Addr<Server>>) -> Result<HttpResponse, Error> {
    let connection = ClientConnection::new(srv.get_ref().clone());
    ws::start(connection, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let poker_server = Server::new().start();

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
