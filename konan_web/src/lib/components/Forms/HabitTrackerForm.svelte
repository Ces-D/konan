<script lang="ts">
	import { toastError, toastSuccess } from '$lib';
	import { HabitTrackerTemplate } from '$lib/template';
	import CalendarIcon from '../Icons/Calendar.svelte';
	import InputLabel from './InputLabel.svelte';
	import SubmitButton from './SubmitButton.svelte';

	const handleSubmit = async (
		e: SubmitEvent & {
			currentTarget: EventTarget & HTMLFormElement;
		}
	) => {
		e.preventDefault();
		const data = new FormData(e.currentTarget);
		const habitTracker = new HabitTrackerTemplate(data);
		const res = await habitTracker.printHabitTrackerTemplate();
		if (res.success) {
			toastSuccess(res.message);
		} else {
			toastError(res.message);
		}
	};
</script>

<form onsubmit={handleSubmit} class="flex flex-col gap-4 justify-start w-full">
	<h1>Habit Tracker</h1>
	<div class="grid grid-cols-2 gap-4">
		<InputLabel
			label="Habit"
			htmlFor="habit"
			info="The habit you want to practice"
			required
		>
			<input
				type="text"
				id="habit"
				name="habit"
				placeholder="e.g., Exercise, Read, Meditate"
				required
				class="py-2 px-3 w-full rounded-md border focus:ring-2 focus:outline-none border-text/20 bg-background placeholder:text-text/40 focus:border-primary-900 focus:ring-primary-900/50"
			/>
		</InputLabel>
		<InputLabel
			label="Start Date"
			htmlFor="start-date"
			info="Habit start date"
			required
		>
			<input
				type="date"
				id="start-date"
				name="start-date"
				required
				class="py-2 px-3 w-full rounded-md border focus:ring-2 focus:outline-none border-text/20 bg-background focus:border-primary-900 focus:ring-primary-900/50"
			/>
		</InputLabel>
		<InputLabel
			label="End Date"
			htmlFor="end-date"
			info="Habit end date"
			required
		>
			<input
				type="date"
				id="end-date"
				name="end-date"
				required
				class="py-2 px-3 w-full rounded-md border focus:ring-2 focus:outline-none border-text/20 bg-background focus:border-primary-900 focus:ring-primary-900/50"
			/>
		</InputLabel>
	</div>

	<div>
		<SubmitButton disabled={false}>
			<CalendarIcon />
			Print Habit Tracker
		</SubmitButton>
	</div>
</form>
