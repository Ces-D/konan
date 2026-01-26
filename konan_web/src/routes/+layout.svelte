<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon/favicon.svg';
	import faviconIco from '$lib/assets/favicon/favicon.ico';
	import favicon96 from '$lib/assets/favicon/favicon-96x96.png';
	import appleTouchIcon from '$lib/assets/favicon/apple-touch-icon.png';
	import siteManifest from '$lib/assets/favicon/site.webmanifest';
	import {
		SvelteToast,
		type SvelteToastOptions
	} from '@zerodevx/svelte-toast';

	let { children } = $props();
	const toastOptions: SvelteToastOptions = {
		dismissable: true,
		duration: 500,
		theme: {
			'--toastBackground': 'var(--color-primary-900)',
			'--toastColor': 'var(--text-color-inverted)',
			'--toastBarHeight': 0
		}
	};
</script>

<svelte:head>
	<link rel="icon" type="image/png" href={favicon96} sizes="96x96" />
	<link rel="icon" type="image/svg+xml" href={favicon} />
	<link rel="shortcut icon" href={faviconIco} />
	<link rel="apple-touch-icon" sizes="180x180" href={appleTouchIcon} />
	<meta name="apple-mobile-web-app-title" content="Konan" />
	<link rel="manifest" href={siteManifest} />
</svelte:head>

<div class="grid grid-cols-1 gap-2 h-screen grid-rows-[auto_1fr]">
	<nav class="px-2 shadow-sm shadow-background-inverted">
		<div class="grid grid-cols-2 items-center">
			<nav class="flex gap-4 justify-start items-center">
				<img alt="Logo" class="w-12 h-12" src={appleTouchIcon} />
				<h1>Konan</h1>
			</nav>
			<div class="text-right">Login,Logout</div>
		</div>
	</nav>
	{@render children()}
	<SvelteToast {...toastOptions} />
</div>
