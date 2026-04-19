<script lang="ts">
	import type { Snippet } from 'svelte';
	import { createForm } from '../lib/form.svelte';
	import { signupSchema } from '../lib/validation';
	import PasswordField from './PasswordField.svelte';
	import PasswordStrengthMeter from './PasswordStrengthMeter.svelte';

	interface Props {
		onSubmit: (values: {
			name?: string;
			email: string;
			password: string;
			terms: true;
		}) => Promise<void>;
		termsHref?: string;
		privacyHref?: string;
		showName?: boolean;
		header?: Snippet;
		footer?: Snippet;
	}

	let {
		onSubmit,
		termsHref = '/terms',
		privacyHref = '/privacy',
		showName = true,
		header,
		footer
	}: Props = $props();

	const form = createForm({
		schema: signupSchema,
		initial: { name: '', email: '', password: '', terms: false as unknown as true },
		onSubmit: (values) => onSubmit(values)
	});
</script>

<form {@attach form.attach} novalidate>
	{#if header}{@render header()}{/if}

	{#if showName}
		<label class="field">
			<span>Name <small>(optional)</small></span>
			<input
				type="text"
				autocomplete="name"
				bind:value={form.values.name}
				aria-invalid={form.errors.name ? 'true' : undefined}
				aria-describedby={form.errors.name ? 'signup-name-err' : undefined}
			/>
			{#if form.errors.name}
				<em id="signup-name-err">{form.errors.name}</em>
			{/if}
		</label>
	{/if}

	<label class="field">
		<span>Email</span>
		<input
			type="email"
			autocomplete="email"
			required
			bind:value={form.values.email}
			aria-invalid={form.errors.email ? 'true' : undefined}
			aria-describedby={form.errors.email ? 'signup-email-err' : undefined}
		/>
		{#if form.errors.email}
			<em id="signup-email-err">{form.errors.email}</em>
		{/if}
	</label>

	<div class="password-group">
		<PasswordField
			bind:value={form.values.password}
			error={form.errors.password}
			autocomplete="new-password"
			id="signup-password"
		/>
		<PasswordStrengthMeter password={form.values.password} />
	</div>

	<label class="checkbox">
		<input
			type="checkbox"
			bind:checked={form.values.terms}
			aria-invalid={form.errors.terms ? 'true' : undefined}
			aria-describedby={form.errors.terms ? 'signup-terms-err' : undefined}
		/>
		<span>
			I agree to the <a href={termsHref}>Terms</a> and
			<a href={privacyHref}>Privacy Policy</a>.
		</span>
	</label>
	{#if form.errors.terms}
		<em id="signup-terms-err" class="err-inline">{form.errors.terms}</em>
	{/if}

	<button type="submit" disabled={form.submitting} aria-busy={form.submitting}>
		{form.submitting ? 'Creating account…' : 'Create account'}
	</button>

	{#if footer}{@render footer()}{/if}
</form>

<style>
	form {
		display: grid;
		gap: var(--space-sm);
		container-type: inline-size;
	}
	.field {
		display: grid;
		gap: var(--space-3xs);
		--_field-border: oklch(85% 0.01 255);
		--_field-border-focus: var(--accent-base);

		& span small {
			color: var(--text-muted);
			font-weight: 400;
		}
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
	.password-group {
		display: grid;
		gap: var(--space-2xs);
	}
	.checkbox {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: var(--space-2xs);
		align-items: start;
		font-size: var(--type-sm);
		color: var(--text-secondary);

		& input {
			margin-block-start: 0.2em;
			inline-size: 1rem;
			block-size: 1rem;
			accent-color: var(--accent-base);
		}
		& a {
			color: var(--accent-base);
			text-decoration: underline;
		}
	}
	.err-inline {
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
		.field input,
		button[type='submit'] {
			transition: none;
		}
	}
</style>
