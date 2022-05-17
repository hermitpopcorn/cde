<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
	import { onMount } from 'svelte'
	import { SvelteToast, toast } from '@zerodevx/svelte-toast'
	import * as animateScroll from "svelte-scrollto";
	import Datatable from './lib/Datatable.svelte'
	import Form from './lib/Form.svelte'

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
			await invoke("db_connect")
			dbConnecting = false
			dbConnected = true
			toast.push("Successfully connected to database.", { theme: { '--toastBackground': 'green' } })
		} catch(e) {
			dbConnecting = false
			dbConnected = false
			toast.push("Successfully connected to database.", { theme: { '--toastBackground': 'red' } })
		}
	}
	
	// Component binding
	let form
	let datatable

	onMount(() => { dbConnect() })
</script>

<svelte:head>
	<link
		rel="stylesheet"
		href="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css"
	/>
</svelte:head>

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
	<section class="form">
		<Form bind:this={form} on:save={() => { datatable.fetchData() }} />
	</section>
	<hr>
	<section class="datatable">
		<Datatable bind:this={datatable} on:edit={(e) => { form.edit(e.detail); animateScroll.scrollToTop() }} />
	</section>
</main>

<style>
	:root {
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
			Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
	}

	section.connection {
		background-color: rgba(255, 173, 173, 0.7);
		vertical-align: middle;
	}

	section.datatable {
		min-height: 4em;
		margin-bottom: 3em;
	}
</style>

<SvelteToast options={{ duration: 3000 }} />
