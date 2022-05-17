<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
	import { toast } from '@zerodevx/svelte-toast'
	import { createEventDispatcher } from 'svelte'
	const dispatch = createEventDispatcher()
	
	let form

	const save = async () => {
		const texts = form.text.split("\n")
		const tsu = texts[0] || null
		const tsa = texts[1] || null

		let save = await invoke('save_document', {
			id: form.id,
			tsu: tsu,
			tsa: tsa,
			theType: form.type,
			note: form.note.length > 0 ? form.note : null,
			volume: form.volume.length > 0 ? form.volume : null,
			page: form.page.length > 0 ? form.page : null,
		})
		if (save) {
			toast.push("Data saved successfully.", { theme: { '--toastBackground': 'green' } })
			dispatch('save')
		} else {
			toast.push("Failed to save data.", { theme: { '--toastBackground': 'red' } })
		}
	}
	const clear = () => {
		form = {
			index: null,
			id: null,
			type: "penambahan",
			note: "",
			volume: "",
			page: "",
			text: "",
		}
	}

	const clearID = () => {
		form.index = null
		form.id = null
	}
	$: displayID = form.id ? `${form.index ? `(${form.index}) ` : ""}${form.id}` : ""

	export const edit = (row) => {
		form.index = row.index
		form.id = row._id.$oid
		form.type = row.type
		form.note = row.note ?? ""
		form.volume = row.volume ?? ""
		form.page = row.page ?? ""
		form.text = [row.tsu || "", row.tsa || ""].join("\n")
	}

	clear()
</script>

<div class="accordion mb-2" id="accordion">
	<div class="accordion-item">
		<div id="form-collapsible" class="accordion-collapse collapse show" aria-labelledby="form-heading" data-bs-parent="#accordion">
			<div class="accordion-body">
				<form id="input-form">
					<div class="row" style="align-items: center">
						<div class="col mb-2">
							<div class="form-group">
								<div class="form-check">
									<input class="form-check-input" type="radio" name="type" id="input-type-amplification" value={"penambahan"} bind:group={form.type} checked>
									<label class="form-check-label" for="input-type-amplification">Amplification</label>
								</div>
								<div class="form-check">
									<input class="form-check-input" type="radio" name="type" id="input-type-reduction" value={"pengurangan"} bind:group={form.type}>
									<label class="form-check-label" for="input-type-reduction">Reduction</label>
								</div>
							</div>
						</div>
						<div class="col-6 mb-2">
							<div class="form-group">
								<label for="input-note">Note</label>
								<input type="text" class="form-control" id="input-note" name="note" bind:value={form.note}>
							</div>
						</div>
						<div class="col mb-2">
							<div class="form-group">
								<label for="input-volume">Volume</label>
								<input type="text" class="form-control" id="input-volume" name="volume" bind:value={form.volume}>
							</div>
						</div>
						<div class="col mb-2">
							<div class="form-group">
								<label for="input-page">Page</label>
								<input type="text" class="form-control" id="input-page" name="page" bind:value={form.page}>
							</div>
						</div>
					</div>
					<div class="row">
						<div class="col mb-2">
							<div class="form-group">
								<label for="input-text">Text</label>
								<textarea class="form-control" id="input-text" name="text" rows="2" bind:value={form.text}></textarea>
							</div>
						</div>
					</div>
					<div class="row justify-content-between">
						<div class="col mb-2">
							<div class="d-flex gap-2">
								<div class="flex-grow-1">
									<div class="form-group">
										<input type="text" class="form-control" id="input-id" name="_id" value={displayID} readonly on:click={() => { clearID() }}>
									</div>
								</div>
								<div class="flex-shrink-1">
									<div class="form-group">
										<button type="button" class="btn btn-secondary" id="input-new-id" on:click={() => { clearID() }}>New</button>
										<button type="button" class="btn btn-secondary" id="input-clear" on:click={() => { clear() }}>Clear</button>
									</div>
								</div>
							</div>
						</div>
						<div class="col mb-2">
							<div class="d-flex justify-content-end align-items-center gap-3">
								<span id="process-info"></span>
								<div class="form-group text-end">
									<button class="btn btn-primary" type="button" on:click={() => { save() }}>Save</button>
								</div>
							</div>
						</div>
					</div>
				</form>
			</div>
		</div>
	</div>
</div>
