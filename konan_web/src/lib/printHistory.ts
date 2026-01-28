import type { JSONContent } from '@tiptap/core';

export type PrintHistoryEntry = {
	id: string;
	content: JSONContent;
	printedAt: number;
};

export const CPL = 48;

const DB_NAME = 'konan_print_history';
const DB_VERSION = 1;
const STORE_NAME = 'print_history';

/** example: "f1a3c9e2b84d7a0c6f9d12e45a7b8c3d" */
const randomId = () => {
	const array = new Uint8Array(16);
	crypto.getRandomValues(array);
	return Array.from(array, (b) => b.toString(16).padStart(2, '0')).join('');
};

function openDb(): Promise<IDBDatabase> {
	return new Promise((resolve, reject) => {
		const request = indexedDB.open(DB_NAME, DB_VERSION);
		request.onerror = () => reject(request.error);
		request.onsuccess = () => resolve(request.result);
		request.onupgradeneeded = (event) => {
			const db = (event.target as IDBOpenDBRequest).result;
			if (!db.objectStoreNames.contains(STORE_NAME)) {
				const store = db.createObjectStore(STORE_NAME, {
					keyPath: 'id'
				});
				store.createIndex('printedAt', 'printedAt', { unique: false });
			}
		};
	});
}

export class PrintHistoryStore {
	private content: JSONContent | null;
	private sessionId: string;

	constructor(content: JSONContent | null, sessionId?: string | null) {
		this.content = content;
		this.sessionId = sessionId ?? randomId();
	}

	private isContentValid(): boolean {
		return this.content != null && typeof this.content === 'object';
	}

	async save(): Promise<PrintHistoryEntry> {
		if (!this.isContentValid()) {
			throw new Error('Content is not valid');
		}

		const entry: PrintHistoryEntry = {
			id: this.sessionId,
			content: this.content!,
			printedAt: Date.now()
		};

		const db = await openDb();
		return new Promise((resolve, reject) => {
			const tx = db.transaction(STORE_NAME, 'readwrite');
			const store = tx.objectStore(STORE_NAME);
			const request = store.put(entry);
			request.onerror = () => reject(request.error);
			request.onsuccess = () => resolve(entry);
			tx.oncomplete = () => db.close();
		});
	}

	static async getById(id: string): Promise<PrintHistoryEntry | undefined> {
		const db = await openDb();
		return new Promise((resolve, reject) => {
			const tx = db.transaction(STORE_NAME, 'readonly');
			const store = tx.objectStore(STORE_NAME);
			const request = store.get(id);
			request.onerror = () => reject(request.error);
			request.onsuccess = () => resolve(request.result);
			tx.oncomplete = () => db.close();
		});
	}

	static async getAll(): Promise<PrintHistoryEntry[]> {
		const db = await openDb();
		return new Promise((resolve, reject) => {
			const tx = db.transaction(STORE_NAME, 'readonly');
			const store = tx.objectStore(STORE_NAME);
			const index = store.index('printedAt');
			const request = index.getAll();
			request.onerror = () => reject(request.error);
			request.onsuccess = () => resolve(request.result);
			tx.oncomplete = () => db.close();
		});
	}

	static async delete(id: string): Promise<void> {
		const db = await openDb();
		return new Promise((resolve, reject) => {
			const tx = db.transaction(STORE_NAME, 'readwrite');
			const store = tx.objectStore(STORE_NAME);
			const request = store.delete(id);
			request.onerror = () => reject(request.error);
			request.onsuccess = () => resolve();
			tx.oncomplete = () => db.close();
		});
	}
}
