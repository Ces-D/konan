<script lang="ts">
	import SquareMenuIcon from '$lib/components/Icons/SquareMenu.svelte';
	import { type PrintHistory } from '$lib/printHistory';

	const props: { printHistory: PrintHistory } = $props();
</script>

<aside class="relative w-72 h-full">
	<section class="overflow-y-auto absolute inset-0 px-2">
		<nav class="flex gap-3 justify-start items-center">
			<h3>Print History</h3>
			<SquareMenuIcon />
		</nav>
		<ul>
			{#each props.printHistory as item}
				<li class="py-2 my-1 border-b border-background-inverted">
					<a
						class="text-left cursor-pointer line-clamp-4 hover:bg-primary-300"
						href={`/${encodeURIComponent(item.id)}`}
					>
						{item.text}
					</a>
				</li>
			{/each}
		</ul>
	</section>
</aside>
