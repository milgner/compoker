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
        <FormGroup legendText="Create Session">
            <TextInput labelText="Session name" placeholder="Name your session" required />
        </FormGroup>
        <Button type="submit">Let's go!</Button>
    </Form>
</Tile>
        </Column>
    </Row>
</Grid>