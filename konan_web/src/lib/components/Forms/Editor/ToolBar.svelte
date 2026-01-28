<script lang="ts">
	import { Editor } from '@tiptap/core';
	import ToolBarButton from './ToolBarButton.svelte';
	import Heading1Icon from '$lib/components/Icons/Heading1.svelte';
	import Heading2Icon from '$lib/components/Icons/Heading2.svelte';
	import Heading3Icon from '$lib/components/Icons/Heading3.svelte';
	import Heading4Icon from '$lib/components/Icons/Heading4.svelte';
	import ParagraphIcon from '$lib/components/Icons/Paragraph.svelte';
	import BoldIcon from '$lib/components/Icons/Bold.svelte';
	import UnderlineIcon from '$lib/components/Icons/Underline.svelte';
	import CodeIcon from '$lib/components/Icons/Code.svelte';
	import BulletListIcon from '$lib/components/Icons/BulletList.svelte';
	import NumberListIcon from '$lib/components/Icons/NumberList.svelte';
	import CheckBoxListIcon from '$lib/components/Icons/CheckBoxList.svelte';
	import LeftAlignIcon from '$lib/components/Icons/LeftAlign.svelte';
	import RightAlignIcon from '$lib/components/Icons/RightAlign.svelte';
	import CenterAlignIcon from '$lib/components/Icons/CenterAlign.svelte';

	const { editor }: { editor?: Editor | null } = $props();
</script>

<div class="toolbar">
	<div class="toolbar-button-container">
		<h4>Text Styles</h4>
		<li class="toolbar-buttons">
			<ToolBarButton
				title="Heading 1"
				onClick={() =>
					editor?.chain().focus().toggleHeading({ level: 1 }).run()}
				isActive={editor?.isActive('heading', {
					level: 1
				})}><Heading1Icon /></ToolBarButton
			>
			<ToolBarButton
				title="Heading 2"
				onClick={() =>
					editor?.chain().focus().toggleHeading({ level: 2 }).run()}
				isActive={editor?.isActive('heading', {
					level: 2
				})}><Heading2Icon /></ToolBarButton
			>
			<ToolBarButton
				title="Heading 3"
				onClick={() =>
					editor?.chain().focus().toggleHeading({ level: 3 }).run()}
				isActive={editor?.isActive('heading', {
					level: 3
				})}><Heading3Icon /></ToolBarButton
			>
			<ToolBarButton
				title="Heading 4"
				onClick={() =>
					editor?.chain().focus().toggleHeading({ level: 4 }).run()}
				isActive={editor?.isActive('heading', {
					level: 4
				})}><Heading4Icon /></ToolBarButton
			>
			<ToolBarButton
				title="Paragraph"
				onClick={() => editor?.chain().focus().setParagraph().run()}
				isActive={editor?.isActive('paragraph')}
				><ParagraphIcon /></ToolBarButton
			>
		</li>
	</div>

	<div class="toolbar-button-container">
		<h4>Format</h4>
		<li class="toolbar-buttons">
			<ToolBarButton
				onClick={() => editor?.chain().focus().toggleBold().run()}
				isActive={editor?.isActive('bold')}
				title="Bold (Ctrl+B)"
			>
				<BoldIcon />
			</ToolBarButton>
			<ToolBarButton
				onClick={() => editor?.chain().focus().toggleUnderline().run()}
				isActive={editor?.isActive('underline')}
				title="Underline"
			>
				<UnderlineIcon />
			</ToolBarButton>
			<ToolBarButton
				onClick={() => editor?.chain().focus().toggleCodeBlock().run()}
				isActive={editor?.isActive('codeBlock')}
				title="Code Block"
			>
				<CodeIcon />
			</ToolBarButton>
		</li>
	</div>

	<div class="toolbar-button-container">
		<h4>Lists</h4>
		<li class="toolbar-buttons">
			<ToolBarButton
				onClick={() => editor?.chain().focus().toggleBulletList().run()}
				isActive={editor?.isActive('bulletList')}
				title="Bullet list"
				><BulletListIcon />
			</ToolBarButton>
			<ToolBarButton
				onClick={() =>
					editor?.chain().focus().toggleOrderedList().run()}
				isActive={editor?.isActive('orderedList')}
				title="Numbered list"
			>
				<NumberListIcon />
			</ToolBarButton>
			<ToolBarButton
				onClick={() => editor?.chain().focus().toggleTaskList().run()}
				isActive={editor?.isActive('taskList')}
				title="Checkbox list"
			>
				<CheckBoxListIcon />
			</ToolBarButton>
		</li>
	</div>
	<div class="toolbar-button-container">
		<h4>Alignment</h4>
		<li class="toolbar-buttons">
			<ToolBarButton
				title="Left Align"
				onClick={() =>
					editor?.chain().focus().setTextAlign('left').run()}
				isActive={editor?.isActive({
					textAlign: 'left'
				})}><LeftAlignIcon /></ToolBarButton
			>
			<ToolBarButton
				title="Center Align"
				onClick={() =>
					editor?.chain().focus().setTextAlign('center').run()}
				isActive={editor?.isActive({
					textAlign: 'center'
				})}><CenterAlignIcon /></ToolBarButton
			>
			<ToolBarButton
				title="Right Align"
				onClick={() =>
					editor?.chain().focus().setTextAlign('right').run()}
				isActive={editor?.isActive({
					textAlign: 'right'
				})}><RightAlignIcon /></ToolBarButton
			>
		</li>
	</div>
</div>

<style lang="postcss">
	@reference "../../../../routes/layout.css";
	.toolbar {
		@apply flex flex-wrap justify-center;
	}

	.toolbar-button-container {
		@apply m-2 grid grid-cols-1 grid-rows-2 p-2;
	}
	.toolbar-button-container:last-child {
		@apply border-r-0;
	}

	.toolbar-buttons {
		@apply flex items-center gap-2;
	}
</style>
