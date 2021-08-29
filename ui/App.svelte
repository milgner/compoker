<script lang="ts">
	import {
		Theme,
		Header,
		HeaderNav,
		HeaderNavItem,
		SkipToContent,
		Content,
		Grid,
		Row,
		Column,
	} from "carbon-components-svelte";

	let isSideNavOpen = false;
</script>

<main>
	<Theme theme="g80" persist persistKey="__carbon-theme"/>

		<Header company="Marcus Ilgner" platformName="Svactix Poker" bind:isSideNavOpen>
			<div slot="skip-to-content">
				<SkipToContent />
			</div>

			<HeaderNav>
				<HeaderNavItem href="/" text="Create Session" />
				<HeaderNavItem href="/" text="Join Session" />
			</HeaderNav>
		</Header>

		<Content>
			<Grid>
				<Row>
					<Column>
						<h1>Welcome to Svactix Poker</h1>
					</Column>
				</Row>
			</Grid>
		</Content>

</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>