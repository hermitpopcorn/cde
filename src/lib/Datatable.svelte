<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri'
  import { onMount } from 'svelte'

	let data = []
  let pageSize = 50
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

  const fetchData = async () => {
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

    if (currentPage > Math.ceil(filteredData.length / pageSize)) {
      currentPage = 1
    }

    appliedFilters = { ... filters }
  }

  const formatDataText = (tsu, tsa) => {
    const makeTextObject = (string) => {
      let finalArray = []
      string = string.match(/([^[\]]+|\[\])/g).map( function(val) { return val==="[]" ? null : val; })
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

  onMount(() => {
    fetchData()
  })
</script>

<table class="table w-100">
  <thead>
    <tr>
      <th class="numbering">
        No
      </th>
      <th class="volume">
        Jilid
        <input type="text" class="form-control" bind:value={filters.volume} on:keyup={(e) => { startFilterData(e, "volume") }}>
      </th>
      <th class="page">
        Hal
        <input type="text" class="form-control" bind:value={filters.page} on:keyup={(e) => { startFilterData(e, "page") }}>
      </th>
      <th class="type">
        Tipe
        <select class="form-control" bind:value={filters.type} on:change={(e) => { startFilterData(e, "type") }}>
          <option value=""></option>
          <option value="penambahan">T+</option>
          <option value="pengurangan">K-</option>
        </select>
      </th>
      <th class="note">
        Catatan
        <input type="text" class="form-control" bind:value={filters.note} on:keyup={(e) => { startFilterData(e, "note") }}>
      </th>
      <th class="text">
        Teks
        <input type="text" class="form-control" bind:value={filters.text} on:keyup={(e) => { startFilterData(e, "text") }}>
      </th>
      <th class="action">
        Edit
      </th>
    </tr>
  </thead>
  <tbody>
  {#if rows}
    {#each rows as row}
      <tr>
        <td>{row.index}</td>
        <td>{row.volume ?? ""}</td>
        <td>{row.page ?? ""}</td>
        <td>{{'penambahan': 'T+', 'pengurangan': 'K-'}[row.type]}</td>
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
          <button class="btn btn-primary btn-sm" on:click={() => alert('edit')}>Edit</button>
          <button class="btn btn-danger btn-sm" on:click={() => alert('delete')}>Delete</button>
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


<style>
  th, td {
    text-align: center;
  }
  p {
    margin: 0.2em;
  }
  th {
    vertical-align: middle;
  }
  th.numbering {
    width: 2%;
  }
  th.volume {
    width: 5%;
  }
  th.page {
    width: 5%;
  }
  th.type {
    width: 10%;
  }
  th.note {
    width: 20%;
  }
  th.action {
    width: 5%;
  }
  td.actions button {
    margin: 0.1em;
  }
  select {
    appearance: revert;
    -moz-appearance: revert;
    -webkit-appearance: revert;
  }
  .pagination-container {
    padding: 0 1em;
    text-align: center;
  }
  ul.pagination {
    display: inline-flex;
  }
</style>