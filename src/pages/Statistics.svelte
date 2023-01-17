<script lang="ts">
	import { onMount } from 'svelte'
	import { invoke } from '@tauri-apps/api/tauri'
	import { link } from 'svelte-spa-router'
	import { PlusIcon, XIcon } from 'svelte-feather-icons'

	let statistics = []
	let columns = ['type', 'tags']
	let limitTags: Array<string> = ['subjek', 'predikat', 'objek', 'pelengkap', 'keterangan']

	export const fetchData = async () => {
		columns = columns.filter(i => i)
		statistics = await invoke("get_count_by_columns", { columns }) ?? []

		let conditionIndex = columns.indexOf('unwind:tags')
		if (conditionIndex > -1) {
			statistics = statistics.filter((row) => {
				return limitTags.includes(row.columns[conditionIndex][1])
			})
		}
	}

	const formatColumnsMap = i => {
		if (i[1] === 'penambahan') {
			return '+'
		} else if (i[1] === 'pengurangan') {
			return '-'
		}
		return i[1]
	}
	const determineRowType = i => {
		for (let column of i) {
			if (column[0] !== 'type') { continue }
			if (column[1] === 'penambahan') {
				return 'amplification';
			}
			if (column[1] === 'pengurangan') {
				return 'reduction'
			}
		}

		return 'neutral';
	}

	onMount(() => {
		if (statistics.length < 1) {
			fetchData()
		}
	})
</script>

<main>
	<section class="p-4">
		<div class="d-grid gap-2 mb-2">
			<label for="column[0]">Group</label>
			{#each columns as column, index}
				<div class="input-group">
					<select class="form-select" name={`column[${index}]`} bind:value={column}>
						<option value="type">Type</option>
						<option value="tags">Tags</option>
						<option value="unwind:tags">Tags (individually)</option>
						<option value="volume">Volume</option>
						<option value="page">Page</option>
					</select>
					<button class="btn btn-icon btn-danger" type="button" on:click={() => { columns = columns.filter((_, i) => i !== index); }}><XIcon size="1.2x" /></button>
				</div>
			{/each}
		</div>
		<div class="d-flex">
			<button title="Add new column" type="button" class="btn btn-icon btn-sm btn-primary" on:click={() => { columns = [...columns, ""] }}><PlusIcon size="1.2x" /></button>
		</div>
		{#if columns.includes('unwind:tags') }
			<div class="d-grid gap-2 mb-2">
				<label for="tag[0]">Limit individual tags to</label>
				{#each limitTags as tag, index}
					<div class="input-group">
						<input class="form-control" type="text" name={`tag[${index}]`} bind:value={tag}>
						<button class="btn btn-icon btn-danger" type="button" on:click={() => { limitTags = limitTags.filter((_, i) => i !== index); }}><XIcon size="1.2x" /></button>
					</div>
				{/each}
			</div>
			<div class="d-flex">
				<button title="Add new column" type="button" class="btn btn-icon btn-sm btn-primary" on:click={() => { columns = [...columns, ""] }}><PlusIcon size="1.2x" /></button>
			</div>
		{/if}
		<div class="d-flex justify-content-end mt-4">
			<button title="Load" class="btn btn-icon btn-primary" on:click={fetchData}>Load</button>
		</div>
	</section>
	<table class="table w-100">
		<thead>
			<tr>
				<th class="numbering">
					No
				</th>
				<th class="type-note">
					Aggregate
				</th>
				<th class="quantity">
					Quantity
				</th>
			</tr>
		</thead>
		<tbody>
			{#if statistics}
				{#each statistics as row, index}
					<tr class={determineRowType(row.columns)}>
						<td class="numbering">{index + 1}</td>
						<td>
							<a href="/data?{row.columns.map(i => `${i[0]}=${i[1]}`).join('&')}" use:link>
								{row.columns.map(formatColumnsMap).join(' ')}
							</a>
						</td>
						<td>{row.quantity}</td>
					</tr>
				{/each}
			{/if}
		</tbody>
	</table>
</main>

<style lang="scss">
	th, td {
		text-align: center;
	}
	th {
		vertical-align: middle;

		&.numbering { width: 10%; }
		&.type-note { width: 80%; }
		&.quantity { width: 10%; }
	}
	tr {
		&.amplification { background: #e7f1ff; }
		&.reduction { background: #fde9e9; }
	}
	a {
		color: inherit;
		text-decoration: none;
		&:hover {
			color: blue;
		}
	}
</style>
