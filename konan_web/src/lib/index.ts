import { type SvelteToastOptions, toast } from '@zerodevx/svelte-toast';

export const toastSuccess = (message: string) => {
	const toastOptions: SvelteToastOptions = {
		dismissable: false,
		duration: 1_000,
		theme: {
			'--toastBackground': 'var(--color-primary-900)',
			'--toastColor': 'var(--text-color-inverted)',
			'--toastBarHeight': 0
		}
	};
	toast.push(message, toastOptions);
};

export const toastError = (message: string) => {
	const toastOptions: SvelteToastOptions = {
		dismissable: false,
		duration: 1_000,
		theme: {
			'--toastBackground': 'var(--bg-color-inverted)',
			'--toastColor': 'var(--text-color-inverted)',
			'--toastBarHeight': 0
		}
	};
	toast.push(message, toastOptions);
};
