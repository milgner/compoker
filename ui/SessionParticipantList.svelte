<script lang="ts">
    import {StructuredList, StructuredListCell, StructuredListRow,} from "carbon-components-svelte";
    import Checkmark20 from "carbon-icons-svelte/lib/Checkmark20";
    import Hourglass20 from "carbon-icons-svelte/lib/Hourglass20";
    import {issueStore, sessionStore, userInfoStore, Vote, VotingState} from "./store";
    import {onDestroy} from "svelte";
    import ParticipantListEntry from "./ParticipantListEntry.svelte";

    let session;
    let currentIssue;

    const unsubscribeSession = sessionStore.subscribe((updated) => {
        session = updated
    });

    const unsubscribeIssue = issueStore.subscribe((updated) => {
        currentIssue = updated
    });

    onDestroy(() => {
        unsubscribeIssue()
        unsubscribeSession()
    })
</script>

<StructuredList condensed>
    {#each session.participants as participant}
        <StructuredListRow>
            <StructuredListCell>
                {#if currentIssue.votes[participant]}
                    <Checkmark20/>
                {:else}
                    <Hourglass20/>
                {/if}
            </StructuredListCell>
            <StructuredListCell>
                <ParticipantListEntry participant_name={participant}></ParticipantListEntry>
            </StructuredListCell>
            <StructuredListCell>
                {#if currentIssue.state == VotingState.Closing && currentIssue.votes[participant] }
                    {currentIssue.votes[participant]}
                {/if}
            </StructuredListCell>
        </StructuredListRow>
    {/each}
</StructuredList>