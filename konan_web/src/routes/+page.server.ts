import { UpsertPrintHistory } from '$lib/printHistory';
import type { Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { DEMO_PRINT_HISTORY } from '$lib/data';

type Status = {
	success: boolean;
	message: string;
};

/* Updates the print history. Either updates the existing item in history or creates a new item */
export const actions = {
	json: async (event): Promise<Status> => {
		const formData = await event.request.formData();
		let id = formData.get('id');
		const text = formData.get('text');
		const handler = new UpsertPrintHistory(
			text?.toString(),
			id?.toString()
		);
		handler.execute();
		return {
			success: false,
			message:
				'This is currently always a test. Until I get the TipTapJsonAdapter running on the server'
		};
	}
} satisfies Actions;

export const load: PageServerLoad = () => {
	const userHistory = DEMO_PRINT_HISTORY;
	return { userHistory };
};
