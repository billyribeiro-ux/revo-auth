<script lang="ts">
	import '../app.css';
	import { createSessionStore } from '@revo-auth/sdk-sveltekit/client';
	import { authClient } from '$lib/auth/client';
	import { goto } from '$app/navigation';
	import { untrack, type Snippet } from 'svelte';
	import type { LayoutData } from './$types';

	interface Props {
		data: LayoutData;
		children: Snippet;
	}

	const { data, children }: Props = $props();
	const client = authClient();
	// Seed the store with the server-loaded session (read once, untracked).
	const session = createSessionStore(
		client,
		untrack(() => data.session)
	);

	// Keep the store in sync when SvelteKit re-runs load on navigation.
	$effect(() => {
		const next = data.session;
		untrack(() => {
			if (session.current?.id !== next?.id) void session.refresh();
		});
	});

	async function handleSignout() {
		await client.signout();
		await session.refresh();
		await goto('/');
	}

	async function resendVerification() {
		await client.emailVerifyRequest();
	}
</script>

<header class="topbar">
	<a class="brand" href="/">Revo-Auth Demo</a>
	<nav aria-label="Primary">
		{#if session.current}
			<a href="/dashboard">Dashboard</a>
			<a href="/dashboard/settings">Settings</a>
			<span class="user" aria-live="polite">
				{session.current.user.email ?? session.current.user.id}
			</span>
			<button type="button" class="link" onclick={handleSignout}>Sign out</button>
		{:else}
			<a href="/login">Sign in</a>
			<a href="/signup" class="primary">Create account</a>
		{/if}
	</nav>
</header>

{#if session.current && !session.current.user.emailVerified}
	<div class="verify-banner" role="status">
		<span>Please verify your email address ({session.current.user.email}).</span>
		<button type="button" class="link" onclick={resendVerification}>Resend link</button>
	</div>
{/if}

<main>
	{@render children()}
</main>

<style>
	.topbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-sm);
		padding-block: var(--space-xs);
		padding-inline: var(--space-md);
		border-block-end: 1px solid var(--surface-sunken);
		background: var(--surface-raised);
	}

	.brand {
		font-weight: 700;
		font-size: var(--type-lg);
		color: var(--text-primary);
		text-decoration: none;
	}

	nav {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	nav a {
		color: var(--text-secondary);
		text-decoration: none;
	}

	nav a:hover {
		color: var(--text-primary);
	}

	.user {
		color: var(--text-muted);
		font-size: var(--type-sm);
	}

	.link {
		border: none;
		background: transparent;
		color: var(--accent-base);
		cursor: pointer;
		padding: 0;
		font: inherit;
	}

	.primary {
		padding-block: var(--space-3xs);
		padding-inline: var(--space-xs);
		border-radius: var(--radius-sm);
		background: var(--accent-base);
		color: var(--accent-text);
	}

	main {
		padding-block: var(--space-md);
		padding-inline: var(--space-md);
	}

	.verify-banner {
		display: flex;
		justify-content: center;
		align-items: center;
		gap: var(--space-xs);
		padding-block: var(--space-2xs);
		padding-inline: var(--space-md);
		background: oklch(from var(--status-warning) l c h / 0.18);
		color: var(--text-primary);
		font-size: var(--type-sm);
	}
</style>
