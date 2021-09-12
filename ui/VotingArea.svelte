<script lang="ts">
    import sessionStore, {Vote} from "./store";
    import VoteCard from "./VoteCard.svelte";

    let session;

    sessionStore.subscribe((updated) => {
        session = updated;
    });
</script>
<div class="voting-area-container">
    <div class="voting-area">
        {#each Object.keys(Vote) as vote}
            <VoteCard vote="{vote}"/>
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