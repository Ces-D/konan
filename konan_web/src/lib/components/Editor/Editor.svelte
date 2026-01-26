<script lang="ts">
	import { applyAction, deserialize } from '$app/forms';
	import { onMount, onDestroy } from 'svelte';
	import { Editor } from '@tiptap/core';
	import TextAlign from '@tiptap/extension-text-align';
	import TaskList from '@tiptap/extension-task-list';
	import TaskItem from '@tiptap/extension-task-item';
	import { StarterKit } from '@tiptap/starter-kit';
	import type { ActionResult } from '@sveltejs/kit';
	import { invalidateAll } from '$app/navigation';
	import { CPL, type EditorContent } from '$lib/printHistory';
	import PrinterIcon from '../Icons/Printer.svelte';
	import ToolBar from './ToolBar.svelte';

	const {
		storedContent,
		children
	}: { storedContent?: Partial<EditorContent>; children: any } = $props();
	let elementRef: HTMLElement;
	let editorState = $state<{ editor: Editor | null }>({ editor: null });
	let lastContentId: string | undefined = storedContent?.id;

	function initEditor(content?: string) {
		editorState.editor = new Editor({
			injectCSS: false,
			element: elementRef,
			extensions: [
				StarterKit,
				TextAlign.configure({
					types: ['heading', 'paragraph'],
					alignments: ['left', 'center', 'right']
				}),
				TaskList,
				TaskItem.configure({ nested: true })
			],
			content,
			onTransaction: ({ editor }) => {
				editorState = { editor };
			}
		});
	}

	onMount(() => {
		initEditor(storedContent?.text);
	});

	onDestroy(() => {
		editorState.editor?.destroy();
	});

	// When storedContent.id changes, reinitialize the editor to reset history
	$effect(() => {
		const id = storedContent?.id as string | undefined;
		if (id === lastContentId) return;
		lastContentId = id;
		if (!elementRef) return;
		if (editorState.editor) {
			editorState.editor.destroy();
		}
		initEditor(storedContent?.text);
	});

	async function handleSubmit(
		event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement }
	) {
		event.preventDefault();
		const data = new FormData();
		if (storedContent?.id) {
			data.set('id', storedContent?.id);
		}
		data.set('text', JSON.stringify(editorState.editor?.getJSON()) ?? '');

		const response = await fetch(event.currentTarget.action, {
			method: 'POST',
			body: data
		});

		const result: ActionResult = deserialize(await response.text());

		if (result.type === 'success') {
			// rerun all `load` functions, following the successful update
			await invalidateAll();
		}

		applyAction(result);
	}
</script>

<form
	method="POST"
	class="overflow-hidden"
	onsubmit={handleSubmit}
	action="?/json"
>
	<ToolBar editor={editorState.editor} />
	<!-- Editor area -->
	<article class="editor-content" bind:this={elementRef}></article>

	<!-- Footer hint -->
	<div class="editor-footer">
		<span class="font-semibold">Max {CPL} characters per line</span>
		<div class="footer-hints">
			Ctrl+B Bold | Ctrl+I Italic | Enter New line
		</div>
	</div>
	<button
		class="print-button"
		type="submit"
		disabled={!editorState.editor || editorState.editor.isEmpty}
	>
		<PrinterIcon />
		Print
	</button>
</form>

{@render children()}

<style lang="postcss">
	@reference "../../../routes/layout.css";

	.editor-content {
		@apply mx-auto my-4 h-96 w-[482px] overflow-x-hidden overflow-y-scroll rounded-md border-2 border-primary-900 p-2 shadow;
	}
	.editor-content :global(.tiptap) {
		@apply h-full text-[16px] leading-6 wrap-break-word whitespace-normal outline-none;
	}

	.editor-content :global(.tiptap p) {
		@apply text-[16px];
	}

	/* Headings from StarterKit (levels 1-6) */
	.editor-content :global(.tiptap h1) {
		@apply text-[42px] font-bold;
	}

	.editor-content :global(.tiptap h2) {
		@apply text-[32px] font-bold;
	}

	.editor-content :global(.tiptap h3) {
		@apply text-[32px];
	}
	.editor-content :global(.tiptap h4) {
		@apply text-[16px] font-bold;
	}
	.editor-content :global(.tiptap h5) {
		@apply text-[14px] font-semibold;
	}
	.editor-content :global(.tiptap h6) {
		@apply text-[12px] font-semibold tracking-wide uppercase;
	}

	.editor-content :global(.tiptap ul),
	.editor-content :global(.tiptap ol) {
		@apply list-inside text-[16px];
	}
	.editor-content :global(.tiptap li::marker) {
		@apply mr-1;
	}
	.editor-content :global(.tiptap li > p) {
		@apply inline;
	}
	.editor-content :global(.tiptap ul) {
		@apply list-disc;
	}
	.editor-content :global(.tiptap ol) {
		@apply list-decimal;
	}

	/* Inline code */
	.editor-content :global(.tiptap code) {
		@apply rounded bg-background-inverted text-[16px] text-text-inverted;
	}

	/* Code block */
	.editor-content :global(.tiptap pre) {
		@apply my-2 rounded bg-background-inverted p-1 text-[16px] wrap-break-word whitespace-normal text-text-inverted;
		font-family:
			ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
			'Liberation Mono', 'Courier New', monospace;
	}

	/* Task list styles (TaskList/TaskItem) */
	.editor-content :global(.tiptap ul[data-type='taskList']) {
		@apply m-0 list-none p-0;
	}
	.editor-content :global(.tiptap ul[data-type='taskList'] li) {
		@apply flex items-start text-[16px];
	}
	.editor-content :global(.tiptap ul[data-type='taskList'] li > label) {
		@apply flex shrink-0 grow-0 select-none;
	}
	.editor-content :global(.tiptap ul[data-type='taskList'] li > div) {
		@apply flex shrink grow basis-auto;
	}
	.editor-content
		:global(.tiptap ul[data-type='taskList'] input[type='checkbox']) {
		@apply mt-1 mr-1;
	}
	.editor-content
		:global(
			.tiptap ul[data-type='taskList'] li[data-checked='true'] > div
		) {
		@apply line-through;
	}

	.editor-footer {
		@apply flex-col gap-2 border-t px-2 py-3 text-sm;
	}

	.footer-hints {
		@apply flex flex-col gap-1 opacity-70;
	}

	.print-button {
		@apply my-4 flex items-center gap-2 rounded-md bg-primary-900 px-4 py-2 text-text-inverted;
	}

	.print-button:hover:not(:disabled) {
		@apply opacity-90;
	}

	.print-button:disabled {
		@apply cursor-not-allowed opacity-50;
	}
</style>
