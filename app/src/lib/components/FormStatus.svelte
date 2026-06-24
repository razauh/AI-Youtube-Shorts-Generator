<script>
  export let message = '';
  export let kind = 'info';
</script>

{#if message}
  <p class="form-status" class:form-status--error={kind === 'error'} class:form-status--success={kind === 'success'}>
    {message}
  </p>
{/if}

<style>
  .form-status {
    margin: 0;
    color: var(--color-text-tertiary);
  }

  .form-status--error {
    color: var(--color-state-error);
  }

  .form-status--success {
    color: var(--color-text-secondary);
  }
</style>
