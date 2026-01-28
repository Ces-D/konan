<script lang="ts">
	import { type Snippet } from 'svelte';
	import InfoIcon from '../Icons/Info.svelte';

	const {
		label,
		htmlFor,
		children,
		required,
		info
	}: {
		label: string;
		htmlFor: string;
		children: Snippet;
		required?: boolean;
		info?: string;
	} = $props();

	let showInfo = $state(false);
</script>

<div class="input-label">
	<label for={htmlFor} class="flex flex-1 gap-2 items-center">
		{label}

		{#if info}
			<button
				type="button"
				class="info-trigger"
				onmouseenter={() => (showInfo = true)}
				onmouseleave={() => (showInfo = false)}
				onclick={() => (showInfo = !showInfo)}
				aria-label="More information"
			>
				<InfoIcon />
				{#if showInfo}
					<div class="info-popup">
						{info}
					</div>
				{/if}
			</button>
		{/if}
		{#if required}
			<span class="text-lg text-primary-900">*</span>
		{/if}
	</label>
	{@render children()}
</div>

<style lang="postcss">
	@reference "../../../routes/layout.css";

	.input-label {
		@apply flex w-full flex-col gap-2;
	}

	.info-trigger {
		@apply relative flex h-5 w-5 cursor-pointer items-center justify-center opacity-50 transition-opacity hover:opacity-100;
	}

	.info-trigger :global(svg) {
		@apply h-4 w-4;
	}

	.info-popup {
		@apply absolute bottom-full left-1/2 z-10 mb-2 -translate-x-1/2 rounded-md px-3 py-2 text-sm whitespace-nowrap shadow-lg;
		@apply bg-background-inverted text-text-inverted;
	}

	.info-popup::after {
		content: '';
		@apply absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent;
		border-top-color: var(--bg-color-inverted);
	}
</style>
