<script lang="ts">
interface Props {
	password: string;
}

const { password }: Props = $props();

function scorePassword(pw: string): number {
	if (!pw) return 0;
	let score = 0;
	const length = pw.length;
	if (length >= 8) score += 1;
	if (length >= 12) score += 1;

	const hasLower = /[a-z]/.test(pw);
	const hasUpper = /[A-Z]/.test(pw);
	const hasDigit = /\d/.test(pw);
	const hasSymbol = /[^A-Za-z0-9]/.test(pw);
	const variety =
		Number(hasLower) + Number(hasUpper) + Number(hasDigit) + Number(hasSymbol);

	if (variety >= 2) score += 1;
	if (variety >= 3 && length >= 10) score += 1;

	if (/(.)\1{2,}/.test(pw)) score -= 1;
	if (length < 6) score = 0;

	return Math.max(0, Math.min(4, score));
}

const score = $derived(scorePassword(password));
const labels = ['Too short', 'Weak', 'Fair', 'Good', 'Strong'] as const;
const label = $derived(labels[score] ?? 'Too short');
</script>

<div class="meter" role="group" aria-label="Password strength">
	<div class="bars" aria-hidden="true">
		{#each [0, 1, 2, 3] as i (i)}
			<span class="bar" data-filled={i < score}></span>
		{/each}
	</div>
	<p class="label" aria-live="polite" data-score={score}>{label}</p>
</div>

<style>
	.meter {
		display: grid;
		gap: var(--space-3xs);
	}
	.bars {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: var(--space-3xs);
	}
	.bar {
		block-size: 0.375rem;
		border-radius: var(--radius-full);
		background: oklch(90% 0.005 255);
		transition: background var(--motion-fast) var(--motion-ease);
	}
	.bar[data-filled='true'] {
		background: var(--_fill, var(--status-danger));
	}
	.label {
		font-size: var(--type-sm);
		color: var(--text-secondary);
		margin: 0;
	}
	.label[data-score='0'] {
		color: var(--text-muted);
	}
	.label[data-score='1'] {
		color: var(--status-danger);
	}
	.label[data-score='2'] {
		color: var(--status-warning);
	}
	.label[data-score='3'] {
		color: oklch(70% 0.15 140);
	}
	.label[data-score='4'] {
		color: var(--status-success);
	}
	.meter:has(.label[data-score='1']) {
		--_fill: var(--status-danger);
	}
	.meter:has(.label[data-score='2']) {
		--_fill: var(--status-warning);
	}
	.meter:has(.label[data-score='3']) {
		--_fill: oklch(70% 0.15 140);
	}
	.meter:has(.label[data-score='4']) {
		--_fill: var(--status-success);
	}
	@media (prefers-reduced-motion: reduce) {
		.bar {
			transition: none;
		}
	}
</style>
