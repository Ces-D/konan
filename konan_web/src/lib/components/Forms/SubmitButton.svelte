<script lang="ts">
	import { type Snippet } from 'svelte';
	const { children, disabled }: { children: Snippet; disabled: boolean } =
		$props();
</script>

<button class="submit-button" type="submit" {disabled}>
	{@render children()}
</button>

<style lang="postcss">
	@reference "../../../routes/layout.css";

	.submit-button {
		@apply my-4 flex cursor-pointer items-center gap-2 rounded-md bg-primary-900 px-4 py-2 text-text-inverted;
	}

	.submit-button:hover:not(:disabled) {
		@apply opacity-90 ;
	}

	.submit-button:disabled {
		@apply cursor-not-allowed opacity-50;
	}
</style>
