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
				<h1>{tag}</h1>
				<ol start={(index > 0 ? pastLength(index) : 0) + 1}>
					{#each rows[tag] as row}
						<li>
							<p>
								{#each [makeTextObject(row.tsu), makeTextObject(row.tsa)] as parts}
									{#each parts as span}
										{#if span.mark == true}
											<u>{span.text}</u>
										{:else}
											{span.text}
										{/if}
									{/each}
									<br>
								{/each}
							</p>
							<p class="text-end">({title}, jilid {row.volume} halaman {simplifyPage(row.page)})</p>
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
