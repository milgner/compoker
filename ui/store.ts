import {Writable, writable} from "svelte/store";
import { md5 } from "hash-wasm";

const socket = new WebSocket(process.env.SERVER_URL);

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
    Secret = "Secret",
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

export interface UserInfo {
    display_name: string,
    avatar_url: String | undefined,
}

export type Store<T> = Writable<T> & { get(): T };

interface SessionStore extends Store<Partial<VotingSession>> {
    createSession(name: string);

    joinSession(session_id: number, name: string);
}

export type UserInfoStore = Writable<Record<string, UserInfo>>;

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
    outcome: Vote.Unknown,
}

const LOCAL_STORAGE_KEY = "session";
let currentSession: Partial<VotingSession>;
let currentIssue: VotingIssue;
let myVote: Vote;

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
    myVote = vote;
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
    const {subscribe, set, update} = writable(blankIssue);
    return {
        subscribe,
        set,
        update,
        changeTopic,
        castVote
    }
}

export type UserInfoStrategy = (username: string) => Promise<UserInfo> | undefined;

interface GithubGitlabApiUserInfo {
    name: string,
    avatar_url: string
}

async function lookupUserInfoFromGithub(username: string): Promise<UserInfo | undefined> {
    const githubUsername = username.match(/^(\w+)@github$/)
    if (githubUsername == null) {
        return undefined
    }
    const response = await fetch(`https://api.github.com/users/${githubUsername[1]}`)
    if (response.status != 200) {
        return undefined
    }
    const json: GithubGitlabApiUserInfo = await response.json()
    return {
        display_name: json.name,
        avatar_url: json.avatar_url
    }
}

async function lookupUserInfoFromGitlab(username: string): Promise<UserInfo | undefined> {
    // FIXME: can only look up from Gitlab.com at the moment
    // but there's no easy way to detect whether it's from another Gitlab installation
    // and adding the TLD would make it look like a real E-mail
    const gitlabUsername = username.match(/^(\w+)@gitlab$/)
    if (gitlabUsername == null) {
        return undefined
    }
    const response = await fetch(`https://gitlab.com/api/v4/users/?username=${gitlabUsername[1]}`)
    if (response.status != 200) {
        return undefined
    }
    const json: Array<GithubGitlabApiUserInfo> = await response.json()
    if (json.length > 0) {
        const userInfo = json[0]
        return {
            display_name: userInfo.name,
            avatar_url: userInfo.avatar_url
        }
    }
}

const USER_INFO_STRATEGIES: UserInfoStrategy[] = [
    lookupUserInfoFromGithub,
    lookupUserInfoFromGitlab,
];

async function lookupUserInfo(username: string) {
    for (let strategy of USER_INFO_STRATEGIES) {
        let user_info = await strategy(username)
        if (user_info) {
            userInfoStore.update((current) => {
                current[username] = user_info
                return current
            })
            return
        }
    }
}

function ensureUserInStore(username: string) {
    userInfoStore.update((currentUsers) => {
        if (!currentUsers.hasOwnProperty(username)) {
            currentUsers[username] = {
                display_name: username,
                avatar_url: undefined
            }
            lookupUserInfo(username)
        }
        return currentUsers
    })
}

function createUserInfoStore(): UserInfoStore {
    return writable<Record<string, UserInfo>>({});
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
                for (let username of current_participants) {
                    ensureUserInStore(username);
                }
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
                id: error == SessionJoinError.ParticipantNameTaken ? session_id : null,
                error: error,
            }
        })
    },
    ParticipantJoinAnnouncement: ({participant_name}) => {
        sessionStore.update((current) => {
            ensureUserInStore(participant_name)
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
        myVote = null
    },
    VoteReceiptAnnouncement: ({participant_name, issue_id}) => {
        issueStore.update((current) => {
            if (current.id != issue_id) {
                console.log("Received vote for unknown issue")
                return current
            }
            if (participant_name == currentSession.my_name) {
                currentIssue.votes[participant_name] = myVote
            } else {
                current.votes[participant_name] = Vote.Secret
            }
            return currentIssue = current
        })
    },
    VotingResultsRevelation: ({issue_id, votes, outcome}) => {
        issueStore.update((current) => {
            if (current.id != issue_id) {
                console.log("Received information about unknown vote")
                return current
            }
            current.state = VotingState.Closing
            current.votes = votes
            current.outcome = outcome
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
const userInfoStore = createUserInfoStore();
export {sessionStore, issueStore, userInfoStore};
