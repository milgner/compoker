<script lang="ts">
    import { Tile } from "carbon-components-svelte";
    import {
        Form,
        FormGroup,
        Grid, Row, Column,
        Button,
        TextInput
    } from "carbon-components-svelte";
</script>

<Grid>
    <Row>
        <Column>
<Tile light>
    <Form on:submit>
        <FormGroup legendText="Join Session">
            <TextInput labelText="Session ID" placeholder="You'll receive this from the person starting the session" required />
            <TextInput labelText="Your Name" placeholder="What should we call you?" required />
        </FormGroup>
        <Button type="submit">Let's go!</Button>
    </Form>
</Tile>
        </Column>
    </Row>
</Grid>