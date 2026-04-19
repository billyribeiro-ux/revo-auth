<script lang="ts">
type Provider = 'google' | 'github' | 'microsoft' | 'discord' | 'apple';

export interface LinkedAccount {
	provider: Provider;
	identifier: string;
	linkedAt?: string | Date;
}

interface Props {
	providers: readonly Provider[];
	linked: readonly LinkedAccount[];
	onLink: (provider: Provider) => Promise<void>;
	onUnlink: (provider: Provider) => Promise<void>;
}

const { providers, linked, onLink, onUnlink }: Props = $props();

const pendingProvider = $state<Set<Provider>>(new Set());

const providerLabels: Readonly<Record<Provider, string>> = {
	google: 'Google',
	github: 'GitHub',
	microsoft: 'Microsoft',
	discord: 'Discord',
	apple: 'Apple',
};

function linkOf(provider: Provider): LinkedAccount | undefined {
	return linked.find((l) => l.provider === provider);
}

async function handle(provider: Provider, linkedNow: boolean): Promise<void> {
	if (pendingProvider.has(provider)) return;
	pendingProvider.add(provider);
	try {
		if (linkedNow) {
			await onUnlink(provider);
		} else {
			await onLink(provider);
		}
	} finally {
		pendingProvider.delete(provider);
	}
}
</script>

<section class="linking" aria-label="Connected accounts">
	<header>
		<h2>Connected accounts</h2>
		<p>Link providers you use to sign in or verify your identity.</p>
	</header>
	<ul>
		{#each providers as provider (provider)}
			{@const account = linkOf(provider)}
			{@const isLinked = account !== undefined}
			{@const pending = pendingProvider.has(provider)}
			<li>
				<div class="meta">
					<p class="name">{providerLabels[provider]}</p>
					{#if isLinked}
						<p class="id">{account?.identifier}</p>
					{:else}
						<p class="id muted">Not connected</p>
					{/if}
				</div>
				<button
					type="button"
					class:unlink={isLinked}
					class:link={!isLinked}
					onclick={() => handle(provider, isLinked)}
					disabled={pending}
					aria-busy={pending}
				>
					{#if pending}
						Working…
					{:else if isLinked}
						Unlink
					{:else}
						Connect
					{/if}
				</button>
			</li>
		{/each}
	</ul>
</section>

<style>
	.linking { display: grid; gap: var(--space-sm); }
	header h2 { font-size: var(--type-xl); color: var(--text-primary); }
	header p { color: var(--text-secondary); font-size: var(--type-sm); }
	ul { list-style: none; margin: 0; display: grid; gap: var(--space-2xs); }
	li { display: grid; grid-template-columns: 1fr auto; align-items: center; gap: var(--space-sm); padding-block: var(--space-xs); padding-inline: var(--space-sm); background: var(--surface-raised); border: 1px solid oklch(88% 0.005 255); border-radius: var(--radius-md); }
	.meta .name { font-weight: 500; color: var(--text-primary); }
	.meta .id { color: var(--text-secondary); font-size: var(--type-sm); }
	.meta .id.muted { color: var(--text-muted); }
	button { padding-block: var(--space-2xs); padding-inline: var(--space-sm); border-radius: var(--radius-md); font-size: var(--type-sm); font-weight: 500; cursor: pointer; transition: background var(--motion-fast) var(--motion-ease), color var(--motion-fast) var(--motion-ease); }
	button.link { background: var(--accent-base); color: var(--accent-text); border: 0; }
	button.link:hover:not(:disabled) { background: var(--accent-hover); }
	button.unlink { background: transparent; color: var(--status-danger); border: 1px solid var(--status-danger); }
	button.unlink:hover:not(:disabled) { background: var(--status-danger); color: oklch(100% 0 0); }
	button:focus-visible { outline: 2px solid var(--accent-base); outline-offset: 2px; }
	button:disabled { opacity: 0.6; cursor: not-allowed; }
	@media (prefers-reduced-motion: reduce) { button { transition: none; } }
</style>
