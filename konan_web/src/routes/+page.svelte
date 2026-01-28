<script lang="ts">
	import { page } from '$app/state';
	import Editor from '$lib/components/Forms/Editor/Editor.svelte';
	import OutlineForm from '$lib/components/Forms/OutlineForm.svelte';
	import HabitTrackerForm from '$lib/components/Forms/HabitTrackerForm.svelte';
	import {
		PrintHistoryStore,
		type PrintHistoryEntry
	} from '$lib/printHistory';

	let storedContent = $state<PrintHistoryEntry | undefined>(undefined);
	let lastSessionId: string | null = null;

	$effect(() => {
		const sessionId = page.url.searchParams.get('id');
		if (sessionId === lastSessionId) return;
		lastSessionId = sessionId;

		if (!sessionId) {
			storedContent = undefined;
			return;
		}

		PrintHistoryStore.getById(sessionId).then((entry) => {
			storedContent = entry;
		});
	});
</script>

<section
	class="flex flex-col gap-20 justify-start items-center mx-auto mt-5 w-full tablet:mt-20 tablet:w-11/12 laptop:w-3/4"
>
	<Editor {storedContent} />
	<OutlineForm />
	<HabitTrackerForm />
</section>
