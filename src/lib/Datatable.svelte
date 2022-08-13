<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
	import { toast } from '@zerodevx/svelte-toast'
	import { EditIcon, Trash2Icon, CheckSquareIcon, StarIcon } from 'svelte-feather-icons'
	import { onMount, onDestroy } from 'svelte'
	import { querystring } from 'svelte-spa-router'
	import { createEventDispatcher } from 'svelte'

	const dispatch = createEventDispatcher()

	let thead

	let data = []
	let pageSize = 20
	let dataCount = 0
	let currentPage = 1
	let filters = {
		volume: "",
		page: "",
		type: "",
		note: "",
		text: "",
	}
	let appliedFilters = { ... filters }

	$: { pageSize, fetchData() }

	// refresh the data by requesting it anew
	export const fetchData = async () => {
		let fetchResult = await invoke("get_documents", { page: currentPage, size: pageSize, filters: filters })
		data = fetchResult[0]
		currentPage = fetchResult[1]
		dataCount = fetchResult[2]
	}

	// change page and request data
	const setPage = async (to: number) => {
		if (to < 1 || to > Math.ceil(dataCount / pageSize)) { return }
		currentPage = to
		await fetchData()
	}

	// filter data after a set timeout
	let timeout: NodeJS.Timeout
	const startFilterData = (event: Event & { currentTarget: EventTarget }, property: string) => {
		// if filters are exactly the same as the one active right now, do not request data
		if (filters[property] === appliedFilters[property]) { return }
		
		// wait until user stops typing before requesting data
		clearTimeout(timeout)
		timeout = setTimeout(filterData, KeyboardEvent.prototype.isPrototypeOf(event) ? 500 : 10)
	}
	const filterData = async () => {
		await fetchData()
		appliedFilters = { ... filters }
	}

	// format data text (string manipulation)
	const formatDataText = (tsu, tsa) => {
		const makeTextObject = (string) => {
			const isInBrackets = (str: string): boolean => str.startsWith('[') && str.endsWith(']')

			let finalArray = []
			string = string.match(/([^[\]]+|(\[(.[^\]]+)\]))/g).map( function(val) { return val==="[]" ? null : val })
			for (let i = 0; i < string.length; i++) {
				finalArray.push({
					text: isInBrackets(string[i]) ? string[i].substr(1, string[i].length - 2) : string[i],
					mark: isInBrackets(string[i]),
				})
			}
			return finalArray
		}

		let array = []
		if (tsu) {
			array.push({ type: 'tsu', texts: makeTextObject(tsu) })
		}
		if (tsa) {
			array.push({ type: 'tsa', texts: makeTextObject(tsa) })
		}
		
		return array
	}

	const edit = (index) => {
		let row = data[index]
		dispatch('edit', row)
	}

	const remove = async (index) => {
		let row = data[index]
		let c = await window.confirm("Are you sure?")
		if (c) {
			await invoke("remove_document", { id: row._id.$oid })
			toast.push("Data has been deleted.", { theme: { '--toastBackground': 'green' } })
			fetchData()
		}
	}

	const star = async (index) => {
		let c: boolean = await invoke("star_document", { id: data[index]._id.$oid })
		if (c) {
			data[index].starred = true
		} else {
			data[index].starred = false
		}
	}
	onMount(() => {
		if (data.length < 1) {
			fetchData()
		}
	})

	const qsUnsubscriber = querystring.subscribe(qs => {
		let filtered = false

		const pairs = qs.split('&')
		for (let i = 0; i < pairs.length; i++) {
			const pair = pairs[i].split('=')
			const key = decodeURIComponent(pair[0])
			const value = decodeURIComponent(pair[1] || '')
			if (key in filters) {
				filters[key] = value
				filtered = true
			}
		}

		if (filtered) {
			setTimeout(() => thead.scrollIntoView({ behavior: 'smooth' }), 500)
		}
	})
	onDestroy(() => { qsUnsubscriber() })
</script>

<div class="row g-3 align-items-center px-4 justify-content-end">
	<div class="col-auto">
	  <label class="col-form-label" for="page-size-control">Items per page</label>
	</div>
	<div class="col-auto">
	  <select class="form-control" id="page-size-control" bind:value={pageSize}>
		<option value={10}>10</option>
		<option value={20}>20</option>
		<option value={50}>50</option>
		<option value={100}>100</option>
	</select>
	</div>
</div>

<table class="table w-100">
	<thead bind:this={thead}>
		<tr>
			<th class="numbering">
				No
			</th>
			<th class="volume">
				Vol
				<input type="text" class="form-control" bind:value={filters.volume} on:keyup={(e) => { startFilterData(e, "volume") }}>
			</th>
			<th class="page">
				Page
				<input type="text" class="form-control" bind:value={filters.page} on:keyup={(e) => { startFilterData(e, "page") }}>
			</th>
			<th class="type">
				Type
				<select class="form-control" bind:value={filters.type} on:change={(e) => { startFilterData(e, "type") }}>
					<option value=""></option>
					<option value="penambahan">A+</option>
					<option value="pengurangan">R-</option>
				</select>
			</th>
			<th class="note">
				Note
				<input type="text" class="form-control" bind:value={filters.note} on:keyup={(e) => { startFilterData(e, "note") }}>
			</th>
			<th class="text">
				Text
				<input type="text" class="form-control" bind:value={filters.text} on:keyup={(e) => { startFilterData(e, "text") }}>
			</th>
			<th class="text">
				Cause
			</th>
			<th class="text">
				Effect
			</th>
			<th class="action">
				Edit
			</th>
		</tr>
	</thead>
	<tbody>
	{#if data}
		{#each data as row, index}
			<tr class={{'penambahan': 'amplification', 'pengurangan': 'reduction'}[row.type]}>
				<td class="numbering">
					{((currentPage - 1) * pageSize) + (index + 1)}
				</td>
				<td>{row.volume ?? ""}</td>
				<td>{row.page ?? ""}</td>
				<td>{{'penambahan': 'A+', 'pengurangan': 'R-'}[row.type]}</td>
				<td>{row.note ?? ""}</td>
				<td>
					{#each formatDataText(row.tsu, row.tsa) as p}
						<p class={p.type}>
							{#each p.texts as span}
								{#if span.mark == true}
									<mark>{span.text}</mark>
								{:else}
									{span.text}
								{/if}
							{/each}
						</p>
					{/each}
				</td>
				<td>
					{#if row.cause}
						<CheckSquareIcon class="text-success" />
					{/if}
				</td>
				<td>
					{#if row.effects?.length > 0}
						<CheckSquareIcon class="text-success" /><br>
						({row.effects.length})
					{/if}
				</td>
				<td class="actions">
					<button class="btn btn-icon btn-primary btn-sm" on:click={() => edit(index)}><EditIcon size="1.2x" /></button>
					<button class="btn btn-icon btn-danger btn-sm" on:click={() => remove(index)}><Trash2Icon size="1.2x" /></button>
					<button class="btn btn-icon btn-dark btn-sm" class:btn-warning={ "starred" in row } class:btn-dark={ !("starred" in row) || row.starred == false } on:click={() => star(index)}><StarIcon size="1.2x" /></button>
				</td>
			</tr>
		{/each}
	{/if}
	</tbody>
</table>

<hr id="finish" />

<section class="pagination-container fixed-bottom">
	<nav aria-label="Datatable pagination">
		<div class="d-inline">
			<ul class="pagination justify-content-center">
				<li class="page-item">
					<button class="page-link" aria-label="Previous" on:click={() => { setPage(currentPage - 1) }}>
						<span aria-hidden="true">&laquo;</span>
					</button>
				</li>
				{#each {length: Math.ceil(dataCount / pageSize)} as _, i}
				<li class={i+1 === currentPage ? "page-item active" : "page-item"}>
					<button class="page-link" on:click={() => { setPage(i+1) }}>{i + 1}</button>
				</li>
				{/each}
				<li class="page-item">
					<button class="page-link" aria-label="Next" on:click={() => { setPage(currentPage + 1) }}>
						<span aria-hidden="true">&raquo;</span>
					</button>
				</li>
			</ul>
		</div>
		<ul class="pagination justify-content-center">
			<li class="page-item active">
				<button class="page-link" aria-label="Refresh" on:click={() => { fetchData() }}>
					<span aria-hidden="true">⟳</span>
				</button>
			</li>
		</ul>
	</nav>
</section>

<style lang="scss">
	th, td {
		text-align: center;
	}
	p {
		margin: 0.2em;
	}
	table {
		margin-bottom: 5em;
	}
	th {
		vertical-align: middle;

		&.numbering { width: 2%; }
		&.volume { width: 5%; }
		&.page { width: 5%; }
		&.type { width: 10%; }
		&.note { width: 20%; }
		&.action { width: 5%; }
	}
	tr {
		&.amplification { background: #e7f1ff; }
		&.reduction { background: #fde9e9; }
	}
	td.actions button {
		margin: 0.1em;
	}
	input, select {
		padding: 0.2em;
		font-size: 0.8em;
	}
	select {
		text-align: center;
		appearance: revert;
		-moz-appearance: revert;
		-webkit-appearance: revert;
	}
	.pagination-container {
		padding: 1em 1em;
		text-align: center;
		background-color: rgba(0, 0, 0, 0.2);
	}
	ul.pagination {
		display: inline-flex;
		flex-wrap: wrap;
		margin: 0;
	}
</style>
