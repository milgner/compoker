<script lang="ts">
    import sessionStore from "./store";
    import {TextInput} from "carbon-components-svelte";

    let session;

    sessionStore.subscribe((updated) => {
        session = updated;
    });
</script>

<TextInput placeholder="Insert Trello card URL"></TextInput>