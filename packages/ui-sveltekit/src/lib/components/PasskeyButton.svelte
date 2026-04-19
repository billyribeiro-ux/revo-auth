<script lang="ts">
type Mode = 'register' | 'authenticate';

interface Props {
	mode: Mode;
	onClick?: () => void | Promise<void>;
	disabled?: boolean;
}

const { mode, onClick, disabled = false }: Props = $props();

const label = $derived(
	mode === 'register' ? 'Create a passkey' : 'Sign in with passkey',
);

let busy = $state(false);

async function handleClick(): Promise<void> {
	if (!onClick || busy || disabled) return;
	busy = true;
	try {
		await onClick();
	} finally {
		busy = false;
	}
}
</script>

<button
	type="button"
	class="passkey-btn"
	disabled={disabled || busy}
	aria-busy={busy}
	onclick={handleClick}
>
	<span class="icon" aria-hidden="true">
		<svg viewBox="0 0 256 256" width="20" height="20" fill="currentColor">
			<path
				d="M216 112a56 56 0 1 0-96 38.9V208a8 8 0 0 0 4.4 7.2l24 12a8 8 0 0 0 7.4-.3l24-13.7A8 8 0 0 0 184 206.3V150.9A56 56 0 0 0 216 112m-56 40a40 40 0 1 1 40-40 40 40 0 0 1-40 40m8-20a12 12 0 1 1-12-12 12 12 0 0 1 12 12"
			/>
		</svg>
	</span>
	<span>{busy ? 'Working…' : label}</span>
</button>

<style>
	.passkey-btn {
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
			border-color: var(--accent-base);
			color: var(--accent-base);
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
	}
	@media (prefers-reduced-motion: reduce) {
		.passkey-btn {
			transition: none;
		}
	}
</style>
