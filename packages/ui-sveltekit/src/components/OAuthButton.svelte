<script lang="ts">
	type Provider = 'google' | 'github' | 'microsoft' | 'discord' | 'apple';

	interface Props {
		provider: Provider;
		onClick?: (provider: Provider) => void | Promise<void>;
		disabled?: boolean;
	}

	let { provider, onClick, disabled = false }: Props = $props();

	const labels: Readonly<Record<Provider, string>> = {
		google: 'Google',
		github: 'GitHub',
		microsoft: 'Microsoft',
		discord: 'Discord',
		apple: 'Apple'
	};

	const label = $derived(labels[provider]);

	let busy = $state(false);

	async function handleClick(): Promise<void> {
		if (!onClick || busy || disabled) return;
		busy = true;
		try {
			await onClick(provider);
		} finally {
			busy = false;
		}
	}
</script>

<button
	type="button"
	class="oauth-btn"
	data-provider={provider}
	disabled={disabled || busy}
	aria-busy={busy}
	onclick={handleClick}
>
	<span class="icon" aria-hidden="true">
		{#if provider === 'google'}
			<svg viewBox="0 0 24 24" width="20" height="20">
				<path
					fill="#4285F4"
					d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.76h3.56c2.08-1.92 3.28-4.74 3.28-8.09Z"
				/>
				<path
					fill="#34A853"
					d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.56-2.76c-.99.66-2.25 1.05-3.72 1.05-2.86 0-5.28-1.93-6.15-4.53H2.18v2.84A11 11 0 0 0 12 23Z"
				/>
				<path
					fill="#FBBC05"
					d="M5.85 14.1A6.6 6.6 0 0 1 5.5 12c0-.73.13-1.44.35-2.1V7.07H2.18A11 11 0 0 0 1 12c0 1.77.42 3.45 1.18 4.93l3.67-2.84Z"
				/>
				<path
					fill="#EA4335"
					d="M12 5.38c1.62 0 3.06.56 4.2 1.64l3.15-3.15C17.45 2.08 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.67 2.83C6.72 7.3 9.14 5.38 12 5.38Z"
				/>
			</svg>
		{:else if provider === 'github'}
			<svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
				<path
					d="M12 .5C5.65.5.5 5.65.5 12a11.5 11.5 0 0 0 7.86 10.92c.57.1.78-.25.78-.55v-2c-3.2.7-3.88-1.37-3.88-1.37-.52-1.32-1.27-1.67-1.27-1.67-1.04-.71.08-.7.08-.7 1.15.08 1.75 1.18 1.75 1.18 1.02 1.75 2.67 1.24 3.32.95.1-.74.4-1.24.73-1.52-2.56-.3-5.25-1.28-5.25-5.69 0-1.26.45-2.28 1.18-3.08-.12-.3-.51-1.47.11-3.06 0 0 .97-.3 3.17 1.18a11 11 0 0 1 5.78 0c2.2-1.48 3.17-1.18 3.17-1.18.63 1.59.24 2.76.12 3.06.74.8 1.18 1.82 1.18 3.08 0 4.42-2.69 5.39-5.26 5.68.42.35.78 1.04.78 2.1v3.11c0 .31.21.67.79.55A11.5 11.5 0 0 0 23.5 12C23.5 5.65 18.35.5 12 .5Z"
				/>
			</svg>
		{:else if provider === 'microsoft'}
			<svg viewBox="0 0 24 24" width="20" height="20">
				<rect x="1" y="1" width="10" height="10" fill="#F25022" />
				<rect x="13" y="1" width="10" height="10" fill="#7FBA00" />
				<rect x="1" y="13" width="10" height="10" fill="#00A4EF" />
				<rect x="13" y="13" width="10" height="10" fill="#FFB900" />
			</svg>
		{:else if provider === 'discord'}
			<svg viewBox="0 0 24 24" width="20" height="20" fill="#5865F2">
				<path
					d="M20.3 4.37A19.8 19.8 0 0 0 15.4 3l-.25.47c1.66.4 2.96 1 4.32 1.87a16.3 16.3 0 0 0-14.94 0c1.36-.87 2.66-1.47 4.32-1.87L8.6 3a19.8 19.8 0 0 0-4.9 1.37A20.3 20.3 0 0 0 .5 16.8a20 20 0 0 0 6.14 3.1l1.23-1.68a12.7 12.7 0 0 1-2.02-.98c.17-.12.33-.25.49-.38a14.3 14.3 0 0 0 13.33 0c.15.13.32.26.49.38-.64.38-1.32.71-2.02.98l1.23 1.68a20 20 0 0 0 6.13-3.1 20.3 20.3 0 0 0-3.2-12.43ZM8.52 14.39c-1.18 0-2.15-1.1-2.15-2.45 0-1.35.95-2.46 2.15-2.46 1.2 0 2.17 1.1 2.15 2.46 0 1.35-.96 2.45-2.15 2.45Zm6.96 0c-1.18 0-2.15-1.1-2.15-2.45 0-1.35.95-2.46 2.15-2.46 1.2 0 2.16 1.1 2.15 2.46 0 1.35-.95 2.45-2.15 2.45Z"
				/>
			</svg>
		{:else if provider === 'apple'}
			<svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
				<path
					d="M17.05 12.54a4.4 4.4 0 0 1 2.1-3.7 4.5 4.5 0 0 0-3.55-1.92c-1.5-.16-2.94.89-3.7.89-.78 0-1.95-.87-3.2-.85-1.65.03-3.17.96-4.02 2.44-1.71 2.97-.44 7.37 1.22 9.79.81 1.18 1.77 2.5 3.02 2.46 1.22-.05 1.68-.79 3.15-.79 1.47 0 1.88.79 3.17.76 1.31-.02 2.14-1.19 2.94-2.38.63-.92 1.12-1.94 1.43-3.04a4.25 4.25 0 0 1-2.56-3.66ZM14.8 5.3a4.1 4.1 0 0 0 .94-3A4.2 4.2 0 0 0 13 3.75a3.9 3.9 0 0 0-.96 2.89 3.5 3.5 0 0 0 2.76-1.34Z"
				/>
			</svg>
		{/if}
	</span>
	<span class="label">Continue with {label}</span>
</button>

<style>
	.oauth-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2xs);
		inline-size: 100%;
		padding-block: var(--space-xs);
		padding-inline: var(--space-sm);
		background: var(--surface-raised);
		color: var(--text-primary);
		border: 1px solid oklch(85% 0.01 255);
		border-radius: var(--radius-md);
		font-size: var(--type-base);
		font-weight: 500;
		cursor: pointer;
		transition:
			background var(--motion-fast) var(--motion-ease),
			border-color var(--motion-fast) var(--motion-ease);

		&:hover:not(:disabled) {
			background: var(--surface-sunken);
			border-color: oklch(78% 0.015 255);
		}
		&:focus-visible {
			outline: 2px solid var(--accent-base);
			outline-offset: 2px;
		}
		&:disabled {
			opacity: 0.6;
			cursor: not-allowed;
		}
	}
	.icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		inline-size: 1.25rem;
		block-size: 1.25rem;
	}
	.label {
		font-weight: 500;
	}
	@media (prefers-reduced-motion: reduce) {
		.oauth-btn {
			transition: none;
		}
	}
</style>
