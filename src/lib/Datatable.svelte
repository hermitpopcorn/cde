<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
import { toast } from '@zerodevx/svelte-toast'
	import { onMount } from 'svelte'
	import { createEventDispatcher } from 'svelte'
	const dispatch = createEventDispatcher()

	let data = []
	let pageSize = 20
	let currentPage = 1
	let filters = {
		volume: "",
		page: "",
		type: "",
		note: "",
		text: "",
	}
	let appliedFilters = { ... filters }
	let filteredData = []

	$: rows = filteredData.slice((currentPage -1) * pageSize, currentPage * pageSize)
	$: currentPage = Math.min(Math.max(Math.ceil(filteredData.length / pageSize), 1), currentPage)

	export const fetchData = async () => {
		data = await invoke('get_all_documents')
		let index = 1
		for (let i in data) {
			data[i].index = index++
		}
		filterData()
	}

	const setPage = (to: number) => {
		if (to < 1 || to > Math.ceil(filteredData.length / pageSize)) { return }
		currentPage = to
	}

	let timeout: NodeJS.Timeout
	const startFilterData = (event: Event & { currentTarget: EventTarget }, property: string) => {
		if (filters[property] === appliedFilters[property]) { return }
		clearTimeout(timeout)
		timeout = setTimeout(filterData, KeyboardEvent.prototype.isPrototypeOf(event) ? 500 : 10)
	}
	const filterData = () => {
		filteredData = []
		
		let activeFilters = []
		for (let property of Object.keys(filters)) {
			if (filters[property].length < 1) { continue }
			activeFilters.push(property)
		}

		if (activeFilters.length < 1) {
			filteredData = data
			return
		}

		let filteredDataPerProperty = []
		for (let property of activeFilters) {
			filteredDataPerProperty[property] = [...filteredData, ...data.filter((element) => {
				let filter = (filters[property] as string).toLowerCase()

				if (property === "text") {
					return (element['tsu'] as string ?? "").toLowerCase().includes(filter) || (element['tsa'] as string ?? "").toLowerCase().includes(filter)
				}
				
				let check: string = String(element[property] ?? "").toLowerCase()
				return check.includes(filter)
			})]
		}

		filteredData = data.filter((item) => {
			for (let p of Object.keys(filteredDataPerProperty)) {
				if (!filteredDataPerProperty[p].includes(item)) {
					return false
				}
			}
			return true
		})

		appliedFilters = { ... filters }
	}

	const formatDataText = (tsu, tsa) => {
		const makeTextObject = (string) => {
			let finalArray = []
			string = string.match(/([^[\]]+|\[\])/g).map( function(val) { return val==="[]" ? null : val })
			for (let i = 0; i < string.length; i++) {
				finalArray.push({
					text: string[i],
					mark: (i % 2 == 1),
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

	const edit = (row) => {
		dispatch('edit', row)
	}

	const remove = async (row) => {
		let c = await window.confirm("Are you sure?")
		if (c) {
			await invoke("remove_document", { id: row._id.$oid })
			toast.push("Data has been deleted.", { theme: { '--toastBackground': 'green' } })
			fetchData()
		}
	}
</script>

<div class="row g-3 align-items-center px-4 justify-content-end">
	<div class="col-auto">
	  <label class="col-form-label" for="page-size-control">Items per page</label>
	</div>
	<div class="col-auto">
	  <select class="form-control" id="page-size-control" bind:value={pageSize}>
		<option value="20">20</option>
		<option value="50">50</option>
		<option value="100">100</option>
	</select>
	</div>
</div>

<table class="table w-100">
	<thead>
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
			<th class="action">
				Edit
			</th>
		</tr>
	</thead>
	<tbody>
	{#if rows}
		{#each rows as row, index}
			<tr class={{'penambahan': 'amplification', 'pengurangan': 'reduction'}[row.type]}>
				<td class="numbering">
					{((currentPage - 1) * pageSize) + (index + 1)}
					<br>
					<span class="id">ID{row.index}</span>
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
				<td class="actions">
					<button class="btn btn-primary btn-sm" on:click={() => edit(row)}>Edit</button>
					<button class="btn btn-danger btn-sm" on:click={() => remove(row)}>Delete</button>
				</td>
			</tr>
		{/each}
	{/if}
	</tbody>
</table>

<section class="pagination-container fixed-bottom">
	<nav aria-label="Datatable pagination">
		<ul class="pagination justify-content-center">
			<li class="page-item">
				<button class="page-link" aria-label="Previous" on:click={() => { setPage(currentPage - 1) }}>
					<span aria-hidden="true">&laquo;</span>
				</button>
			</li>
			{#each {length: Math.ceil(filteredData.length / pageSize)} as _, i}
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
		<ul class="pagination justify-content-center">
			<li class="page-item active">
				<button class="page-link" aria-label="Refresh" on:click={() => { fetchData() }}>
					<span aria-hidden="true">⟳</span>
				</button>
			</li>
		</ul>
	</nav>
</section>

<style type="text/scss">
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
	td.numbering span.id {
		font-size: 0.7em;
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
		margin: 0;
	}
</style>
