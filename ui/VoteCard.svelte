<script lang="ts">
    import {ImageLoader} from "carbon-components-svelte";
    import {Vote} from "./store";

    export let vote: Vote;

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
</script>

<div class="vote-card">
    <div class="background">
        <ImageLoader class="background-image" src="/images/cards/{vote.toLowerCase()}.gif" alt="Complexity {vote}" ratio="3x4"/>
    </div>
    <div class="foreground">
        {overlays[vote]}
    </div>
</div>

<style global>
    .background {
        background-color: black;
        position: relative;
        top: 0;
        z-index: 0;
        border-radius: 0.5em;
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
        color: gray;
        line-height: 200px;
    }
    .vote-card {
        width: 150px;
        height: 200px;
        display: inline-block;
        position: relative;
        border-radius: 0.5em;
    }
</style>
