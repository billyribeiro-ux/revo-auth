<script lang="ts">
interface Props {
	onSubmit: (code: string) => Promise<void>;
	autoSubmit?: boolean;
	error?: string | undefined;
	label?: string;
}

const {
	onSubmit,
	autoSubmit = true,
	error = undefined,
	label = 'Enter the 6-digit code from your authenticator app',
}: Props = $props();

const digits = $state<string[]>(['', '', '', '', '', '']);
let submitting = $state(false);
let localError = $state<string | undefined>(undefined);

const combined = $derived(digits.join(''));
const complete = $derived(combined.length === 6);
const displayError = $derived(error ?? localError);

const inputs: HTMLInputElement[] = [];

function registerInput(index: number) {
	return (node: HTMLInputElement) => {
		inputs[index] = node;
		return () => {
			if (inputs[index] === node) {
				inputs.length = Math.max(inputs.length, 0);
			}
		};
	};
}

function sanitize(ch: string): string {
	return ch.replace(/\D/g, '').slice(0, 1);
}

function onInput(index: number, event: Event): void {
	const target = event.target as HTMLInputElement;
	const cleaned = sanitize(target.value);
	digits[index] = cleaned;
	target.value = cleaned;

	if (cleaned && index < 5) {
		inputs[index + 1]?.focus();
	}
	if (autoSubmit && digits.every((d) => d !== '')) {
		void submit();
	}
}

function onKey(index: number, event: KeyboardEvent): void {
	if (event.key === 'Backspace' && !digits[index] && index > 0) {
		event.preventDefault();
		inputs[index - 1]?.focus();
		digits[index - 1] = '';
	} else if (event.key === 'ArrowLeft' && index > 0) {
		event.preventDefault();
		inputs[index - 1]?.focus();
	} else if (event.key === 'ArrowRight' && index < 5) {
		event.preventDefault();
		inputs[index + 1]?.focus();
	} else if (event.key === 'Enter') {
		event.preventDefault();
		void submit();
	}
}

function onPaste(event: ClipboardEvent): void {
	const pasted = event.clipboardData?.getData('text') ?? '';
	const cleaned = pasted.replace(/\D/g, '').slice(0, 6);
	if (!cleaned) return;
	event.preventDefault();
	for (let i = 0; i < 6; i += 1) {
		digits[i] = cleaned[i] ?? '';
	}
	const nextIndex = Math.min(cleaned.length, 5);
	inputs[nextIndex]?.focus();
	if (autoSubmit && cleaned.length === 6) {
		void submit();
	}
}

async function submit(): Promise<void> {
	if (!complete || submitting) return;
	submitting = true;
	localError = undefined;
	try {
		await onSubmit(combined);
	} catch (err) {
		localError = err instanceof Error ? err.message : 'Invalid code';
	} finally {
		submitting = false;
	}
}

function attachForm(form: HTMLFormElement): () => void {
	const handler = (event: SubmitEvent): void => {
		event.preventDefault();
		void submit();
	};
	form.addEventListener('submit', handler);
	return () => form.removeEventListener('submit', handler);
}
</script>

<form {@attach attachForm} novalidate>
	<fieldset>
		<legend>{label}</legend>
		<div class="boxes">
			{#each digits as _d, i (i)}
				<input
					{@attach registerInput(i)}
					value={digits[i]}
					type="text"
					inputmode="numeric"
					autocomplete={i === 0 ? 'one-time-code' : 'off'}
					maxlength="1"
					aria-label={`Digit ${i + 1}`}
					aria-invalid={displayError ? 'true' : undefined}
					oninput={(e) => onInput(i, e)}
					onkeydown={(e) => onKey(i, e)}
					onpaste={onPaste}
				/>
			{/each}
		</div>
		{#if displayError}
			<em role="alert">{displayError}</em>
		{/if}
	</fieldset>
	<button type="submit" disabled={!complete || submitting} aria-busy={submitting}>
		{submitting ? 'Verifying…' : 'Verify'}
	</button>
</form>

<style>
	form {
		display: grid;
		gap: var(--space-sm);
	}
	fieldset {
		border: 0;
		padding: 0;
		margin: 0;
		display: grid;
		gap: var(--space-2xs);
	}
	legend {
		font-size: var(--type-sm);
		color: var(--text-secondary);
		margin-block-end: var(--space-2xs);
	}
	.boxes {
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: var(--space-2xs);
	}
	input {
		inline-size: 100%;
		padding-block: var(--space-xs);
		text-align: center;
		font-size: var(--type-lg);
		font-variant-numeric: tabular-nums;
		border: 1px solid oklch(85% 0.01 255);
		border-radius: var(--radius-md);
		background: var(--surface-raised);
		color: var(--text-primary);
		transition: border-color var(--motion-fast) var(--motion-ease);

		&:focus-visible {
			outline: 2px solid var(--accent-base);
			outline-offset: 2px;
			border-color: var(--accent-base);
		}
	}
	em {
		color: var(--status-danger);
		font-size: var(--type-sm);
		font-style: normal;
	}
	button[type='submit'] {
		padding-block: var(--space-xs);
		padding-inline: var(--space-md);
		background: var(--accent-base);
		color: var(--accent-text);
		border: 0;
		border-radius: var(--radius-md);
		font-size: var(--type-base);
		font-weight: 600;
		cursor: pointer;
		transition: background var(--motion-fast) var(--motion-ease);

		&:hover:not(:disabled) {
			background: var(--accent-hover);
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
	@media (prefers-reduced-motion: reduce) {
		input,
		button[type='submit'] {
			transition: none;
		}
	}
</style>
