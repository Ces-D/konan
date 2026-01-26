<script lang="ts">
	import { enhance } from '$app/forms';
	import Editor from '$lib/components/Editor/Editor.svelte';
	import OutlineFormAction from '$lib/components/Editor/OutlineFormAction.svelte';
	import HabitTrackerFormAction from '$lib/components/Editor/HabitTrackerFormAction.svelte';
	import PrintHistory from '$lib/components/PrintHistory.svelte';
	import { Confetti } from 'svelte-confetti';
	import type { PageProps } from './$types';
	import { toast } from '@zerodevx/svelte-toast';
	const { data, form }: PageProps = $props();
	$effect(() => {
		if (form?.success) {
			toast.push(form.message);
		}
	});
</script>

<main class="grid grid-rows-1 gap-x-3 h-full grid-cols-[auto_1fr]">
	<PrintHistory printHistory={data.userHistory} />
	<section class="flex justify-center items-start mt-20 w-full">
		<section>
			<Editor>
				<OutlineFormAction />
				<HabitTrackerFormAction />
			</Editor>
		</section>
	</section>
	{#if form?.success}
		<Confetti size={300} duration={500} destroyOnComplete />
	{/if}
</main>
