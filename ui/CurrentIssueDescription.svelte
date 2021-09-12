<script lang="ts">
    import sessionStore from "./store";
    import {TextInput} from "carbon-components-svelte";
    import {afterUpdate, onMount} from "svelte";

    let session;

    sessionStore.subscribe((updated) => {
        session = updated;
    });

    async function requestTopicChange(event) {
        sessionStore.changeTopic(event.target.value);
    }

    let trelloCardHolder;
    afterUpdate(() => {
       if (session.current_issue?.trello_card?.match(/^https?:\/\/trello.com\/c\//)) {
           trelloCardHolder.innerHTML = "";
           window.TrelloCards.create(session.current_issue.trello_card, trelloCardHolder, {
               compact: true,
           });
       }
    });
</script>

<div class="issue-description">
    <TextInput placeholder="Insert Trello card URL" value="{session.current_issue.trello_card}" on:input={requestTopicChange}></TextInput>
    <div class="trello-card-holder" bind:this={trelloCardHolder}>
    </div>
</div>

<style>
    .trello-card-holder {
        text-align: center;
        padding: 2em 0;
    }
</style>