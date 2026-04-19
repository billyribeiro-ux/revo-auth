<script lang="ts">
import type { Snippet } from 'svelte';
import { createForm } from '../form.svelte';
import { magicRequestSchema } from '../validation';

interface Props {
	onSubmit: (values: { email: string }) => Promise<void>;
	header?: Snippet;
	success?: Snippet<[{ email: string }]>;
}

const { onSubmit, header, success }: Props = $props();

let sent = $state(false);
let sentEmail = $state('');

const form = createForm({
	schema: magicRequestSchema,
	initial: { email: '' },
	onSubmit: async (values) => {
		await onSubmit(values);
		sentEmail = values.email;
		sent = true;
	},
});
</script>

<div class="magic">
	{#if header}{@render header()}{/if}

	{#if sent}
		{#if success}
			{@render success({ email: sentEmail })}
		{:else}
			<div class="sent" role="status">
				<h3>Check your inbox</h3>
				<p>
					We sent a sign-in link to <strong>{sentEmail}</strong>. Open it on
					this device to continue.
				</p>
			</div>
		{/if}
	{:else}
		<form {@attach form.attach} novalidate>
			<label class="field">
				<span>Email</span>
				<input
					type="email"
					autocomplete="email"
					required
					bind:value={form.values.email}
					aria-invalid={form.errors.email ? 'true' : undefined}
					aria-describedby={form.errors.email ? 'magic-email-err' : undefined}
				/>
				{#if form.errors.email}
					<em id="magic-email-err">{form.errors.email}</em>
				{/if}
			</label>
			<button type="submit" disabled={form.submitting} aria-busy={form.submitting}>
				{form.submitting ? 'Sending link…' : 'Send sign-in link'}
			</button>
		</form>
	{/if}
</div>

<style>
	.magic {
		display: grid;
		gap: var(--space-sm);
	}
	form {
		display: grid;
		gap: var(--space-sm);
	}
	.field {
		display: grid;
		gap: var(--space-3xs);
		--_field-border: oklch(85% 0.01 255);
		--_field-border-focus: var(--accent-base);

		& input {
			padding-block: var(--space-2xs);
			padding-inline: var(--space-xs);
			border: 1px solid var(--_field-border);
			border-radius: var(--radius-md);
			font-size: var(--type-base);
			background: var(--surface-raised);
			color: var(--text-primary);
			transition: border-color var(--motion-fast) var(--motion-ease);

			&:focus-visible {
				outline: 2px solid var(--_field-border-focus);
				outline-offset: 2px;
				border-color: var(--_field-border-focus);
			}
		}
		& em {
			color: var(--status-danger);
			font-size: var(--type-sm);
			font-style: normal;
		}
	}
	.sent {
		padding-block: var(--space-sm);
		padding-inline: var(--space-sm);
		background: var(--surface-sunken);
		border-radius: var(--radius-md);
		border: 1px solid oklch(88% 0.015 160);

		& h3 {
			margin-block-end: var(--space-3xs);
			color: var(--text-primary);
			font-size: var(--type-lg);
		}
		& p {
			color: var(--text-secondary);
			font-size: var(--type-base);
		}
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
		.field input,
		button[type='submit'] {
			transition: none;
		}
	}
</style>
