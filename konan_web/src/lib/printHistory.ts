import { DEMO_PRINT_HISTORY } from '$lib/data';

export type PrintHistory = Array<EditorContent>;
export type EditorContent = { id: string; text: string };

export const CPL = 48;

/** example: "f1a3c9e2b84d7a0c6f9d12e45a7b8c3d" */
const randomId = () => {
	const array = new Uint8Array(16);
	crypto.getRandomValues(array);
	return Array.from(array, (b) => b.toString(16).padStart(2, '0')).join('');
};

export class UpsertPrintHistory {
	text?: string | null;
	id?: string | null;
	constructor(text?: string | null, id?: string | null) {
		this.text = text;
		this.id = id;
	}

	private textIsValid(): boolean {
		let isValid = false;
		isValid = typeof this.text === 'string';
		isValid = (this.text ?? '').trim().length > 0;
		return isValid;
	}
	private isNewSession(): boolean {
		return this.id == null;
	}
	private sessionId(): string {
		return this.id ?? randomId();
	}

	execute() {
		if (!this.textIsValid()) {
			throw new Error('Text is not valid');
		}
		const content: EditorContent = {
			id: this.sessionId(),
			text: this.text!
		};
		if (!this.isNewSession()) {
			const foundIndex = DEMO_PRINT_HISTORY.findIndex(
				(v) => v.id == this.sessionId()
			);
			if (foundIndex > -1) {
				DEMO_PRINT_HISTORY.splice(foundIndex, 1, content);
				return DEMO_PRINT_HISTORY;
			}
		}
		DEMO_PRINT_HISTORY.push(content);
		return DEMO_PRINT_HISTORY;
	}
}
