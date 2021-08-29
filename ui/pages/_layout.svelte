<script lang="ts">
    import {
        Header,
        HeaderNav,
        HeaderNavItem,
        Content
    } from "carbon-components-svelte";
    let isSideNavOpen = false;
</script>
<main>
    <Header platformName="Svactix Poker" bind:isSideNavOpen>
        <HeaderNav>
            <HeaderNavItem href="/NewSession" text="Create Session" />
            <HeaderNavItem href="/JoinSession" text="Join Session" />
        </HeaderNav>
    </Header>
    <Content>
        <slot>
        </slot>
    </Content>
</main>
