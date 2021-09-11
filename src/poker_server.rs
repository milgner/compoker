//! The `PokerServer` is an actor that maintains a list of sessions,
//! their participants and current votes

use actix::prelude::*;
use rand::{self, Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::{ Serialize, Deserialize };

use std::collections::{HashMap};

// helper function to generate a random id string
fn generate_random_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

#[derive(Message)]
#[rtype(result = "String")] // return session id
pub struct Connect {
    pub addr: Recipient<PokerMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")] // responses are sent out asynchronously
pub enum PokerMessage {
    CreateSessionRequest {
        participant_name: String
    },
    JoinSessionRequest {
        session_id: String,
        participant_name: String
    },
    SessionInfoResponse {
        session_id: String,
        current_issue: VotingIssue,
        current_participants: Vec<String>
    },
    ParticipantJoinAnnouncement {
        participants: Vec<String>
    },
    ParticipantLeaveAnnouncement {
        participants: Vec<String>
    },
    VotingIssueAnnouncement {
        issue: VotingIssue
    },
    VoteRequest {
        issue_id: String,
        vote: Vote
    },
    VoteReceipt,
    VotingResultsRevelation {
        issue_id: String,
        votes: HashMap<String, Vote>,
        outcome: Vote
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
enum VotingState {
    Opening,
    Voting,
    Closing
}

struct VotingParticipant {
    id: String,
    name: String,
}

impl Clone for VotingParticipant {
    fn clone(&self) -> Self {
        VotingParticipant {
            id: self.id.clone(),
            name: self.name.clone(),
        }
    }
}

impl VotingParticipant {
    pub fn new(name: String) -> VotingParticipant {
        VotingParticipant {
            id: generate_random_id(),
            name
        }
    }
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

impl Clone for VotingIssue {
    fn clone(&self) -> Self {
        VotingIssue {
            id: self.id.clone(),
            state: self.state.clone(),
            outcome: self.outcome.clone(),
            votes: self.votes.clone(),
            trello_card: self.trello_card.clone(),
        }
    }
}

impl VotingIssue {
    pub fn new() -> VotingIssue {
        VotingIssue  {
            id: generate_random_id(),
            votes: HashMap::new(),
            outcome: None,
            state: VotingState::Opening,
            trello_card: None
        }
    }
}

struct VotingSession {
    id: String,
    participants: Vec<VotingParticipant>,
    current_issue: VotingIssue,
}

impl VotingSession {
    pub fn new(session_id: String, initiator_name: String) -> VotingSession {
        VotingSession  {
            id: session_id,
            participants: vec!(VotingParticipant::new(initiator_name)),
            current_issue: VotingIssue::new()
        }
    }
}

pub struct Server {
    sessions: HashMap<String, VotingSession>,
    clients: HashMap<String, Recipient<PokerMessage>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            sessions: HashMap::new(),
            clients: HashMap::new(),
        }
    }

    fn create_session(&mut self, initiator_name: String) -> String {
        let session_id = generate_random_id();
        let session = VotingSession::new(session_id.clone(), initiator_name);
        self.sessions.insert(session_id.clone(), session);
        session_id
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

impl Handler<Connect> for Server {
    type Result = String;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> String {
        let client_id = generate_random_id();
        self.clients.insert(client_id.clone(), msg.addr);
        client_id
    }
}

impl Handler<Disconnect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        self.clients.remove(&msg.id);
    }
}

impl Handler<PokerMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: PokerMessage, ctx: &mut Context<Self>) {
        match msg {
            PokerMessage::CreateSessionRequest { participant_name } => {
                let session_id = self.create_session(participant_name);
                let session = self.sessions.get(session_id.as_str()).unwrap();
                let current_participants = session.participants.iter().map( |participant| -> String {
                    participant.name.clone()
                }).collect();
                ctx.notify(PokerMessage::SessionInfoResponse {
                    session_id: session.id.clone(),
                    current_issue: session.current_issue.clone(),
                    current_participants
                })
            }
            _ => {
                println!("Message not handled: {:?}", msg)
            }
        }
    }
}
