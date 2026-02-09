import { type JSONContent } from '@tiptap/core';
import { env } from '$env/dynamic/public';
const MESSAGE_ENDPOINT = env.PUBLIC_MESSAGE_ENDPOINT;

export class EditorMessage {
	private content: JSONContent;
	private rows?: number;

	private setRows(r: number) {
		if (r > 0) {
			this.rows = r;
		}
	}
	constructor(content: JSONContent, rows?: number) {
		if (rows) {
			this.setRows(parseInt(rows.toString()));
		}
		console.log(content);
		this.content = content;
	}

	async printEditorMessage() {
		const url = new URL(MESSAGE_ENDPOINT!);
		const res = await fetch(url, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ content: this.content, rows: this.rows })
		});
		if (res.ok) {
			return { success: true, message: 'Printed message successfully!' };
		} else {
			return { success: false, message: await res.text() };
		}
	}
}
