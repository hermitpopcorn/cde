<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import Datatable from '../lib/Datatable.svelte'
	import Form from '../lib/Form.svelte'
	import { refreshDatatable } from '../lib/EventBus'

	// Component binding
	let form
	let datatable	

	// Variables
	let stickyForm = JSON.parse(localStorage.getItem('stickyForm')) ?? false
	const toggleStickyForm = () => { stickyForm = !stickyForm; localStorage.setItem('stickyForm', stickyForm) }

	// Listen to EventBus
	refreshDatatable.subscribe(_ => {
		if (!datatable) { return }
		datatable.fetchData()
	})

	const handlePageHotkeys = (e) => {
		if (!(e instanceof KeyboardEvent)) { return }
		
		var kbdE: KeyboardEvent = e
		if (kbdE.ctrlKey && kbdE.shiftKey && kbdE.key === "ArrowRight") {
			return datatable.lastPage()
		}
		if (kbdE.ctrlKey && kbdE.shiftKey && kbdE.key === "ArrowLeft") {
			return datatable.firstPage()
		}
		if (kbdE.ctrlKey && kbdE.key === "ArrowRight") {
			return datatable.nextPage()
		}
		if (kbdE.ctrlKey && kbdE.key === "ArrowLeft") {
			return datatable.previousPage()
		}
		if (kbdE.ctrlKey && kbdE.key === "ArrowDown") {
			return toggleStickyForm()
		}
	}

	onMount(() => { document.addEventListener('keyup', handlePageHotkeys) })
	onDestroy(() => { document.removeEventListener('keyup', handlePageHotkeys) })
</script>

<main>
	<section class={stickyForm ? "form sticky-top" : "form"}>
		<Form bind:this={form} on:save={() => { datatable.fetchData() }} on:toggle-sticky={toggleStickyForm} sticky={stickyForm} />
	</section>
	<hr>
	<section class="datatable">
		<Datatable bind:this={datatable} on:edit={(e) => { form.edit(e.detail) }} />
	</section>
</main>

<style lang="scss">
	main {
		min-width: 640px;
	}

	section.datatable {
		min-height: 4em;
	}
</style>
