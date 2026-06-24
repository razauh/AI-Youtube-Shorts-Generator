<script>
  import { createEventDispatcher, onDestroy } from 'svelte';

  export let ariaLabel = '';
  export let value = '';
  export let options = [];
  export let disabled = false;

  const dispatch = createEventDispatcher();
  let open = false;
  let rootEl;
  let buttonEl;
  let listEl;
  let activeIndex = -1;

  $: selectedIndex = options.findIndex((option) => option.value === value);
  $: selectedLabel = selectedIndex >= 0 ? options[selectedIndex].label : '';

  function closeMenu() {
    open = false;
    activeIndex = selectedIndex >= 0 ? selectedIndex : 0;
  }

  function toggleMenu() {
    if (disabled) return;
    open = !open;
    activeIndex = selectedIndex >= 0 ? selectedIndex : 0;
  }

  function selectValue(nextValue) {
    value = nextValue;
    dispatch('input', { value: nextValue });
    dispatch('change', { value: nextValue });
    closeMenu();
    buttonEl?.focus();
  }

  function onOptionMouseDown(event, nextValue) {
    event.preventDefault();
    event.stopPropagation();
    selectValue(nextValue);
  }

  function onKeydown(event) {
    if (disabled) return;
    if (!open && (event.key === 'ArrowDown' || event.key === 'ArrowUp' || event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      open = true;
      activeIndex = selectedIndex >= 0 ? selectedIndex : 0;
      return;
    }
    if (!open) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      closeMenu();
      buttonEl?.focus();
      return;
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      activeIndex = Math.min(options.length - 1, activeIndex + 1);
      return;
    }
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      activeIndex = Math.max(0, activeIndex - 1);
      return;
    }
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      const option = options[activeIndex];
      if (option) selectValue(option.value);
    }
  }

  function onDocumentClick(event) {
    if (!rootEl?.contains(event.target)) {
      closeMenu();
    }
  }

  if (typeof document !== 'undefined') {
    document.addEventListener('click', onDocumentClick, true);
  }

  onDestroy(() => {
    if (typeof document !== 'undefined') {
      document.removeEventListener('click', onDocumentClick, true);
    }
  });
</script>

<div class="themed-select" bind:this={rootEl}>
  <button
    type="button"
    class="select-trigger"
    bind:this={buttonEl}
    aria-label={ariaLabel}
    aria-haspopup="listbox"
    aria-expanded={open}
    disabled={disabled}
    on:click={toggleMenu}
    on:keydown={onKeydown}
  >
    <span>{selectedLabel}</span>
    <span class="chevron" aria-hidden="true"></span>
  </button>
  {#if open}
    <ul
      class="select-menu"
      role="listbox"
      bind:this={listEl}
      aria-label={ariaLabel}
      on:mousedown|preventDefault
    >
      {#each options as option, index}
        <li
          role="option"
          aria-selected={option.value === value}
          class:selected={option.value === value}
          class:active={index === activeIndex}
          on:mousedown={(event) => onOptionMouseDown(event, option.value)}
        >
          {option.label}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .themed-select {
    position: relative;
  }
  .select-trigger {
    width: 100%;
    min-height: 2.45rem;
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 34%, transparent);
    background: color-mix(in srgb, var(--color-surface-input) 88%, var(--color-panel-card));
    color: var(--color-text-primary);
    padding: .6rem 2rem .6rem .7rem;
    font-family: inherit;
    text-align: left;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-sm);
  }
  .select-trigger:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--color-focus-ring) 44%, var(--color-border-strong));
  }
  .select-trigger:focus-visible {
    outline: 2px solid color-mix(in srgb, var(--color-focus-ring) 62%, transparent);
    outline-offset: 1px;
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-focus-ring) 24%, transparent);
  }
  .select-trigger:disabled {
    cursor: not-allowed;
    opacity: .66;
  }
  .chevron {
    width: .45rem;
    height: .45rem;
    border-right: 2px solid color-mix(in srgb, var(--color-text-tertiary) 92%, transparent);
    border-bottom: 2px solid color-mix(in srgb, var(--color-text-tertiary) 92%, transparent);
    transform: translateY(-.08rem) rotate(45deg);
    flex: 0 0 auto;
  }
  .select-menu {
    position: absolute;
    z-index: 30;
    margin: .25rem 0 0;
    width: 100%;
    max-height: 15rem;
    overflow: auto;
    list-style: none;
    padding: .3rem;
    border-radius: var(--radius-md);
    border: 1px solid color-mix(in srgb, var(--color-border-strong) 30%, transparent);
    background: color-mix(in srgb, var(--color-panel-card) 94%, var(--color-surface-input));
    box-shadow: 0 14px 36px rgba(0, 0, 0, .32);
  }
  .select-menu li {
    padding: .5rem .55rem;
    border-radius: var(--radius-xs);
    cursor: pointer;
    color: var(--color-text-primary);
  }
  .select-menu li.active,
  .select-menu li:hover {
    background: color-mix(in srgb, var(--color-focus-ring) 15%, transparent);
  }
  .select-menu li.selected {
    color: var(--color-secondary);
    font-weight: 700;
  }
</style>
