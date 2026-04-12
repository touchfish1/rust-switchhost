<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import App from './lib/App.svelte'
  import TrayMetricsWidget from './lib/components/TrayMetricsWidget.svelte'

  const isTrayMetricsWindow = getCurrentWindow().label === 'tray-metrics'
</script>

<main>
  {#if isTrayMetricsWindow}
    <TrayMetricsWidget />
  {:else}
    <App />
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }
  
  :global(*) {
    box-sizing: border-box;
  }

  main {
    min-height: 100vh;
  }
</style>
