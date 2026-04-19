<script lang="ts">
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	const { data }: Props = $props();
	const session = $derived(data.session);
</script>

<section class="hero">
	<h1>Revo-Auth end-to-end demo</h1>
	<p>
		A minimal SvelteKit app that exercises every feature of <code>@revo-auth/sdk-sveltekit</code>
		and <code>@revo-auth/ui-sveltekit</code>: signup, signin, magic links, passkeys, TOTP, OAuth
		linking, session management, and email verification.
	</p>

	{#if session}
		<p class="welcome">
			Signed in as
			<strong>{session.user.email ?? session.user.id}</strong>.
		</p>
		<div class="cta">
			<a class="btn primary" href="/dashboard">Open dashboard</a>
			<a class="btn" href="/dashboard/settings">Manage account</a>
		</div>
	{:else}
		<div class="cta">
			<a class="btn primary" href="/signup">Create an account</a>
			<a class="btn" href="/login">Sign in</a>
		</div>
	{/if}
</section>

<style>
	.hero {
		max-inline-size: 60ch;
		margin-inline: auto;
		padding-block: var(--space-lg);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	h1 {
		font-size: var(--type-xl);
		margin: 0;
	}

	p {
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.5;
	}

	.welcome {
		color: var(--text-primary);
	}

	.cta {
		display: flex;
		gap: var(--space-xs);
		flex-wrap: wrap;
		margin-block-start: var(--space-xs);
	}

	.btn {
		display: inline-flex;
		align-items: center;
		padding-block: var(--space-2xs);
		padding-inline: var(--space-sm);
		border-radius: var(--radius-sm);
		border: 1px solid var(--surface-sunken);
		color: var(--text-primary);
		text-decoration: none;
		background: var(--surface-raised);
	}

	.primary {
		background: var(--accent-base);
		color: var(--accent-text);
		border-color: transparent;
	}

	code {
		background: var(--surface-sunken);
		padding-inline: var(--space-3xs);
		border-radius: var(--radius-sm);
		font-size: 0.95em;
	}
</style>
