<script lang="ts">
	import MfaChallenge from './MfaChallenge.svelte';

	interface Props {
		qrUri: string;
		secret: string;
		recoveryCodes: readonly string[];
		onConfirm: (code: string) => Promise<void>;
		issuer?: string;
	}

	let {
		qrUri,
		secret,
		recoveryCodes,
		onConfirm,
		issuer = 'Revo Auth'
	}: Props = $props();

	let secretVisible = $state(false);
	let copied = $state(false);

	function toggleSecret(): void {
		secretVisible = !secretVisible;
	}

	async function copyCodes(): Promise<void> {
		const text = recoveryCodes.join('\n');
		try {
			await navigator.clipboard.writeText(text);
			copied = true;
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch {
			copied = false;
		}
	}

	const maskedSecret = $derived(
		secretVisible ? secret : secret.replace(/./g, '•')
	);
</script>

<section class="mfa" aria-label={`Set up two-factor authentication for ${issuer}`}>
	<header>
		<h2>Turn on two-factor authentication</h2>
		<p>
			Scan the QR code with your authenticator app, then confirm the six-digit
			code to finish setup.
		</p>
	</header>

	<div class="grid">
		<figure class="qr">
			{#if qrUri.startsWith('data:')}
				<img src={qrUri} alt="Authenticator QR code" width="192" height="192" />
			{:else}
				<div class="qr-fallback">
					<a href={qrUri} rel="noopener noreferrer">Open setup link</a>
				</div>
			{/if}
			<figcaption>Scan with Authy, 1Password, Google Authenticator, etc.</figcaption>
		</figure>

		<div class="manual">
			<h3>Can't scan?</h3>
			<p>Enter this key manually in your authenticator app:</p>
			<div class="secret">
				<code aria-live="polite">{maskedSecret}</code>
				<button
					type="button"
					class="link-btn"
					onclick={toggleSecret}
					aria-pressed={secretVisible}
				>
					{secretVisible ? 'Hide' : 'Show'}
				</button>
			</div>
		</div>
	</div>

	<MfaChallenge
		onSubmit={(code) => onConfirm(code)}
		label="Enter the 6-digit code to confirm"
	/>

	{#if recoveryCodes.length > 0}
		<div class="recovery">
			<div class="recovery-head">
				<h3>Recovery codes</h3>
				<button
					type="button"
					class="btn-copy"
					onclick={copyCodes}
					aria-live="polite"
				>
					{copied ? 'Copied!' : 'Copy codes'}
				</button>
			</div>
			<p class="hint">
				Store these one-time codes somewhere safe. Each can be used once if
				you lose access to your authenticator.
			</p>
			<ul>
				{#each recoveryCodes as code (code)}
					<li><code>{code}</code></li>
				{/each}
			</ul>
		</div>
	{/if}
</section>

<style>
	.mfa {
		display: grid;
		gap: var(--space-md);
		container-type: inline-size;
	}
	header h2 {
		font-size: var(--type-xl);
		color: var(--text-primary);
		margin-block-end: var(--space-2xs);
	}
	header p {
		color: var(--text-secondary);
		font-size: var(--type-base);
	}
	.grid {
		display: grid;
		gap: var(--space-sm);
		grid-template-columns: 1fr;
	}
	@container (inline-size > 560px) {
		.grid {
			grid-template-columns: auto 1fr;
			align-items: start;
		}
	}
	.qr {
		display: grid;
		gap: var(--space-2xs);
		justify-items: center;
		padding-block: var(--space-sm);
		padding-inline: var(--space-sm);
		background: var(--surface-raised);
		border: 1px solid oklch(88% 0.005 255);
		border-radius: var(--radius-md);
		margin: 0;
	}
	.qr img {
		inline-size: 12rem;
		block-size: 12rem;
		border-radius: var(--radius-sm);
	}
	.qr figcaption {
		color: var(--text-muted);
		font-size: var(--type-sm);
		text-align: center;
	}
	.qr-fallback {
		display: grid;
		place-items: center;
		inline-size: 12rem;
		block-size: 12rem;
	}
	.manual {
		display: grid;
		gap: var(--space-2xs);
	}
	.manual h3 {
		font-size: var(--type-lg);
	}
	.secret {
		display: flex;
		align-items: center;
		gap: var(--space-2xs);
		padding-block: var(--space-2xs);
		padding-inline: var(--space-xs);
		background: var(--surface-sunken);
		border: 1px solid oklch(88% 0.005 255);
		border-radius: var(--radius-md);
		font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
	}
	.secret code {
		flex: 1;
		letter-spacing: 0.05em;
		word-break: break-all;
	}
	.link-btn {
		background: transparent;
		border: 0;
		color: var(--accent-base);
		cursor: pointer;
		font-weight: 500;
	}
	.link-btn:focus-visible {
		outline: 2px solid var(--accent-base);
		outline-offset: 2px;
		border-radius: var(--radius-sm);
	}
	.recovery {
		display: grid;
		gap: var(--space-2xs);
		padding-block: var(--space-sm);
		padding-inline: var(--space-sm);
		background: var(--surface-sunken);
		border: 1px solid oklch(88% 0.005 255);
		border-radius: var(--radius-md);
	}
	.recovery-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2xs);
	}
	.hint {
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	.recovery ul {
		list-style: none;
		margin: 0;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
		gap: var(--space-2xs);
	}
	.recovery li code {
		display: inline-block;
		padding-block: var(--space-3xs);
		padding-inline: var(--space-2xs);
		background: var(--surface-raised);
		border-radius: var(--radius-sm);
		font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
		font-size: var(--type-sm);
	}
	.btn-copy {
		padding-block: var(--space-3xs);
		padding-inline: var(--space-xs);
		background: var(--surface-raised);
		color: var(--text-primary);
		border: 1px solid oklch(85% 0.01 255);
		border-radius: var(--radius-sm);
		font-size: var(--type-sm);
		cursor: pointer;
	}
	.btn-copy:hover {
		background: var(--surface-sunken);
	}
	.btn-copy:focus-visible {
		outline: 2px solid var(--accent-base);
		outline-offset: 2px;
	}
</style>
