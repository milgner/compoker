<script lang="ts">
    import UserAvatar24 from "carbon-icons-svelte/lib/UserAvatar24"
    import {userInfoStore} from "./store"
    import {onDestroy} from "svelte"
    import {ImageLoader} from "carbon-components-svelte";

    export let participant_name
    let avatar_url: string | undefined = undefined

    const unsubscribeUserInfoStore = userInfoStore.subscribe((updated) => {
       avatar_url = updated[participant_name]?.avatar_url
    })

    onDestroy(() => {
        unsubscribeUserInfoStore()
    })
</script>
{#if avatar_url}
    <div class="avatar-image">
        <ImageLoader src={avatar_url}></ImageLoader>
    </div>
{:else}
    <UserAvatar24></UserAvatar24>
{/if}
<style>
    .avatar-image {
        height: 24px;
        width: 24px;
        display: inline-block;
    }
</style>