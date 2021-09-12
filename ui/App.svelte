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

    import sessionStore from "./store";
    let session;

    sessionStore.subscribe(value => {
        session = value;
    });
</script>
<main>
    <Header platformName="Svactix Poker" bind:isSideNavOpen>
        <HeaderNav>
        </HeaderNav>
    </Header>
    <Content>
        {#if session.error === null}
            <Session />
        {:else}
            <JoinOrCreateSession />
        {/if}
    </Content>
</main>
