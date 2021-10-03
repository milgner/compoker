<script lang="ts">
    import ParticipantAvatar from "./ParticipantAvatar.svelte"
    import {userInfoStore} from "./store";
    import {onDestroy} from "svelte";

    export let participant_name: string

    let display_name = participant_name;

    const unsubscribeUserInfo = userInfoStore.subscribe((updated) => {
        display_name = updated[participant_name]?.display_name || participant_name
    })

    onDestroy(unsubscribeUserInfo)
</script>
<div class="participant-list-entry">
    <ParticipantAvatar participant_name={participant_name}></ParticipantAvatar> <span class="display-name">{display_name}</span>
</div>
<style>
    .participant-list-entry {
        display: flex;
        justify-content: flex-start;
        align-items: center;
    }

    .display-name {
        margin-left: 0.5em;
    }
</style>