<script lang="ts">
    import {Button, Column, Form, FormGroup, Grid, Row, TextInput, Tile} from "carbon-components-svelte";

    import {sessionStore, SessionJoinError} from './store';
    import {onDestroy} from "svelte";

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

    let session;
    const unsubscribe = sessionStore.subscribe((updated) => session = updated);
    onDestroy(unsubscribe);
</script>

<Grid>
    <Row>
        <Column>
            <Tile light>
                {#if session.error == SessionJoinError.ParticipantNameTaken}
                    <h3>The name you requested was already taken</h3>
                {:else if session.error == SessionJoinError.UnknownSession}
                    <h3>You tried to join a non-existing session</h3>
                {/if}
                <Form on:submit={handleSubmit}>
                    <FormGroup legendText="Join or create session">
                        <TextInput id="myName" labelText="Your Name" placeholder="What should we call you?" value="{session.my_name}" required/>
                        <TextInput id="sessionId" labelText="Session ID" placeholder="Leave blank for new session" value="{session.id > 0 ? session.id : ''}"/>
                    </FormGroup>
                    <Button type="submit">Let's go!</Button>
                </Form>
            </Tile>
        </Column>
    </Row>
</Grid>