<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
	import { onMount } from 'svelte'
	import { SvelteToast, toast } from '@zerodevx/svelte-toast'
	import * as animateScroll from "svelte-scrollto"
	import Datatable from './lib/Datatable.svelte'
	import Form from './lib/Form.svelte'

	// Component binding
	let form
	let datatable

	// ScrollTo
	animateScroll.setGlobalOptions({
		offset: 0,
		delay: 0,
		duration: 100,
	})

	// Database
	let dbConnected = false
	let dbConnecting = false
	const dbConnect = async () => {
		dbConnecting = true
		try {
			let connect: boolean = await invoke("db_connect")
			dbConnecting = false
			dbConnected = true
			if (connect) {
				toast.push("Successfully connected to database.", { theme: { '--toastBackground': 'green' } })
				datatable.fetchData()
			}
		} catch(e) {
			dbConnecting = false
			dbConnected = false
			toast.push("Failed to connect to database.", { theme: { '--toastBackground': 'red' } })
		}
	}

	onMount(() => { dbConnect() })
</script>

<main>
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
	<section class="form sticky-top">
		<Form bind:this={form} on:save={() => { datatable.fetchData() }} />
	</section>
	<hr>
	<section class="datatable">
		<Datatable bind:this={datatable} on:edit={(e) => { form.edit(e.detail); animateScroll.scrollToTop() }} />
	</section>
</main>

<style type="text/scss">
	:root {
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
			Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
		--toastContainerTop: auto;
		--toastContainerRight: auto;
		--toastContainerBottom: 5rem;
		--toastContainerLeft: calc(50vw - 8rem);
	}

	section {
		&.connection {
			background-color: rgba(255, 173, 173, 0.7);
			vertical-align: middle;
		}
		&.datatable {
			min-height: 4em;
			margin-bottom: 3em;
		}
	}
</style>

<SvelteToast options={{ duration: 2000, reversed: true, intro: { y: 192 } }} />
