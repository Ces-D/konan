import { env } from '$env/dynamic/public';
const HABITS_ENDPOINT = env.PUBLIC_HABITS_ENDPOINT;
const OUTLINE_ENDPOINT = env.PUBLIC_OUTLINE_ENDPOINT;

export class OutlineTemplate {
	private rows?: number;
	private banner?: string;
	private date?: Date;
	private lined: boolean = false;

	private setRows(r: number) {
		if (r > 0) {
			this.rows = r;
		}
	}
	private setBanner(b: string) {
		if (b.trim().length > 0) {
			this.banner = b;
		}
	}
	private setDate(d: Date) {
		this.date = d;
	}
	private setLined(l: boolean) {
		this.lined = l;
	}

	constructor(data: FormData) {
		const rows = data.get('rows');
		if (rows) {
			this.setRows(parseInt(rows.toString()));
		}
		const banner = data.get('banner');
		if (banner) {
			this.setBanner(banner.toString());
		}
		const date = data.get('date');
		if (date) {
			this.setDate(new Date(date.toString()));
		}
		this.setLined(Boolean(data.get('lined') ?? false));
	}

	async printOutlineTemplate() {
		const url = new URL(OUTLINE_ENDPOINT!);
		const res = await fetch(url, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				rows: this.rows,
				date: this.date,
				banner: this.banner,
				lined: this.lined
			})
		});
		if (res.ok) {
			return { success: true, message: 'Printed template successfully!' };
		} else {
			return { success: false, message: await res.text() };
		}
	}
}

export class HabitTrackerTemplate {
	private habit: string;
	private startDate: Date;
	private endDate: Date;

	constructor(data: FormData) {
		const habit = (data.get('habit') ?? '').toString().trim();
		this.habit = habit;
		const startDate = data.get('start-date');
		if (startDate) {
			this.startDate = new Date(startDate.toString());
		} else {
			this.startDate = new Date();
		}
		const endDate = data.get('end-date');
		if (endDate) {
			this.endDate = new Date(endDate.toString());
		} else {
			this.endDate = new Date();
		}
	}

	async printHabitTrackerTemplate() {
		const url = new URL(HABITS_ENDPOINT!);
		const res = await fetch(url, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				habit: this.habit,
				start_date: this.startDate.toISOString(),
				end_date: this.endDate.toISOString()
			})
		});
		if (res.ok) {
			return { success: true, message: 'Printed template successfully!' };
		} else {
			return { success: true, message: await res.text() };
		}
	}
}
