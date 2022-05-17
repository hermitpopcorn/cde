<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri'
  import { onMount } from 'svelte'
  import { SvelteToast, toast } from '@zerodevx/svelte-toast'
  import Datatable from './lib/Datatable.svelte'

  let dbConnected = false
  let dbConnecting = false
  const dbConnect = async () => {
    dbConnecting = true
    try {
      await invoke("db_connect")
      dbConnecting = false
      dbConnected = true
      toast.push("Successfully connected to database.")
    } catch(e) {
      dbConnecting = false
      dbConnected = false
      toast.push("Could not connect to database.")
    }
  }
  onMount(() => { dbConnect() })
</script>

<svelte:head>
  <link
    rel="stylesheet"
    href="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css"
  />
</svelte:head>

<main>
  {#if !dbConnected}
    <section class="connection">
      <div class="container">
        {#if !dbConnecting}
          Not connected to database. <button class="btn btn-primary btn-sm" on:click={() => { dbConnect() }}>Attempt Connection</button>
        {:else}
          Connecting...
        {/if}
      </div>
    </section>
  {/if}
  <section class="datatable">
    <Datatable />
  </section>
</main>

<style>
  :root {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  }

  section {
    padding: 1em;
  }

  section.connection {
    background-color: rgba(255, 173, 173, 0.7);
  }

  section.datatable {
    min-height: 4em;
    margin-bottom: 3em;
  }
</style>

<SvelteToast />