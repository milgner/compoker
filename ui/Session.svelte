<script lang="ts">
    import {
        Column,
        Grid,
        Row,
    } from "carbon-components-svelte";
    import SessionParticipantList from "./SessionParticipantList.svelte";

    import CurrentIssueDescription from "./CurrentIssueDescription.svelte";
    import VotingArea from "./VotingArea.svelte";

    let session;

    import sessionStore from "./store";
    sessionStore.subscribe((updated) => {
        session = updated;
    });
</script>

<Grid>
    <Row>
        <Column sm="12">
            Current session: {session.id}
        </Column>
    </Row>
    <Row>
        <Column>
            <SessionParticipantList/>
        </Column>
        <Column>
            <CurrentIssueDescription/>
            <VotingArea/>
        </Column>
    </Row>
</Grid>