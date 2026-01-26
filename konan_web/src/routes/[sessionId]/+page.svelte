<script lang="ts">
	import Editor from '$lib/components/Editor/Editor.svelte';
	import PrintHistory from '$lib/components/PrintHistory.svelte';
	import type { PageProps } from './$types';
	const { data }: PageProps = $props();
</script>

<main class="grid grid-rows-1 gap-x-3 h-full grid-cols-[auto_1fr]">
	<PrintHistory printHistory={data.userHistory} />
	<section class="flex justify-center items-start mt-20 w-full">
		<Editor storedContent={data.item} />
	</section>
</main>
