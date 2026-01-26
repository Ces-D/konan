import { UpsertPrintHistory } from '$lib/printHistory';
import { redirect, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { DEMO_PRINT_HISTORY } from '$lib/data';

/* Updates the print history. Either updates the existing item in history or creates a new item */
export const actions = {
	default: async (event) => {
		const formData = await event.request.formData();
		let id = formData.get('id');
		const text = formData.get('text');
		const handler = new UpsertPrintHistory(
			text?.toString(),
			id?.toString()
		);
		handler.execute();
	}
} satisfies Actions;

export const load: PageServerLoad = (req) => {
	const sessionId = req.params.sessionId;
	console.log('Searching for sessionId: ', sessionId);
	const found = DEMO_PRINT_HISTORY.findIndex((v) => v.id === sessionId);
	if (found < 0) {
		redirect(307, '/');
	}
	const item = DEMO_PRINT_HISTORY[found];
	const userHistory = DEMO_PRINT_HISTORY;
	console.log('found item: ', item);
	return { item, userHistory };
};
