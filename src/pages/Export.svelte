<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
	import { PlusIcon, XIcon } from 'svelte-feather-icons'
	import { makeTextObject } from '../lib/Helper'

	let title = localStorage.getItem('export-title') ?? "TITLE"
	
	let tags: Array<string> = ['subjek', 'predikat', 'objek', 'pelengkap', 'keterangan']
	let rows: Array<Array<any>> = []
	let rows_tags: Array<string> = [] // Same as tags but doesn't change with user input

	const fetchData = async () => {
		tags = tags.filter(i => i)
		if (tags) {
			rows = await invoke("get_documents_by_tags", { tags }) ?? []
			rows_tags = tags;
		}
		localStorage.setItem('export-title', title)
	}

	const simplifyPage = (page: string) => {
		return page.split(".")[0]
	}

	const pastLength = (index: number) => {
		let len = 0
		for (let i = 0; i < index; i++) {
			len += rows[rows_tags[i]].length
		}
		return len
	}

	const penerjemah = (volume: number) => {
		if (volume == 1) { return 'Frisian Yuniardi'; }
		if (volume >= 2 && volume <= 8) { return 'Indra Nugraha'; }
		if (volume >= 9 && volume <= 12) { return 'Evelyn Limanjaya'; }
		if (volume >= 13 && volume <= 14) { return 'Olive Irianto'; }
	}

	const year = (volume: number, index: number) => {
		if (index == 0) {
			return {
				1: 2013,
				2: 2014,
				3: 2014,
				4: 2014,
				5: 2015,
				6: 2015,
				7: 2015,
				8: 2016,
				9: 2016,
				10: 2016,
				11: 2017,
				12: 2017,
				13: 2018,
				14: 2018,
			}[volume]
		} else {
			return {
				1: 2016,
				2: 2016,
				3: 2016,
				4: 2016,
				5: 2017,
				6: 2017,
				7: 2017,
				8: 2017,
				9: 2019,
				10: 2019,
				11: 2019,
				12: 2019,
				13: 2019,
				14: 2020,
			}[volume]
		}
	}
</script>

<main>
	<section class="p-4 border-bottom">
		<div class="mb-2">
			<div class="form-group">
				<label for="input-title">Title</label>
				<input class="form-control" name="input-title" type="text" bind:value={title}>
			</div>
		</div>
		<div class="d-grid gap-2 mb-2">
			<label for="tag[0]">Tags</label>
			{#each tags as tag, index}
				<div class="input-group">
					<input class="form-control" type="text" name={`tag[${index}]`} bind:value={tag}>
					<button class="btn btn-icon btn-danger" type="button" on:click={() => { tags = tags.filter((_, i) => i !== index); }}><XIcon size="1.2x" /></button>
				</div>
			{/each}
		</div>
		<div class="d-flex justify-content-between">
			<button title="Add new column" type="button" class="btn btn-icon btn-sm btn-primary" on:click={() => { tags = [...tags, ""] }}><PlusIcon size="1.2x" /></button>
			<button title="Load" class="btn btn-icon btn-primary" on:click={fetchData}>Load</button>
		</div>
	</section>
	
	<section id="list">
		{#if rows}
			{#each rows_tags as tag, index}
				<ol start={(index > 0 ? pastLength(index) : 0) + 1}>
					{#each rows[tag] as row}
						<li>
							{#each [makeTextObject(row.tsu), makeTextObject(row.tsa)] as parts, index}
								<p>{#each parts as span}
									{#if span.mark == true}
										<u>{span.text}</u>
									{:else}
										{span.text}
									{/if}
								{/each}</p>
								<p class="text-end">({#if index == 0}Matsuura Daruma{:else}{penerjemah(row.volume)}{/if}, {year(row.volume, index)}: {title} jilid {row.volume} halaman {simplifyPage(row.page)})</p>
							{/each}
						</li>
					{/each}
				</ol>
			{/each}
		{/if}
	</section>
</main>

<style lang="scss">
	section#list {
		padding: 1em 2em;
		h1 {
			text-transform: capitalize;
		}
	}
</style>
