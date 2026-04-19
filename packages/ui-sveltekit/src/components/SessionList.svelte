<script lang="ts">
	export interface SessionRow {
		id: string;
		userAgent: string;
		ip: string;
		createdAt: string | Date;
		lastUsedAt: string | Date;
		current?: boolean;
	}

	interface Props {
		sessions: readonly SessionRow[];
		currentId: string;
		onRevoke: (id: string) => Promise<void>;
	}

	let { sessions, currentId, onRevoke }: Props = $props();

	const pendingIds = $state<Set<string>>(new Set());

	function toDate(value: string | Date): Date {
		return value instanceof Date ? value : new Date(value);
	}

	const dateFmt = new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short'
	});

	function formatDate(value: string | Date): string {
		return dateFmt.format(toDate(value));
	}

	async function revoke(id: string): Promise<void> {
		if (pendingIds.has(id)) return;
		pendingIds.add(id);
		try {
			await onRevoke(id);
		} finally {
			pendingIds.delete(id);
		}
	}
</script>

<section class="sessions" aria-label="Active sessions">
	<header>
		<h2>Active sessions</h2>
		<p>Devices currently signed in to your account.</p>
	</header>

	{#if sessions.length === 0}
		<p class="empty">No active sessions.</p>
	{:else}
		<ul>
			{#each sessions as s (s.id)}
				{@const isCurrent = s.id === currentId || s.current === true}
				{@const pending = pendingIds.has(s.id)}
				<li class:current={isCurrent}>
					<div class="meta">
						<p class="ua">{s.userAgent || 'Unknown device'}</p>
						<p class="details">
							<span>{s.ip || 'unknown IP'}</span>
							<span aria-hidden="true">·</span>
							<span>Started {formatDate(s.createdAt)}</span>
							<span aria-hidden="true">·</span>
							<span>Last used {formatDate(s.lastUsedAt)}</span>
						</p>
					</div>
					{#if isCurrent}
						<span class="badge">This device</span>
					{:else}
						<button
							type="button"
							class="revoke"
							onclick={() => revoke(s.id)}
							disabled={pending}
							aria-busy={pending}
						>
							{pending ? 'Revoking…' : 'Revoke'}
						</button>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.sessions {
		display: grid;
		gap: var(--space-sm);
	}
	header h2 {
		font-size: var(--type-xl);
		color: var(--text-primary);
	}
	header p {
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	ul {
		list-style: none;
		margin: 0;
		display: grid;
		gap: var(--space-2xs);
	}
	li {
		display: grid;
		grid-template-columns: 1fr auto;
		align-items: center;
		gap: var(--space-sm);
		padding-block: var(--space-xs);
		padding-inline: var(--space-sm);
		background: var(--surface-raised);
		border: 1px solid oklch(88% 0.005 255);
		border-radius: var(--radius-md);
	}
	li.current {
		border-color: var(--accent-base);
	}
	.ua {
		font-weight: 500;
		color: var(--text-primary);
		overflow-wrap: anywhere;
	}
	.details {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-3xs);
		color: var(--text-muted);
		font-size: var(--type-sm);
	}
	.badge {
		padding-block: var(--space-3xs);
		padding-inline: var(--space-2xs);
		background: oklch(92% 0.05 259);
		color: var(--accent-base);
		border-radius: var(--radius-full);
		font-size: var(--type-sm);
		font-weight: 500;
	}
	.revoke {
		padding-block: var(--space-2xs);
		padding-inline: var(--space-sm);
		background: transparent;
		color: var(--status-danger);
		border: 1px solid var(--status-danger);
		border-radius: var(--radius-md);
		font-size: var(--type-sm);
		font-weight: 500;
		cursor: pointer;
		transition:
			background var(--motion-fast) var(--motion-ease),
			color var(--motion-fast) var(--motion-ease);
	}
	.revoke:hover:not(:disabled) {
		background: var(--status-danger);
		color: oklch(100% 0 0);
	}
	.revoke:focus-visible {
		outline: 2px solid var(--status-danger);
		outline-offset: 2px;
	}
	.revoke:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
	.empty {
		color: var(--text-muted);
		font-style: italic;
	}
	@media (prefers-reduced-motion: reduce) {
		.revoke {
			transition: none;
		}
	}
</style>
