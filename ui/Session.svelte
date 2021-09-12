<script lang="ts">
    import {Column, Grid, Row, Tile} from "carbon-components-svelte";

    import sessionStore from "./store";

    let session;

    sessionStore.subscribe((updated) => {
        session = updated;
    });
</script>

<h1>We are in session {session.id}</h1>
<Grid>
    <Row>
        <Column>
            {#each session.participants as participant}
                <Tile light>
                    {participant}
                </Tile>
            {/each}
        </Column>
    </Row>
</Grid>