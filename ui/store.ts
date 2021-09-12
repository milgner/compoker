import {Writable, writable} from "svelte/store";

const socket = new WebSocket("ws://localhost:8080/ws");

// Connection opened
socket.addEventListener("open", function (event) {
    console.log("It's open");
});

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

export enum VotingState {
    Opening = "Opening",
    Voting = "Voting",
    Closing = "Closing",
}

export interface VotingSession {
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

interface SessionStore extends Writable<VotingSession> {
    createSession(name: string);
    joinSession(session_id: number, name: string);
}


const blankSession: VotingSession = {
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

function createSessionStore(): SessionStore  {
    const {subscribe, set, update} = writable<VotingSession>(blankSession)

    return {
        subscribe,
        update,
        set,
        createSession: (my_name: string) => {
            sendJson({
                CreateSessionRequest: {
                    participant_name: my_name,
                }
            });
        },
        joinSession: (session_id: number, my_name: string) => {
            sendJson({
                JoinSessionRequest: {
                    participant_name: my_name,
                    session_id
                }
            })
        }
    }
}

const sessionStore = createSessionStore();

const messageHandlers = {
    SessionInfoResponse: ({ session_id, current_issue, current_participants }: { session_id: number, current_issue: VotingIssue, current_participants: string[]}) => {
        sessionStore.update((current) => {
                return {
                    ...current,
                    current_issue,
                    id: session_id,
                    participants: current_participants,
                }
            }
        );
    },
    SessionUnknownResponse: ({ session_id }) => {
        sessionStore.set(blankSession)
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

export default sessionStore;
