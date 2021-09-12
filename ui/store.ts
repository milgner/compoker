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
    votes: Record<string, Vote>,
    trello_card: string | null,
    outcome: Vote
}

export type Store<T> = Writable<T> & { get(): T };

interface SessionStore extends Store<VotingSession> {
    createSession(name: string);

    joinSession(session_id: number, name: string);
}


const blankSession: VotingSession = {
    error: null,
    id: 0,
    my_name: "",
    participants: [],
    current_issue: {
        id: 0,
        votes: {},
        outcome: Vote.Unknown,
        trello_card: null,
        state: VotingState.Opening
    }
}

const LOCAL_STORAGE_KEY = "session";
let currentValue: VotingSession;

function createSession(my_name: string) {
    currentValue.my_name = my_name;
    sendJson({
        CreateSessionRequest: {
            participant_name: my_name,
        }
    });
}

function joinSession(session_id: number, my_name: string) {
    currentValue.my_name = my_name;
    sendJson({
        JoinSessionRequest: {
            participant_name: my_name,
            session_id
        }
    })
}

function createSessionStore(): SessionStore {
    const saveState = (session: VotingSession): VotingSession => {
        localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(session, ["id", "my_name"]));
        return currentValue = session;
    }

    let persistedState = localStorage.getItem(LOCAL_STORAGE_KEY);
    if (persistedState === null) {
        currentValue = blankSession;
        saveState(currentValue);
    } else {
        currentValue = {
            ...blankSession,
            ...JSON.parse(persistedState)
        };
        if (currentValue.id > 0) {
            waitForOpenSocket().then(() => {
                joinSession(currentValue.id, currentValue.my_name);
            })
        }
    }

    const {subscribe, set, update} = writable<VotingSession>(currentValue)

    return {
        subscribe,
        update: (callback) => {
            update((oldValue) => saveState(callback(oldValue)));
        },
        get: () => currentValue,
        set: (session: VotingSession) => {
            return set(saveState(session));
        },
        createSession,
        joinSession
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
                    current_issue,
                    error: null,
                    id: session_id,
                    participants: current_participants,
                }
            }
        );
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
export default sessionStore;
