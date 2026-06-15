<script lang="ts">
  import '../app.css';
  import { onDestroy, onMount } from 'svelte';
  import { disposeTauriBridge, initializeTauriBridge } from '$lib/stores/app';

  onMount(() => {
    initializeTauriBridge().catch(() => undefined);
  });

  onDestroy(() => {
    disposeTauriBridge();
  });
</script>

<slot />
