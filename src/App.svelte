<script lang="ts">
  import App from './lib/App.svelte'
</script>

<main>
  <App />
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
</style>
