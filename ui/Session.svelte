<script lang="ts">
    import {
        Column,
        Grid,
        Row,
        StructuredList,
        StructuredListCell,
        StructuredListRow,
    } from "carbon-components-svelte";
    import Checkmark20 from "carbon-icons-svelte/lib/Checkmark20";
    import Hourglass20 from "carbon-icons-svelte/lib/Hourglass20";
    import sessionStore from "./store";

    let session;

    sessionStore.subscribe((updated) => {
        session = updated;
    });

    function didVote(participant_name: string): boolean {
        return Object.keys(session.current_issue.votes).includes(participant_name);
    }
</script>

<Grid>
    <Row>
        <Column sm="12">
            Current session: {session.id}
        </Column>
    </Row>
    <Row>
        <Column>
            <StructuredList condensed>
            {#each session.participants as participant}
                <StructuredListRow>
                    <StructuredListCell>{#if didVote(participant)}<Checkmark20 />{:else}<Hourglass20/>{/if}</StructuredListCell>
                    <StructuredListCell>{participant}</StructuredListCell>
                </StructuredListRow>
            {/each}
            </StructuredList>
        </Column>
    </Row>
</Grid>