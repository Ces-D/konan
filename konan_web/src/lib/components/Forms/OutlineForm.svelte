<script lang="ts">
	import OutlineBookIcon from '../Icons/OutlineBook.svelte';
	import InputLabel from './InputLabel.svelte';
	import SubmitButton from './SubmitButton.svelte';
	import { OutlineTemplate } from '$lib/template';
	import { toastSuccess, toastError } from '$lib';

	const handleSubmit = async (
		e: SubmitEvent & {
			currentTarget: EventTarget & HTMLFormElement;
		}
	) => {
		e.preventDefault();
		const data = new FormData(e.currentTarget);
		const outline = new OutlineTemplate(data);
		const res = await outline.printOutlineTemplate();
		if (res.success) {
			toastSuccess(res.message);
		} else {
			toastError(res.message);
		}
	};
</script>

<form onsubmit={handleSubmit} class="flex flex-col gap-4 justify-center w-full">
	<h1>Outline</h1>
	<div class="grid grid-cols-2 gap-4">
		<InputLabel
			label="Rows"
			htmlFor="rows"
			info="Print as pages with this many rows"
		>
			<input
				type="number"
				id="rows"
				name="rows"
				min="1"
				max="100"
				class="py-2 px-3 w-full rounded-md border focus:ring-2 focus:outline-none border-text/20 bg-background focus:border-primary-900 focus:ring-primary-900/50"
			/>
		</InputLabel>
		<InputLabel
			label="Date"
			htmlFor="date"
			info="Attach a date label to the printout "
		>
			<input
				type="date"
				id="date"
				name="date"
				class="py-2 px-3 w-full rounded-md border focus:ring-2 focus:outline-none border-text/20 bg-background focus:border-primary-900 focus:ring-primary-900/50"
			/>
		</InputLabel>
		<InputLabel
			label="Banner"
			htmlFor="banner"
			info="Attach large banner text to top of printout "
		>
			<input
				type="text"
				id="banner"
				name="banner"
				placeholder="Custom header text"
				class="py-2 px-3 w-full rounded-md border focus:ring-2 focus:outline-none border-text/20 bg-background placeholder:text-text/40 focus:border-primary-900 focus:ring-primary-900/50"
			/>
		</InputLabel>
		<InputLabel
			label="Lined"
			htmlFor="lined"
			info="Print out a lined sheet"
		>
			<input
				type="checkbox"
				id="lined"
				name="lined"
				class="w-5 h-5 rounded border focus:ring-2 border-text/20 bg-background accent-primary-900 focus:ring-primary-900/50"
			/>
		</InputLabel>
	</div>

	<div>
		<SubmitButton disabled={false}>
			<OutlineBookIcon />
			Print Outline
		</SubmitButton>
	</div>
</form>
