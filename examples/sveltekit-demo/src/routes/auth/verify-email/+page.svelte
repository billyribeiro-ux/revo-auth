<script lang="ts">
	import { authClient } from '$lib/auth/client';
	import { page } from '$app/state';
	import { untrack } from 'svelte';

	type State = 'idle' | 'pending' | 'success' | 'error';

	const token = $derived(page.url.searchParams.get('token') ?? '');

	let state: State = $state('idle');
	let message = $state('');

	async function confirm() {
		if (token === '') {
			state = 'error';
			message = 'Missing token in URL.';
			return;
		}
		state = 'pending';
		const result = await authClient().emailVerifyConfirm({ token });
		if (result.ok) {
			state = 'success';
			message = 'Email verified. You can close this tab or return to the app.';
		} else {
			state = 'error';
			message = result.error.message;
		}
	}

	$effect(() => {
		if (token !== '') {
			untrack(() => {
				if (state === 'idle') void confirm();
			});
		}
	});
</script>

<section class="panel">
	<h1>Verify email</h1>
	{#if state === 'pending'}
		<p aria-busy="true">Verifying…</p>
	{:else if state === 'success'}
		<p class="success">{message}</p>
		<a class="btn" href="/dashboard">Continue to dashboard</a>
	{:else if state === 'error'}
		<p class="error">{message}</p>
		<button type="button" class="btn" onclick={confirm}>Try again</button>
	{:else}
		<p>Waiting for token…</p>
	{/if}
</section>

<style>
	.panel {
		max-inline-size: 40ch;
		margin-inline: auto;
		padding-block: var(--space-lg);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	h1 {
		margin: 0;
		font-size: var(--type-xl);
	}

	.success {
		color: var(--status-success);
	}

	.error {
		color: var(--status-danger);
	}

	.btn {
		align-self: flex-start;
		padding-block: var(--space-2xs);
		padding-inline: var(--space-sm);
		border-radius: var(--radius-sm);
		background: var(--accent-base);
		color: var(--accent-text);
		border: none;
		cursor: pointer;
		text-decoration: none;
		font: inherit;
	}
</style>
