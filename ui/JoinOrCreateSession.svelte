<script lang="ts">
    import {Button, Column, Form, FormGroup, Grid, Row, TextInput, Tile} from "carbon-components-svelte";

    import sessionStore from './store';

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
</script>

<Grid>
    <Row>
        <Column>
            <Tile light>
                <Form on:submit={handleSubmit}>
                    <FormGroup legendText="Join or create session">
                        <TextInput id="myName" labelText="Your Name" placeholder="What should we call you?" required/>
                        <TextInput id="sessionId" labelText="Session ID" placeholder="Leave blank for new session"/>
                    </FormGroup>
                    <Button type="submit">Let's go!</Button>
                </Form>
            </Tile>
        </Column>
    </Row>
</Grid>