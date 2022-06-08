<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
	import { toast } from '@zerodevx/svelte-toast'
	import { CornerRightDownIcon, LoaderIcon, SaveIcon, PlusIcon, XIcon, AnchorIcon } from 'svelte-feather-icons'
	import { createEventDispatcher } from 'svelte'
	const dispatch = createEventDispatcher()
	
	let form
	export let sticky = false

	const save = async () => {
		const texts = form.text.split("\n")
		const tsu = texts[0] || null
		const tsa = texts[1] || null

		try {
			await invoke('save_document', {
				id: form.id,
				tsu: tsu,
				tsa: tsa,
				theType: form.type,
				note: form.note.length > 0 ? form.note : null,
				volume: form.volume.length > 0 ? form.volume : null,
				page: form.page.length > 0 ? form.page : null,
				cause: form.cause.length > 0 ? form.cause : null,
				effects: form.effects.filter((i) => i.length > 0),
			})
			toast.push("Data saved successfully.", { theme: { '--toastBackground': 'green' } })
			dispatch('save')
		} catch(e) {
			let message = (e as string).split(": ").pop()
			toast.push(`Save failed: ${message}`, { theme: { '--toastBackground': 'red' }, duration: 5000 })
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
			cause: "",
			effects: [],
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
		form.cause = row.cause ?? ""
		form.effects = row.effects ?? []
	}

	const handleInputKeyup = (e: Event) => {
		if (!(e instanceof KeyboardEvent)) { return }
		
		var kbdE: KeyboardEvent = e;
		if (kbdE.ctrlKey && kbdE.key === "Enter") {
			return save()
		}
		if (kbdE.ctrlKey && kbdE.key === "n") {
			return clear()
		}
		if (kbdE.ctrlKey && kbdE.key === "d") {
			return clearID()
		}
	}

	const toggleSticky = () => {
		dispatch('toggle-sticky')
	}

	clear()
</script>

<div class="accordion mb-2" id="accordion">
	<div class="accordion-item">
		<div id="form-collapsible" class="accordion-collapse collapse show" aria-labelledby="form-heading" data-bs-parent="#accordion">
			<div class="accordion-body">
				<form id="input-form" on:keyup={(e) => { handleInputKeyup(e) }}>
					<div class="row">
						<div class="col-lg-6">
							<div class="row align-items-center">
								<div class="col">
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
								<div class="col">
									<div class="form-group">
										<label for="input-volume">Volume</label>
										<input type="text" class="form-control" id="input-volume" name="volume" bind:value={form.volume}>
									</div>
								</div>
								<div class="col">
									<div class="form-group">
										<label for="input-page">Page</label>
										<input type="text" class="form-control" id="input-page" name="page" bind:value={form.page}>
									</div>
								</div>
								<div class="col-6">
									<div class="form-group">
										<label for="input-note">Note</label>
										<input type="text" class="form-control" id="input-note" name="note" bind:value={form.note}>
									</div>
								</div>
							</div>
							<div class="row">
								<div class="col">
									<div class="form-group">
										<label for="input-text">Text</label>
										<textarea class="form-control" id="input-text" name="text" rows="2" bind:value={form.text}></textarea>
									</div>
								</div>
							</div>
						</div>
						<div class="col-lg-6">
							<div class="row h-100">
								<div class="col">
									<div class="form-group h-100 d-flex flex-column">
										<label for="input-text">Cause</label>
										<textarea class="form-control flex-fill" id="input-cause" name="cause" rows="2" bind:value={form.cause}></textarea>
									</div>
								</div>
								<div class="col">
									<div class="form-group">
										<label for="input-text">Effects</label>
										<div class="d-grid gap-2">
											{#each form.effects as effect, index}
												<div class="input-group">
													<input type="text" class="form-control" name={`effect[${index}]`} bind:value={effect}>
													<button class="btn btn-icon btn-danger" type="button" on:click={() => { form.effects = form.effects.filter((_, i) => i !== index); }}><XIcon size="1.2x" /></button>
												</div>
											{/each}
											<button type="button" class="btn btn-icon btn-sm btn-primary" on:click={() => { form.effects = [...form.effects, ""] }}><PlusIcon size="1.2x" /></button>
										</div>
									</div>
								</div>
							</div>
						</div>
					</div>
					<div class="d-flex justify-content-between mt-2">
						<div class="flex-grow-1">
							<div class="d-flex gap-2">
								<div class="flex-grow-1">
									<div class="form-group">
										<input type="text" class="form-control" id="input-id" name="_id" value={displayID} readonly>
									</div>
								</div>
								<div class="flex-shrink-1">
									<div class="form-group">
										<button type="button" class="btn btn-icon btn-secondary" id="input-new-id" on:click={() => { clearID() }}><CornerRightDownIcon size="1.2x" /></button>
										<button type="button" class="btn btn-icon btn-secondary" id="input-clear" on:click={() => { clear() }}><LoaderIcon size="1.2x" /></button>
									</div>
								</div>
							</div>
						</div>
						<div class="col">
							<div class="d-flex justify-content-end align-items-center gap-3">
								<span id="process-info"></span>
								<div class="form-group text-end">
									<button class={sticky ? "btn btn-primary" : "btn"} type="button" on:click={() => { toggleSticky() }}><AnchorIcon size="1.2x" /></button>
									<button class="btn btn-primary" type="button" on:click={() => { save() }}><SaveIcon size="1.2x" /></button>
								</div>
							</div>
						</div>
					</div>
				</form>
			</div>
		</div>
	</div>
</div>
