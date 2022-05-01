//! The `PokerServer` is an actor that maintains a list of sessions,
//! their participants and current votes

use bastion::prelude::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use rand::{self, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;

// helper function to generate a random id string
fn generate_random_id() -> u32 {
    thread_rng().gen::<u32>()
}

pub struct Disconnect {
    pub participant_id: u32,
    pub session_id: u32,
}

fn zero_id() -> u32 {
    0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SessionJoinError {
    UnknownSession,
    ParticipantNameTaken,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PokerMessage {
    // a client requests to create a session
    CreateSessionRequest {
        #[serde(default = "zero_id")]
        participant_id: u32,
        participant_name: String,
    },
    // a client requests to join a session
    JoinSessionRequest {
        #[serde(default = "zero_id")]
        participant_id: u32,
        session_id: u32,
        participant_name: String,
    },
    // the server sends the client the state of the current session
    SessionInfoResponse {
        session_id: u32,
        current_issue: VotingIssue,
        current_participants: Vec<String>,
    },
    // the server notifies the client that joining the session failed
    SessionJoinErrorResponse {
        session_id: u32,
        error: SessionJoinError,
    },
    // the server announces to everyone else that a new participant entered their session
    ParticipantJoinAnnouncement {
        participant_name: String,
    },
    // the server announces to everyone else that someone left their session
    ParticipantLeaveAnnouncement {
        participant_name: String,
    },
    // the client requests to change the issue being voted upon
    TopicChangeRequest {
        #[serde(default = "zero_id")]
        participant_id: u32,
        #[serde(default = "zero_id")]
        session_id: u32,
        trello_card: String,
    },
    // the server announces a new issue being voted on
    VotingIssueAnnouncement {
        voting_issue: VotingIssue,
    },
    // the client sends the server its vote
    VoteRequest {
        #[serde(default = "zero_id")]
        participant_id: u32,
        #[serde(default = "zero_id")]
        session_id: u32,
        issue_id: u32,
        vote: Vote,
    },
    // the server announces that it received a vote from a specific user
    VoteReceiptAnnouncement {
        participant_name: String,
        issue_id: u32,
    },
    // the client requests for the votes to be revealed
    VoteRevelationRequest {
        #[serde(default = "zero_id")]
        participant_id: u32,
        issue_id: u32,
    },
    // the server reveals all the votes
    VotingResultsRevelation {
        issue_id: u32,
        votes: HashMap<String, Vote>,
        outcome: Vote,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Vote {
    Secret,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum VotingState {
    Opening,
    Voting,
    Closing,
}

pub struct VotingParticipant {
    id: u32,
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
    pub fn new(id: u32, name: String) -> VotingParticipant {
        VotingParticipant { id, name }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VotingIssue {
    id: u32,
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
    pub fn new(trello_card: Option<String>) -> VotingIssue {
        VotingIssue {
            id: generate_random_id(),
            votes: HashMap::new(),
            outcome: None,
            state: VotingState::Opening,
            trello_card,
        }
    }

    // clone this issue but with all votes set to Secret
    pub fn clone_blinded(&self, participant_name: Option<&String>) -> VotingIssue {
        let votes: HashMap<String, Vote> = match self.state.clone() {
            VotingState::Closing => self.votes.clone(),
            _ => self
                .votes
                .iter()
                .map(|entry| {
                    let vote = match participant_name {
                        Some(p) if p == entry.0 => entry.1.clone(),
                        _ => Vote::Secret,
                    };
                    (entry.0.clone(), vote)
                })
                .collect(),
        };
        VotingIssue {
            id: self.id.clone(),
            votes,
            outcome: self.outcome.clone(),
            state: self.state.clone(),
            trello_card: self.trello_card.clone(),
        }
    }
}

struct VotingSession {
    id: u32,
    participants: Vec<VotingParticipant>,
    current_issue: VotingIssue,
}

impl VotingSession {
    pub fn new(session_id: u32, initiator_id: u32, initiator_name: String) -> VotingSession {
        VotingSession {
            id: session_id,
            participants: vec![VotingParticipant::new(initiator_id, initiator_name)],
            current_issue: VotingIssue::new(None),
        }
    }

    pub fn participant_names(&self) -> Vec<String> {
        self.participants.iter().map(|p| p.name.clone()).collect()
    }

    pub fn participant_ids(&self) -> Vec<u32> {
        self.participants.iter().map(|p| p.id.clone()).collect()
    }

    pub fn all_votes_cast(&self) -> bool {
        self.participants
            .iter()
            .all(|p| self.current_issue.votes.contains_key(p.name.as_str()))
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

// pub struct Server {
//     sessions: HashMap<u32, VotingSession>,
//     timeout_sessions: HashMap<u32, std::time::Instant>,
//     clients: HashMap<u32, Recipient<PokerMessage>>,
// }
//
// impl Server {
//     pub fn new() -> Server {
//         Server {
//             sessions: HashMap::new(),
//             clients: HashMap::new(),
//             timeout_sessions: HashMap::new(),
//         }
//     }
//
//     fn create_session(&mut self, initiator_id: u32, initiator_name: String) -> VotingSession {
//         let session_id = generate_random_id();
//         let session = VotingSession::new(session_id, initiator_id, initiator_name);
//         self.sessions.insert(session_id, session.clone());
//         session
//     }
//
//     // dispatch the message to the right participant
//     fn send_message(&self, participant_id: u32, message: PokerMessage) {
//         if let Some((_, recipient)) = self
//             .clients
//             .iter()
//             .find(|entry| -> bool { *entry.0 == participant_id })
//         {
//             let _ = recipient.do_send(message);
//         } else {
//             tracing::error!(
//                 "Trying to dispatch message to unknown participant {}",
//                 participant_id
//             );
//         };
//     }
// }
//
// impl Actor for Server {
//     type Context = Context<Self>;
//
//     fn started(&mut self, ctx: &mut Self::Context) {
//         self.start_session_timeout_check(ctx);
//     }
// }
//
// impl Handler<Connect> for Server {
//     type Result = u32;
//
//     fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> u32 {
//         let client_id = generate_random_id();
//         self.clients.insert(client_id.clone(), msg.addr);
//         client_id
//     }
// }
//
// impl Handler<Disconnect> for Server {
//     type Result = ();
//
//     fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
//         if let Some(session) = self.sessions.get_mut(&msg.session_id) {
//             if session.participants.len() == 1 {
//                 session.participants.clear();
//                 self.timeout_sessions
//                     .insert(session.id, std::time::Instant::now());
//             } else {
//                 // TODO: it should be perfectly acceptable to factor this out but it does not work
//                 if let Some(pos) = session
//                     .participants
//                     .iter()
//                     .position(|p| p.id == msg.participant_id)
//                 {
//                     let removed = session.participants.remove(pos);
//                     let participant_ids: Vec<u32> =
//                         session.participants.iter().map(|p| p.id).collect();
//                     participant_ids.iter().for_each(|p| {
//                         let message = PokerMessage::ParticipantLeaveAnnouncement {
//                             participant_name: removed.name.clone(),
//                         };
//                         self.send_message(*p, message);
//                     });
//                 } else {
//                     println!("For some reason the participant wasn't in the expected session?!");
//                 }
//             }
//             self.reveal_if_everyone_voted(msg.session_id);
//         } else {
//             if msg.session_id > 0 {
//                 println!(
//                     "Client is trying to leave non-existing session {}",
//                     msg.session_id
//                 );
//             }
//         }
//
//         self.clients.remove(&msg.participant_id);
//     }
// }
//
// impl Handler<PokerMessage> for Server {
//     type Result = ();
//
//     fn handle(&mut self, msg: PokerMessage, _: &mut Context<Self>) {
//         match msg {
//             PokerMessage::CreateSessionRequest {
//                 participant_id,
//                 participant_name,
//             } => {
//                 self.handle_create_session_request(participant_id, participant_name);
//             }
//             PokerMessage::JoinSessionRequest {
//                 participant_id,
//                 participant_name,
//                 session_id,
//             } => self.handle_join_session_request(session_id, participant_id, participant_name),
//             PokerMessage::TopicChangeRequest {
//                 session_id,
//                 participant_id,
//                 trello_card,
//             } => self.handle_topic_change_request(session_id, participant_id, trello_card),
//             PokerMessage::VoteRequest {
//                 session_id,
//                 participant_id,
//                 issue_id,
//                 vote,
//             } => self.handle_vote_request(session_id, issue_id, participant_id, vote),
//             _ => {
//                 println!("Message not handled: {:?}", msg);
//             }
//         }
//     }
// }
//
// const SESSION_TIMEOUT: Duration = Duration::from_secs(20);
// const SESSION_TIMEOUT_CHECK_INTERVAL: Duration = Duration::from_secs(5);
//
// impl Server {
//     fn start_session_timeout_check(&self, ctx: &mut Context<Server>) {
//         ctx.run_interval(SESSION_TIMEOUT_CHECK_INTERVAL, |act, _| {
//             let mut sessions_to_delete = Vec::new();
//             act.timeout_sessions
//                 .retain(|session_id, last_seen| -> bool {
//                     if Instant::now().duration_since(*last_seen) > SESSION_TIMEOUT {
//                         sessions_to_delete.push(session_id.clone());
//                         false
//                     } else {
//                         true
//                     }
//                 });
//             act.sessions
//                 .retain(|session_id, _| -> bool { !sessions_to_delete.contains(&session_id) });
//         });
//     }
//
//     fn handle_create_session_request(&mut self, participant_id: u32, participant_name: String) {
//         let session = self.create_session(participant_id, participant_name.clone());
//         let current_participant_names = session.participant_names();
//         self.send_message(
//             participant_id,
//             PokerMessage::SessionInfoResponse {
//                 session_id: session.id,
//                 current_issue: session.current_issue.clone_blinded(Some(&participant_name)),
//                 current_participants: current_participant_names,
//             },
//         );
//     }
//
//     fn handle_join_session_request(
//         &mut self,
//         session_id: u32,
//         participant_id: u32,
//         participant_name: String,
//     ) {
//         if let Some(session) = self.sessions.get_mut(&session_id) {
//             // if someone joins a session that was previously set to time out, it needs to be kept alive
//             self.timeout_sessions.remove(&session_id);
//
//             // now check that the name hasn't already been taken
//             if session
//                 .participants
//                 .iter()
//                 .any(|p| p.name == participant_name)
//             {
//                 self.send_message(
//                     participant_id,
//                     PokerMessage::SessionJoinErrorResponse {
//                         session_id,
//                         error: SessionJoinError::ParticipantNameTaken,
//                     },
//                 );
//                 return;
//             }
//
//             // save the current participant list so we can notify them about someone joining
//             let current_participant_ids: Vec<u32> =
//                 session.participants.iter().map(|p| p.id).collect();
//             // add the new participant
//             session.participants.push(VotingParticipant::new(
//                 participant_id,
//                 participant_name.clone(),
//             ));
//             // and once they were added, let them know that they successfully joined
//             let message = PokerMessage::SessionInfoResponse {
//                 session_id: session.id,
//                 current_issue: session.current_issue.clone_blinded(Some(&participant_name)),
//                 current_participants: session.participant_names(),
//             };
//             self.send_message(participant_id, message);
//             // notify everyone else about the new participant
//             current_participant_ids.iter().for_each(|participant_id| {
//                 let message = PokerMessage::ParticipantJoinAnnouncement {
//                     participant_name: participant_name.clone(),
//                 };
//                 self.send_message(*participant_id, message);
//             });
//         } else {
//             self.send_message(
//                 participant_id,
//                 PokerMessage::SessionJoinErrorResponse {
//                     session_id,
//                     error: SessionJoinError::UnknownSession,
//                 },
//             );
//         }
//     }
//
//     fn handle_topic_change_request(
//         &mut self,
//         session_id: u32,
//         _participant_id: u32,
//         trello_card: String,
//     ) {
//         if let Some(session) = self.sessions.get_mut(&session_id) {
//             let trello_card: Option<String> = if trello_card.len() > 0 {
//                 Some(trello_card)
//             } else {
//                 None
//             };
//             if session.current_issue.trello_card == trello_card {
//                 return;
//             }
//             let issue = VotingIssue::new(trello_card);
//             session.current_issue = issue.clone();
//             let participant_ids = session.participant_ids();
//             participant_ids.iter().for_each(|p| {
//                 self.send_message(
//                     *p,
//                     PokerMessage::VotingIssueAnnouncement {
//                         voting_issue: issue.clone(),
//                     },
//                 );
//             });
//         }
//     }
//
//     fn handle_vote_request(
//         &mut self,
//         session_id: u32,
//         issue_id: u32,
//         participant_id: u32,
//         vote: Vote,
//     ) {
//         if let Some(session) = self.sessions.get_mut(&session_id) {
//             if session.current_issue.id != issue_id {
//                 // TODO: notify sender about issue id mismatch
//                 return;
//             }
//             let participant = session.participants.iter().find(|p| p.id == participant_id);
//             if participant.is_none() || session.current_issue.state == VotingState::Closing {
//                 return;
//             }
//             let participant_name = participant.unwrap().name.clone();
//             session
//                 .current_issue
//                 .votes
//                 .insert(participant_name.to_string(), vote);
//             {
//                 session.participant_ids().iter().for_each(|&p| {
//                     self.send_message(
//                         p,
//                         PokerMessage::VoteReceiptAnnouncement {
//                             participant_name: participant_name.to_string(),
//                             issue_id,
//                         },
//                     );
//                 });
//             }
//         }
//         self.reveal_if_everyone_voted(session_id);
//     }
//
//     fn reveal_if_everyone_voted(&mut self, session_id: u32) {
//         if let Some(session) = self.sessions.get_mut(&session_id) {
//             let participant_ids = session.participant_ids();
//
//             if !session.all_votes_cast() {
//                 return;
//             }
//             let outcome = Vote::Unknown;
//             session.current_issue.outcome = Some(outcome.clone()); // TODO: determine outcome from votes cast
//             session.current_issue.state = VotingState::Closing;
//             let issue_id = session.current_issue.id;
//             let votes = session.current_issue.votes.clone();
//             participant_ids.iter().for_each(|&p| {
//                 self.send_message(
//                     p,
//                     PokerMessage::VotingResultsRevelation {
//                         issue_id: issue_id.clone(),
//                         votes: votes.clone(),
//                         outcome: outcome.clone(),
//                     },
//                 );
//             });
//         }
//     }
// }


/// Use this distributor name if you want to send messages to the poker server
pub const SERVER_DISTRIBUTOR_NAME: &str = "PokerServer";

pub fn run() -> Result<(), simple_error::SimpleError> {
    Bastion::children(|children| {
            children.with_redundancy(1) // don't want more than 1 poker server
                .with_distributor(Distributor::named(SERVER_DISTRIBUTOR_NAME))
                .with_exec(move |context| async move {
                loop {
                    if let Some(msg) = context.try_recv().await {}
                }
            })
        })
        .map_err(|_| SimpleError::new("Failed to start poker server"))?;
    Ok(())
}
