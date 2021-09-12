import {Writable, writable} from "svelte/store";

const socket = new WebSocket("ws://localhost:8080/ws");

// Connection opened
socket.addEventListener("open", function (event) {
    console.log("It's open");
});

async function waitForOpenSocket() {
    return new Promise<void>((resolve, reject) => {
        if (socket.readyState !== socket.OPEN) {
            socket.addEventListener("open", (_) => {
                resolve();
            })
            setTimeout(() => {
                if (socket.readyState !== socket.OPEN) {
                    reject();
                }
            }, 5000);
        } else {
            resolve();
        }
    });
}

function sendJson(data) {
    if (socket.readyState <= 1) {
        socket.send(JSON.stringify(data));
    }
}

export enum Vote {
    Unknown = "Unknown",
    One = "One",
    Two = "Two",
    Three = "Three",
    Five = "Five",
    Eight = "Eight",
    Thirteen = "Thirteen",
    TwentyOne = "TwentyOne",
    Infinite = "Infinite",
}

export enum SessionJoinError {
    UnknownSession = "UnknownSession",
    ParticipantNameTaken = "ParticipantNameTaken",
}

export enum VotingState {
    Opening = "Opening",
    Voting = "Voting",
    Closing = "Closing",
}

export interface VotingSession {
    error: SessionJoinError | null,
    id: number,
    my_name: string,
    participants: string[],
    current_issue: VotingIssue,
}

export interface VotingIssue {
    id: number,
    state: VotingState,
    votes: Record<string, Vote | boolean>,
    trello_card: string | null,
    outcome: Vote
}

export type Store<T> = Writable<T> & { get(): T };

interface SessionStore extends Store<Partial<VotingSession>> {
    createSession(name: string);

    joinSession(session_id: number, name: string);
}


const blankSession: Partial<VotingSession> = {
    error: null,
    id: 0,
    my_name: "",
    participants: []
}

const blankIssue: VotingIssue = {
    trello_card: null,
    id: 0,
    votes: {},
    state: VotingState.Opening,
    outcome: Vote.Unknown
}

const LOCAL_STORAGE_KEY = "session";
let currentSession: Partial<VotingSession>;
let currentIssue: VotingIssue;

function createSession(my_name: string) {
    currentSession.my_name = my_name;
    sendJson({
        CreateSessionRequest: {
            participant_name: my_name,
        }
    });
}

function joinSession(session_id: number, my_name: string) {
    currentSession.my_name = my_name;
    sendJson({
        JoinSessionRequest: {
            participant_name: my_name,
            session_id
        }
    })
}

function changeTopic(trello_card: string) {
    sendJson({
        TopicChangeRequest: {
            trello_card
        }
    })
}

function castVote(vote: Vote) {
    sendJson({
        VoteRequest: {
            issue_id: currentIssue.id,
            vote,
        },
    })
}

interface IssueStore extends Writable<VotingIssue> {
    changeTopic(trello_card: string);
    castVote(vote: Vote);
}

function createIssueStore(): IssueStore {
    const { subscribe, set, update } = writable(blankIssue);
    return {
        subscribe,
        set,
        update,
        changeTopic,
        castVote
    }
}

function createSessionStore(): SessionStore {
    const saveState = (session: Partial<VotingSession>): Partial<VotingSession> => {
        localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(session, ["id", "my_name"]));
        return currentSession = session;
    }

    let persistedState = localStorage.getItem(LOCAL_STORAGE_KEY);
    if (persistedState === null) {
        currentSession = blankSession;
        saveState(currentSession);
    } else {
        currentSession = {
            ...blankSession,
            ...JSON.parse(persistedState)
        };
        if (currentSession.id > 0) {
            waitForOpenSocket().then(() => {
                joinSession(currentSession.id, currentSession.my_name);
            })
        }
    }

    const {subscribe, set, update} = writable<Partial<VotingSession>>(currentSession)

    return {
        subscribe,
        update: (callback) => {
            update((oldValue) => saveState(callback(oldValue)));
        },
        get: () => currentSession,
        set: (session: VotingSession) => {
            return set(saveState(session));
        },
        createSession,
        joinSession,
    }
}

const messageHandlers = {
    SessionInfoResponse: ({
                              session_id,
                              current_issue,
                              current_participants
                          }: { session_id: number, current_issue: VotingIssue, current_participants: string[] }) => {
        sessionStore.update((current) => {
                return {
                    ...current,
                    error: null,
                    id: session_id,
                    participants: current_participants,
                }
            }
        );
        issueStore.set(current_issue);
        currentIssue = current_issue;
    },
    SessionJoinErrorResponse: ({session_id, error}) => {
        console.log(`Failed to join session: ${error}`);
        sessionStore.update((current) => {
          return {
              ...current,
              id: session_id,
              error: error,
          }
        })
    },
    ParticipantJoinAnnouncement: ({participant_name}) => {
        sessionStore.update((current) => {
            current.participants.push(participant_name)
            return current
        })
    },
    ParticipantLeaveAnnouncement: ({participant_name}) => {
        sessionStore.update((current) => {
            current.participants = current.participants.filter((p) => p != participant_name)
            return current
        })
    },
    VotingIssueAnnouncement: ({voting_issue}) => {
        issueStore.set(currentIssue = voting_issue)
    },
    VoteReceiptAnnouncement: ({ participant_name, issue_id }) => {
        issueStore.update( (current) => {
            if (current.id != issue_id) {
                console.log("Received vote for unknown issue")
                return current
            }
            current.votes[participant_name] = true
            return currentIssue = current
        })
    }
}

socket.addEventListener("message", function (event) {
    console.log(`Message: ${event.data}`);
    const decoded = JSON.parse(event.data);
    const messageType = Object.keys(decoded)[0];
    if (messageHandlers.hasOwnProperty(messageType)) {
        messageHandlers[messageType](decoded[messageType]);
    } else {
        console.log(`Unknown Message: ${event.data}`);
    }
});

const sessionStore = createSessionStore();
const issueStore = createIssueStore();
export { sessionStore, issueStore };
