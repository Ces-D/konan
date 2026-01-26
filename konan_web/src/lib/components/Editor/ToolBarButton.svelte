<script lang="ts">
	let { title, isActive, onClick, children } = $props();
</script>

<button
	onclick={(e) => {
		e.preventDefault();
		onClick();
	}}
	class="flex justify-center items-center w-8 h-8 rounded border cursor-pointer hover:bg-primary-300"
	class:active={isActive}
	{title}
	aria-label={title}
>
	{@render children()}
</button>

<style lang="postcss">
	@reference "../../../routes/layout.css";

	.active {
		@apply bg-background-inverted text-text-inverted;
	}
</style>
