<script lang="ts">
    import {issueStore, sessionStore, Vote, VotingState} from "./store";
    import VoteCard from "./VoteCard.svelte";
    import {onDestroy} from "svelte";

    let my_name;
    let issue;
    let votingClosed = false;

    const sessionUnsubscribe = sessionStore.subscribe((updated) => { my_name = updated.my_name });
    const issueUnsubscribe = issueStore.subscribe((updated) => {
        issue = updated;
        votingClosed = updated.state == VotingState.Closing;
    });

    onDestroy(() => {
        sessionUnsubscribe()
        issueUnsubscribe()
    })

    let availableVotes = Object.keys(Vote)
    availableVotes.shift()

</script>
<div class="voting-area-container">
    <div class="voting-area">
        {#each availableVotes as vote}
            <VoteCard vote="{vote}" disabled="{votingClosed}"/>
        {/each}
    </div>
</div>
<style>
    .voting-area {
        display: flex;
        flex-wrap: wrap;
        justify-content: space-between;
        align-items: center;
        gap: 1em;
    }
    /*.voting-area-container {*/
    /*    display: block;*/
    /*    width: 100%;*/
    /*    height: 100%;*/
    /*}*/
</style>