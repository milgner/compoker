<script lang="ts">
    import { issueStore, Vote} from "./store";

    export let vote: Vote;
    export let disabled: boolean;
    export let active: boolean;

    const overlays = {
        [Vote.Unknown]: "?",
        [Vote.One]: "1",
        [Vote.Two]: "2",
        [Vote.Three]: "3",
        [Vote.Five]: "5",
        [Vote.Eight]: "8",
        [Vote.Thirteen]: "13",
        [Vote.TwentyOne]: "21",
        [Vote.Infinite]: "∞",
    };

    async function castVote() {
        if (disabled) {
            return
        }
        issueStore.castVote(vote);
    }
</script>

<div class="vote-card {disabled ? 'disabled' : ''} {active ? 'active' : ''}" on:click={castVote}>
    <div class="background vote-{vote.toLowerCase()}">
    </div>
    <div class="foreground">
        {overlays[vote]}
    </div>
</div>

<style global>
    .background {
        position: relative;
        top: 0;
        z-index: 0;
        border-radius: 0.5em;
        width: 100%;
        height: 100%;
    }

    .vote-card:hover .foreground {
        transition: all 0.4s;
        transform: rotateY(360deg);
    }

    .vote-card:not(.disabled) {
        cursor: pointer;
    }

    .vote-card.active .background {
        border: 4px solid green;
    }

    .vote-card.active .background::after {
        position: absolute;
        right: 0;
        bottom: 0;
        padding-right: 0.5em;
        padding-bottom: 0.5em;
        text-align: right;
        color: green;
        content: '✓';
        font-size: larger;
    }

    .foreground {
        top: 0;
        left: 0;
        width: 100%;
        height: 200px;
        text-align: center;
        position: absolute;
        font-size: 8em;
        z-index: 1;
        mix-blend-mode: screen;
        color: #dddddd;
        line-height: 200px;
    }
    .vote-card {
        width: 150px;
        height: 200px;
        display: inline-block;
        position: relative;
        border-radius: 0.5em;
        user-select: none;
    }
</style>
