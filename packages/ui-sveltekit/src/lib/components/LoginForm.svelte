<script lang="ts">
import type { Snippet } from 'svelte';
import { createForm } from '../form.svelte';
import { loginSchema } from '../validation';
import OAuthButton from './OAuthButton.svelte';
import PasswordField from './PasswordField.svelte';

type OAuthProvider = 'google' | 'github' | 'microsoft' | 'discord' | 'apple';

interface Props {
	providers?: readonly OAuthProvider[];
	onSubmit: (values: { email: string; password: string }) => Promise<void>;
	onOAuth?: (provider: OAuthProvider) => void | Promise<void>;
	header?: Snippet;
	footer?: Snippet;
}
const { providers = [], onSubmit, onOAuth, header, footer }: Props = $props();

const form = createForm({
	schema: loginSchema,
	initial: { email: '', password: '' },
	onSubmit: (values) => onSubmit(values),
});
</script>

<form {@attach form.attach} novalidate>
	{#if header}{@render header()}{/if}

	{#if providers.length > 0}
		<div class="oauth-grid">
			{#each providers as provider (provider)}
				{#if onOAuth}
					<OAuthButton {provider} onClick={onOAuth} />
				{:else}
					<OAuthButton {provider} />
				{/if}
			{/each}
		</div>
		<div class="divider" role="separator">or</div>
	{/if}

	<label class="field">
		<span>Email</span>
		<input
			type="email"
			autocomplete="email"
			required
			bind:value={form.values.email}
			aria-invalid={form.errors.email ? 'true' : undefined}
			aria-describedby={form.errors.email ? 'email-err' : undefined}
		/>
		{#if form.errors.email}
			<em id="email-err">{form.errors.email}</em>
		{/if}
	</label>

	<PasswordField
		bind:value={form.values.password}
		error={form.errors.password}
		autocomplete="current-password"
	/>

	<button
		type="submit"
		disabled={form.submitting}
		aria-busy={form.submitting}
	>
		{form.submitting ? 'Signing in…' : 'Sign in'}
	</button>

	{#if footer}{@render footer()}{/if}
</form>

<style>
	form {
		display: grid;
		gap: var(--space-sm);
		container-type: inline-size;
	}
	.oauth-grid {
		display: grid;
		gap: var(--space-2xs);
		grid-template-columns: 1fr;
	}
	@container (inline-size > 380px) {
		.oauth-grid {
			grid-template-columns: repeat(2, 1fr);
		}
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
	.divider {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		gap: var(--space-xs);
		color: var(--text-muted);
		font-size: var(--type-sm);
		&::before,
		&::after {
			content: '';
			block-size: 1px;
			background: oklch(90% 0.005 255);
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
