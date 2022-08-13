<script lang="ts">
	import { onMount } from 'svelte'
	import { invoke } from '@tauri-apps/api/tauri'
	import { link, replace } from 'svelte-spa-router'

	let statistics = []

	export const fetchData = async () => {
		statistics = await invoke("get_count_by_columns", { columns: ["type", "note"] })
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
	<table class="table w-100">
		<thead>
			<tr>
				<th class="numbering">
					No
				</th>
				<th class="type-note">
					Type/Note
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
