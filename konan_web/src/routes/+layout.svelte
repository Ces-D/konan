<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon/favicon.svg';
	import faviconIco from '$lib/assets/favicon/favicon.ico';
	import favicon96 from '$lib/assets/favicon/favicon-96x96.png';
	import appleTouchIcon from '$lib/assets/favicon/apple-touch-icon.png';
	import siteManifest from '$lib/assets/favicon/site.webmanifest';
	import { SvelteToast } from '@zerodevx/svelte-toast';
	import PrintHistory from '$lib/components/PrintHistory.svelte';

	let { children } = $props();
	// TODO: implement Confetti - https://github.com/Mitcheljager/svelte-confetti/blob/master/src/routes/ToggleConfetti.svelte
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
		</div>
	</nav>

	<main
		class="grid grid-cols-1 gap-x-3 px-1 h-full grid-rows-[auto_1fr] tablet:grid-cols-[auto_1fr] tablet:grid-rows-1 tablet:px-0"
	>
		<PrintHistory />
		{@render children()}
		<SvelteToast />
	</main>
</div>
