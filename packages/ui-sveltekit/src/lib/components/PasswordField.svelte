<script lang="ts">
interface Props {
	value: string;
	error?: string | undefined;
	label?: string;
	id?: string;
	name?: string;
	autocomplete?: 'current-password' | 'new-password' | 'off';
	required?: boolean;
	placeholder?: string;
}

// biome-ignore lint/style/useConst: $bindable prop must be `let` for two-way binding
let {
	value = $bindable(''),
	error = undefined,
	label = 'Password',
	id = 'password',
	name = 'password',
	autocomplete = 'current-password',
	required = true,
	placeholder = '',
}: Props = $props();

let visible = $state(false);

const errorId = $derived(`${id}-err`);
const type = $derived(visible ? 'text' : 'password');

function toggleVisible(): void {
	visible = !visible;
}
</script>

<label class="field" for={id}>
	<span>{label}</span>
	<div class="wrap">
		<input
			{id}
			{name}
			{type}
			{autocomplete}
			{required}
			{placeholder}
			bind:value
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={error ? errorId : undefined}
		/>
		<button
			type="button"
			class="toggle"
			aria-label={visible ? 'Hide password' : 'Show password'}
			aria-pressed={visible}
			onclick={toggleVisible}
		>
			{#if visible}
				<svg
					viewBox="0 0 256 256"
					aria-hidden="true"
					focusable="false"
					width="20"
					height="20"
					fill="currentColor"
				>
					<path
						d="M53.92 34.62a8 8 0 1 0-11.84 10.76L58 63.08C25.22 82.23 12.05 113.6 11.44 115a8 8 0 0 0 0 6c.32.73 8 18.2 25.43 33.62C60 175.14 91.85 188 128 188a116 116 0 0 0 45.19-9l19.6 21.58a8 8 0 1 0 11.84-10.76ZM128 172c-30.78 0-57.67-11.19-79.93-33.25A133.47 133.47 0 0 1 28.12 118c4.86-7.65 14.1-20.21 28-30.95l16.31 17.95a48 48 0 0 0 63.75 70.15l14.81 16.29A99.6 99.6 0 0 1 128 172m-27.4-56.56 33 36.37A32 32 0 0 1 100.6 115.44ZM244.56 121c-.37.82-9.13 20.22-27.53 38.37a8 8 0 1 1-11.22-11.41c13.49-13.3 21-26.92 24-32.94a133.47 133.47 0 0 0-19.94-20.74C187.67 72.19 160.78 61 130 61a115.7 115.7 0 0 0-19.17 1.6A8 8 0 1 1 108 46.83 131.9 131.9 0 0 1 130 45c36.15 0 68 12.86 92.13 37.23C239.6 97.8 247.25 115.27 247.57 116a8 8 0 0 1-3.01 5m-112-17.11a8 8 0 0 1 8.17-7.83 32 32 0 0 1 31.2 31.21 8 8 0 0 1-7.82 8.17h-.18a8 8 0 0 1-8-7.83A16 16 0 0 0 141 112.05a8 8 0 0 1-8.44-8.16"
					/>
				</svg>
			{:else}
				<svg
					viewBox="0 0 256 256"
					aria-hidden="true"
					focusable="false"
					width="20"
					height="20"
					fill="currentColor"
				>
					<path
						d="M247.31 124.76c-.35-.79-8.82-19.58-27.65-38.41C194.57 61.26 162.88 48 128 48S61.43 61.26 36.34 86.35C17.51 105.18 9 124 8.69 124.76a8 8 0 0 0 0 6.5c.35.79 8.82 19.57 27.65 38.4C61.43 194.74 93.12 208 128 208s66.57-13.26 91.66-38.34c18.83-18.83 27.3-37.61 27.65-38.4a8 8 0 0 0 0-6.5M128 192c-30.78 0-57.67-11.19-79.93-33.25A133.16 133.16 0 0 1 25 128a133.2 133.2 0 0 1 23.07-30.75C70.33 75.19 97.22 64 128 64s57.67 11.19 79.93 33.25A133.5 133.5 0 0 1 231.05 128c-7.21 13.46-38.62 64-103.05 64m0-112a48 48 0 1 0 48 48 48.05 48.05 0 0 0-48-48m0 80a32 32 0 1 1 32-32 32 32 0 0 1-32 32"
					/>
				</svg>
			{/if}
		</button>
	</div>
	{#if error}
		<em id={errorId}>{error}</em>
	{/if}
</label>

<style>
	.field {
		display: grid;
		gap: var(--space-3xs);
		--_field-border: oklch(85% 0.01 255);
		--_field-border-focus: var(--accent-base);
	}
	.wrap {
		position: relative;
		display: grid;
	}
	input {
		padding-block: var(--space-2xs);
		padding-inline-start: var(--space-xs);
		padding-inline-end: calc(var(--space-xs) + 2.25rem);
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
	.toggle {
		position: absolute;
		inset-block: 0;
		inset-inline-end: var(--space-3xs);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		inline-size: 2rem;
		background: transparent;
		border: 0;
		color: var(--text-secondary);
		cursor: pointer;
		border-radius: var(--radius-sm);

		&:hover {
			color: var(--text-primary);
		}
		&:focus-visible {
			outline: 2px solid var(--accent-base);
			outline-offset: 2px;
		}
	}
	em {
		color: var(--status-danger);
		font-size: var(--type-sm);
		font-style: normal;
	}
	@media (prefers-reduced-motion: reduce) {
		input {
			transition: none;
		}
	}
</style>
