<script lang="ts">
    import {Button, Column, Form, FormGroup, Grid, Row, TextInput, Tile} from "carbon-components-svelte";

    import sessionStore, {SessionJoinError} from './store';

    async function handleSubmit(event) {
        const form = event.target;
        const sessionId = Number(form.sessionId.value);
        const myName = form.myName.value;
        if (sessionId > 0) {
            sessionStore.joinSession(sessionId, myName);
        } else {
            sessionStore.createSession(myName);
        }
    }

    let oldSession = sessionStore.get();
</script>

<Grid>
    <Row>
        <Column>
            <Tile light>
                {#if oldSession.error == SessionJoinError.ParticipantNameTaken}
                    <h3>The name you requested was already taken</h3>
                {:else if oldSession.error == SessionJoinError.UnknownSession}
                    <h3>You tried to join a non-existing session</h3>
                {/if}
                <Form on:submit={handleSubmit}>
                    <FormGroup legendText="Join or create session">
                        <TextInput id="myName" labelText="Your Name" placeholder="What should we call you?" value="{oldSession.my_name}" required/>
                        <TextInput id="sessionId" labelText="Session ID" placeholder="Leave blank for new session" value="{oldSession.id}"/>
                    </FormGroup>
                    <Button type="submit">Let's go!</Button>
                </Form>
            </Tile>
        </Column>
    </Row>
</Grid>