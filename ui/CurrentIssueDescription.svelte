<svelte:options immutable={true}/>
<script lang="ts">
    import {issueStore} from "./store";
    import {TextInput} from "carbon-components-svelte";
    import {afterUpdate, onDestroy, onMount} from "svelte";

    // so that not every keypress immediately updates the issue but waits a bit
    const INPUT_DEBOUNCE_INTERVAL = 300;

    let issue;

    const issueUnsubscribe = issueStore.subscribe((updated) => {
        issue = updated;
    });

    function debounce(duration: number, callback: (...args: any[]) => any) {
        let interval = null;

        return function(params) {
            if (interval != null) {
                clearInterval(interval);
            }
            interval = setInterval(() => {
                interval = null;
                callback(params)
            }, duration);
        }
    }

    const requestTopicChange = debounce(INPUT_DEBOUNCE_INTERVAL, (event) => {
        issueStore.changeTopic(event.target.value);
    })

    let trelloCardHolder;
    afterUpdate(() => {
        trelloCardHolder.innerHTML = "";

        if (issue.trello_card?.match(/^https?:\/\/trello.com\/c\//)) {
           window.TrelloCards.create(issue.trello_card, trelloCardHolder, {
               compact: true,
           });
       }
    });

    onDestroy(issueUnsubscribe);
</script>

<div class="issue-description">
    <TextInput placeholder="Describe issue or paste Trello card URL" value="{issue.trello_card}" on:input={requestTopicChange}></TextInput>
    <div class="trello-card-holder" bind:this={trelloCardHolder}>
    </div>
</div>

<style>
    .trello-card-holder {
        text-align: center;
        padding: 2em 0;
    }
</style>