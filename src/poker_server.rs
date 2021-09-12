//! The `PokerServer` is an actor that maintains a list of sessions,
//! their participants and current votes

use std::collections::HashMap;

use actix::prelude::*;
use rand::{self, thread_rng, Rng};
use serde::{Deserialize, Serialize};

// helper function to generate a random id string
fn generate_random_id() -> usize {
    thread_rng().gen::<usize>()
}

#[derive(Message)]
#[rtype(result = "usize")] // return participant id
pub struct Connect {
    pub addr: Recipient<PokerMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub participant_id: usize,
    pub session_id: usize,
}

fn zero_usize() -> usize {
    0
}

#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")] // responses are sent out asynchronously
                        // participant ids always use Option<> so that they can be deserialized from JSON
                        // the participant id is then filled in through the `ClientConnection`
pub enum PokerMessage {
    CreateSessionRequest {
        #[serde(default = "zero_usize")]
        participant_id: usize,
        participant_name: String,
    },
    JoinSessionRequest {
        #[serde(default = "zero_usize")]
        participant_id: usize,
        session_id: usize,
        participant_name: String,
    },
    SessionInfoResponse {
        session_id: usize,
        current_issue: VotingIssue,
        current_participants: Vec<String>,
    },
    SessionUnknownResponse {
        session_id: usize,
    },
    ParticipantJoinAnnouncement {
        participant_name: String,
    },
    ParticipantLeaveAnnouncement {
        participant_name: String,
    },
    VotingIssueAnnouncement {
        issue: VotingIssue,
    },
    VoteRequest {
        #[serde(default = "zero_usize")]
        participant_id: usize,
        issue_id: usize,
        vote: Vote,
    },
    VoteReceipt,
    VotingResultsRevelation {
        issue_id: usize,
        votes: HashMap<String, Vote>,
        outcome: Vote,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Vote {
    Unknown,
    One,
    Two,
    Three,
    Five,
    Eight,
    Thirteen,
    TwentyOne,
    Infinite,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VotingState {
    Opening,
    Voting,
    Closing,
}

pub struct VotingParticipant {
    id: usize,
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
    pub fn new(id: usize, name: String) -> VotingParticipant {
        VotingParticipant { id, name }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VotingIssue {
    id: usize,
    state: VotingState,
    trello_card: Option<String>,
    // further abstraction TBD
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
        VotingIssue {
            id: generate_random_id(),
            votes: HashMap::new(),
            outcome: None,
            state: VotingState::Opening,
            trello_card: None,
        }
    }
}

struct VotingSession {
    id: usize,
    participants: Vec<VotingParticipant>,
    current_issue: VotingIssue,
}

impl VotingSession {
    pub fn new(session_id: usize, initiator_id: usize, initiator_name: String) -> VotingSession {
        VotingSession {
            id: session_id,
            participants: vec![VotingParticipant::new(initiator_id, initiator_name)],
            current_issue: VotingIssue::new(),
        }
    }

    pub fn participant_names(&self) -> Vec<String> {
        self.participants.iter().map(|p| p.name.clone()).collect()
    }
}

impl Clone for VotingSession {
    fn clone(&self) -> Self {
        VotingSession {
            id: self.id,
            current_issue: self.current_issue.clone(),
            participants: self.participants.clone(),
        }
    }
}

pub struct Server {
    sessions: HashMap<usize, VotingSession>,
    clients: HashMap<usize, Recipient<PokerMessage>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            sessions: HashMap::new(),
            clients: HashMap::new(),
        }
    }

    fn create_session(&mut self, initiator_id: usize, initiator_name: String) -> VotingSession {
        let session_id = generate_random_id();
        let session = VotingSession::new(session_id, initiator_id, initiator_name);
        self.sessions.insert(session_id, session.clone());
        session
    }

    // dispatch the message to the right participant
    fn send_message(&self, participant_id: usize, message: PokerMessage) {
        if let Some((_, recipient)) = self
            .clients
            .iter()
            .find(|entry| -> bool { *entry.0 == participant_id })
        {
            let _ = recipient.do_send(message);
        } else {
            println!(
                "Trying to dispatch message to unknown participant {}",
                participant_id
            );
        };
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

impl Handler<Connect> for Server {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> usize {
        let client_id = generate_random_id();
        self.clients.insert(client_id.clone(), msg.addr);
        client_id
    }
}

impl Handler<Disconnect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        if let Some(session) = self.sessions.get_mut(&msg.session_id) {
            if session.participants.len() == 1 {
                self.sessions.remove(&msg.session_id);
            } else {
                if let Some(pos) = session
                    .participants
                    .iter()
                    .position(|p| p.id == msg.participant_id)
                {
                    let removed = session.participants.remove(pos);
                    let participant_ids: Vec<usize> =
                        session.participants.iter().map(|p| p.id).collect();
                    participant_ids.iter().for_each(|p| {
                        let message = PokerMessage::ParticipantLeaveAnnouncement {
                            participant_name: removed.name.clone(),
                        };
                        self.send_message(*p, message);
                    });
                } else {
                    println!("For some reason the participant wasn't in the expected session?!");
                }
            }
        } else {
            println!(
                "Client is trying to leave non-existing session {}",
                msg.session_id
            );
        }

        self.clients.remove(&msg.participant_id);
    }
}

impl Handler<PokerMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: PokerMessage, _: &mut Context<Self>) {
        match msg {
            PokerMessage::CreateSessionRequest {
                participant_id,
                participant_name,
            } => {
                self.handle_create_session_request(participant_id, participant_name);
            }
            PokerMessage::JoinSessionRequest {
                participant_id,
                participant_name,
                session_id,
            } => self.handle_join_session_request(session_id, participant_id, participant_name),
            _ => {
                println!("Message not handled: {:?}", msg);
            }
        }
    }
}

impl Server {
    fn handle_create_session_request(&mut self, participant_id: usize, participant_name: String) {
        let session = self.create_session(participant_id, participant_name);
        let current_participant_names = session.participant_names();
        self.send_message(
            participant_id,
            PokerMessage::SessionInfoResponse {
                session_id: session.id,
                current_issue: session.current_issue,
                current_participants: current_participant_names,
            },
        );
    }

    fn handle_join_session_request(
        &mut self,
        session_id: usize,
        participant_id: usize,
        participant_name: String,
    ) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            let current_participant_ids: Vec<usize> =
                session.participants.iter().map(|p| p.id).collect();
            let message = PokerMessage::SessionInfoResponse {
                session_id: session.id,
                current_issue: session.current_issue.clone(),
                current_participants: session.participant_names(),
            };
            // TODO: investigate parallelism
            session.participants.push(VotingParticipant::new(
                participant_id,
                participant_name.clone(),
            ));
            self.send_message(participant_id, message);
            current_participant_ids.iter().for_each(|participant_id| {
                let message = PokerMessage::ParticipantJoinAnnouncement {
                    participant_name: participant_name.clone(),
                };
                self.send_message(*participant_id, message);
            });
        } else {
            self.send_message(
                participant_id,
                PokerMessage::SessionUnknownResponse { session_id },
            );
        }
    }
}
