<script lang="ts">
	import OutlineBookIcon from '../Icons/OutlineBook.svelte';

	const handleSubmit = async () => {
		console.log('Submitting');
		const url = new URL('/template/habit-tracker', 'http://127.0.0.1:8080');
		console.log('Making request to ', url.toString());
		const res = await fetch(url, { method: 'GET' });
		if (res.ok) {
			return { success: true, message: 'Printed template successfully!' };
		} else {
			return { success: false, message: 'Failed to print template!' };
		}
	};
</script>

<form onsubmit={handleSubmit}>
	<button type="submit" class="print-button">
		<OutlineBookIcon />
		Habit Tracker</button
	>
</form>
