<script lang="ts">
interface Props {
	email: string;
	onResend: () => Promise<void>;
	dismissed: boolean;
	onDismiss: () => void;
}

const { email, onResend, dismissed, onDismiss }: Props = $props();

let sending = $state(false);
let sent = $state(false);
let error = $state<string | undefined>(undefined);

async function resend(): Promise<void> {
	if (sending) return;
	sending = true;
	error = undefined;
	try {
		await onResend();
		sent = true;
	} catch (err) {
		error = err instanceof Error ? err.message : 'Could not resend';
	} finally {
		sending = false;
	}
}
</script>

{#if !dismissed}
	<aside class="banner" role="status" aria-live="polite">
		<div class="icon" aria-hidden="true">
			<svg viewBox="0 0 256 256" width="24" height="24" fill="currentColor">
				<path
					d="M224 48H32a8 8 0 0 0-8 8v136a16 16 0 0 0 16 16h176a16 16 0 0 0 16-16V56a8 8 0 0 0-8-8m-96 85.15L52.57 64h150.86ZM98.71 128 40 181.81V74.19Zm11.84 10.85 12 11.05a8 8 0 0 0 10.82 0l12-11.05 58 53.15H52.57Zm46.74-10.85L216 74.19v107.62Z"
				/>
			</svg>
		</div>
		<div class="body">
			<p class="title">Verify your email address</p>
			<p class="text">
				We sent a verification link to <strong>{email}</strong>. Click the
				link to confirm your account.
			</p>
			{#if sent}
				<p class="sent">A new link is on its way.</p>
			{/if}
			{#if error}
				<p class="err" role="alert">{error}</p>
			{/if}
		</div>
		<div class="actions">
			<button
				type="button"
				class="resend"
				onclick={resend}
				disabled={sending}
				aria-busy={sending}
			>
				{sending ? 'Sending…' : 'Resend email'}
			</button>
			<button
				type="button"
				class="dismiss"
				onclick={onDismiss}
				aria-label="Dismiss verification banner"
			>
				<svg viewBox="0 0 256 256" width="18" height="18" fill="currentColor" aria-hidden="true">
					<path d="M205.66 194.34a8 8 0 0 1-11.32 11.32L128 139.31l-66.34 66.35a8 8 0 0 1-11.32-11.32L116.69 128 50.34 61.66a8 8 0 0 1 11.32-11.32L128 116.69l66.34-66.35a8 8 0 0 1 11.32 11.32L139.31 128Z" />
				</svg>
			</button>
		</div>
	</aside>
{/if}

<style>
	.banner {
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: var(--space-sm);
		padding-block: var(--space-xs);
		padding-inline: var(--space-sm);
		background: oklch(95% 0.05 75);
		color: oklch(28% 0.08 75);
		border-block-end: 1px solid oklch(85% 0.08 75);
		container-type: inline-size;
	}
	@media (prefers-color-scheme: dark) {
		.banner {
			background: oklch(28% 0.05 75);
			color: oklch(95% 0.05 75);
			border-block-end-color: oklch(40% 0.08 75);
		}
	}
	.icon {
		display: grid;
		place-items: center;
		inline-size: 2.25rem;
		block-size: 2.25rem;
		border-radius: var(--radius-full);
		background: oklch(85% 0.1 75);
		color: oklch(30% 0.15 75);
	}
	.body { display: grid; gap: var(--space-3xs); }
	.title { font-weight: 600; }
	.text { font-size: var(--type-sm); }
	.sent { color: var(--status-success); font-size: var(--type-sm); }
	.err { color: var(--status-danger); font-size: var(--type-sm); }
	.actions { display: flex; align-items: center; gap: var(--space-2xs); }
	.resend {
		padding-block: var(--space-2xs);
		padding-inline: var(--space-sm);
		background: var(--accent-base);
		color: var(--accent-text);
		border: 0;
		border-radius: var(--radius-md);
		font-size: var(--type-sm);
		font-weight: 500;
		cursor: pointer;
		transition: background var(--motion-fast) var(--motion-ease);
	}
	.resend:hover:not(:disabled) { background: var(--accent-hover); }
	.resend:focus-visible { outline: 2px solid var(--accent-base); outline-offset: 2px; }
	.resend:disabled { opacity: 0.6; cursor: not-allowed; }
	.dismiss {
		background: transparent;
		border: 0;
		color: inherit;
		padding: var(--space-3xs);
		cursor: pointer;
		border-radius: var(--radius-sm);
		display: inline-flex;
	}
	.dismiss:hover { background: oklch(90% 0.05 75 / 0.4); }
	.dismiss:focus-visible { outline: 2px solid currentColor; outline-offset: 2px; }
	@media (prefers-reduced-motion: reduce) { .resend { transition: none; } }
	@container (inline-size < 540px) {
		.banner { grid-template-columns: auto 1fr; }
		.actions { grid-column: 2; }
	}
</style>
