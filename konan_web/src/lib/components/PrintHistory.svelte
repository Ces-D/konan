<script lang="ts">
	import { page } from '$app/state';
	import SquareMenuIcon from '$lib/components/Icons/SquareMenu.svelte';
	import {
		PrintHistoryStore,
		type PrintHistoryEntry
	} from '$lib/printHistory';
	import { generateText } from '@tiptap/core';
	import { StarterKit } from '@tiptap/starter-kit';
	import DeleteIcon from './Icons/Delete.svelte';
	import TextAlign from '@tiptap/extension-text-align';
	import TaskList from '@tiptap/extension-task-list';
	import TaskItem from '@tiptap/extension-task-item';

	let printHistory = $state<PrintHistoryEntry[]>([]);

	async function fetchHistory() {
		printHistory = await PrintHistoryStore.getAll();
	}

	$effect(() => {
		page.url.searchParams;
		fetchHistory();
	});

	function getPreviewText(entry: PrintHistoryEntry): string {
		return generateText(entry.content, [
			StarterKit,
			TextAlign.configure({
				types: ['heading', 'paragraph'],
				alignments: ['left', 'center', 'right']
			}),
			TaskList,
			TaskItem.configure({ nested: true })
		]);
	}

	async function deletHistoryItem(item: PrintHistoryEntry) {
		await PrintHistoryStore.delete(item.id);
		printHistory = await PrintHistoryStore.getAll();
	}
</script>

<aside class="relative w-5 h-full tablet:w-37 laptop:w-72">
	<section class="overflow-y-auto absolute inset-0 px-1">
		<nav class="flex gap-3 justify-start items-center">
			<h3 class="hidden tablet:block">Print History</h3>
			<SquareMenuIcon />
		</nav>
		<ul>
			{#each printHistory as item}
				<li class="py-1 my-1 border-b border-background-inverted">
					<a
						class="grid items-start p-1 text-left rounded cursor-pointer grid-cols-[1fr_auto] hover:bg-primary-300"
						href={`/?id=${encodeURIComponent(item.id)}`}
					>
						<span class="text-sm opacity-70"
							>{new Date(item.printedAt).toLocaleString()}</span
						>
						<button
							class="rounded cursor-pointer disabled:cursor-not-allowed hover:bg-background-inverted hover:text-text-inverted"
							disabled={item.id ===
								page.url.searchParams.get('id')}
							onclick={(e) => {
								e.preventDefault();
								e.stopPropagation();
								deletHistoryItem(item);
							}}><DeleteIcon /></button
						>
						<p
							class="col-span-2 line-clamp-4"
						>
							{getPreviewText(item)}
						</p>
					</a>
				</li>
			{/each}
		</ul>
	</section>
</aside>
