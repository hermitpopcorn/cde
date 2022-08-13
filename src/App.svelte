<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
	import { onMount } from 'svelte'
	import { SvelteToast, toast } from '@zerodevx/svelte-toast'
	import Router, { replace } from 'svelte-spa-router'
	import { listen } from '@tauri-apps/api/event'
	import Data from './pages/Data.svelte'
	import Statistics from './pages/Statistics.svelte'
	import { refreshDatatable } from './lib/EventBus'
	
	// Custom types
	type ChangePagePayload = {
		page: string,
	}

	// Routes
	const routes = {
		'/': Data, // Default route
		'/data': Data,
		'/statistics': Statistics,
	}

	// Database
	let dbConnected = false
	let dbConnecting = false
	const dbConnect = async () => {
		dbConnecting = true
		try {
			let connect: boolean = await invoke('db_connect')
			dbConnecting = false
			dbConnected = true
			if (connect) {
				toast.push("Successfully connected to database.", { theme: { '--toastBackground': 'green' } })
				refreshDatatable.update(n => n + 1)
			}
		} catch(e) {
			dbConnecting = false
			dbConnected = false
			toast.push("Failed to connect to database.", { theme: { '--toastBackground': 'red' } })
		}
	}

	// Remove preloader and connect to DB
	onMount(() => {
		document.getElementById('app-preloader')?.remove()
		dbConnect()
	})

	// Listen to Tauri events
	listen('change_page', (event) => {
		replace(`/${(<ChangePagePayload>event.payload).page}`)
	})
</script>

<section>
	{#if !dbConnected}
		<section class="connection">
			<div class="container">
				{#if !dbConnecting}
					Not connected to database.
					<button class="btn btn-primary btn-sm" on:click={() => { dbConnect() }}>Attempt Connection</button>
				{:else}
					Connecting...
				{/if}
			</div>
		</section>
	{/if}

	<Router {routes} />

	<SvelteToast options={{ duration: 2000, reversed: true, intro: { y: 192 } }} />
</section>

<style lang="scss">
	:root {
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
			Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
		--toastContainerTop: auto;
		--toastContainerRight: auto;
		--toastContainerBottom: 5rem;
		--toastContainerLeft: calc(50vw - 8rem);
	}

	:global(.btn-icon svg) {
		vertical-align: text-bottom;
	}

	section.connection {
		background-color: rgba(255, 173, 173, 0.7);
		vertical-align: middle;
	}
</style>
