<script lang="ts">
    import {
        Header,
        HeaderNav,
        HeaderNavItem,
        Content
    } from "carbon-components-svelte";

    import Session from "./Session.svelte";
    import JoinOrCreateSession from "./JoinOrCreateSession.svelte";

    let isSideNavOpen = false;

    import { sessionStore } from "./store";
    import {onDestroy} from "svelte";
    let session;

    const unsubscribe = sessionStore.subscribe(value => {
        session = value;
    });

    onDestroy(unsubscribe);
</script>
<main>
    <Header platformName="Svactix Poker" bind:isSideNavOpen>
        <HeaderNav>
        </HeaderNav>
    </Header>
    <Content>
        {#if session.error === null && session.id > 0}
            <Session />
        {:else}
            <JoinOrCreateSession />
        {/if}
    </Content>
</main>
