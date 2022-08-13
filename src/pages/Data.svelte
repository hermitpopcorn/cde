<script lang="ts">
	import Datatable from '../lib/Datatable.svelte'
	import Form from '../lib/Form.svelte'
	import { refreshDatatable } from '../lib/EventBus'

	// Component binding
	let form
	let datatable	

	// Variables
	let stickyForm = false

	// Listen to EventBus
	refreshDatatable.subscribe(_ => {
		if (!datatable) { return }
		datatable.fetchData()
	})
</script>

<main>
	<section class={stickyForm ? "form sticky-top" : "form"}>
		<Form bind:this={form} on:save={() => { datatable.fetchData() }} on:toggle-sticky={() => { stickyForm = !stickyForm }} sticky={stickyForm} />
	</section>
	<hr>
	<section class="datatable">
		<Datatable bind:this={datatable} on:edit={(e) => { form.edit(e.detail) }} />
	</section>
</main>

<style lang="scss">
	section.datatable {
		min-height: 4em;
		margin-bottom: 3em;
	}
</style>
