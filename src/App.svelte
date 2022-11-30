<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
	import { onMount } from 'svelte'
	import { SvelteToast, toast } from '@zerodevx/svelte-toast'
	import Router, { replace } from 'svelte-spa-router'
	import { listen } from '@tauri-apps/api/event'
	import Data from './pages/Data.svelte'
	import Statistics from './pages/Statistics.svelte'
	import Export from './pages/Export.svelte'
	import { databaseConnectionStatus, refreshDatatable } from './lib/EventBus'
	import { PlayIcon } from 'svelte-feather-icons'
	
	// Custom types
	type ChangePagePayload = {
		page: string,
	}

	// Routes
	const routes = {
		'/': Data, // Default route
		'/data': Data,
		'/statistics': Statistics,
		'/export': Export,
	}

	// Database
	// Form
	let parsedLS = JSON.parse(localStorage.getItem('connection-parameters'))
	let connectionParams = {
		connectionString: parsedLS?.connectionString ?? "",
		databaseName: parsedLS?.databaseName ?? "",
		collectionName: parsedLS?.collectionName ?? "",
	}	

	// Connection
	let dbConnected = false
	let dbConnecting = false
	const dbConnect = async () => {
		if (dbConnecting) { return }
		localStorage.setItem('connection-parameters', JSON.stringify(connectionParams))
		dbConnecting = true
		try {
			let params = {
				connectionString: connectionParams.connectionString || "",
				databaseName: connectionParams.databaseName || "",
				collectionName: connectionParams.collectionName || "",
			}
			let connect: boolean = await invoke('db_connect', params)
			dbConnecting = false
			dbConnected = true
			if (connect) {
				toast.push("Successfully connected to database.", { theme: { '--toastBackground': 'green' } })
				databaseConnectionStatus.update(_value => true)
				refreshDatatable.update(n => n + 1)
			}
		} catch(e) {
			dbConnecting = false
			dbConnected = false
			let message = (e as string).split(": ").pop()
			toast.push(message, { theme: { '--toastBackground': 'red' } })
		}
	}

	// Remove preloader and connect to DB
	onMount(() => {
		document.getElementById('app-preloader')?.remove()
	})

	// Listen to Tauri events
	listen('change_page', (event) => {
		replace(`/${(<ChangePagePayload>event.payload).page}`)
	})
</script>

<section>
	<section class="connection">
		<div class="container p-2">
			<form class="mb-2">
				<div class="row gx-1">
					<div class="col-7">
						<input class="form-control" type="text" bind:value={connectionParams.connectionString} placeholder="MongoDB connection string">
					</div>
					<div class="col-2">
						<input class="form-control" type="text" bind:value={connectionParams.databaseName} placeholder="Database">
					</div>
					<div class="col-2">
						<input class="form-control" type="text" bind:value={connectionParams.collectionName} placeholder="Collection">
					</div>
					<div class="col-1">
						<button title="Connect" class="{['btn btn-icon', dbConnected ? 'btn-success' : 'btn-primary'].join(' ')}" type="button" on:click={() => { dbConnect() }}><PlayIcon size="1.2x" /></button>
					</div>
				</div>
			</form>
		</div>
	</section>

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
		vertical-align: sub;
	}

	section.connection {
		background-color: rgba(188,200,224,1);
		vertical-align: middle;
	}
</style>
