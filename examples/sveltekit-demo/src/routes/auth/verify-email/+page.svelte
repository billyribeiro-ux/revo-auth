<script lang="ts">
	import { authClient } from '$lib/auth/client';
	import { page } from '$app/state';
	import { untrack } from 'svelte';

	type State = 'idle' | 'pending' | 'success' | 'error';

	const token = $derived(page.url.searchParams.get('token') ?? '');

	let status: State = $state('idle');
	let message = $state('');

	async function confirm() {
		if (token === '') {
			status = 'error';
			message = 'Missing token in URL.';
			return;
		}
		status = 'pending';
		const result = await authClient().emailVerifyConfirm({ token });
		if (result.ok) {
			status = 'success';
			message = 'Email verified. You can close this tab or return to the app.';
		} else {
			status = 'error';
			message = result.error.message;
		}
	}

	$effect(() => {
		if (token !== '') {
			untrack(() => {
				if (status === 'idle') void confirm();
			});
		}
	});
</script>

<section class="panel">
	<h1>Verify email</h1>
	{#if status === 'pending'}
		<p aria-busy="true">Verifying…</p>
	{:else if status === 'success'}
		<p class="success">{message}</p>
		<a class="btn" href="/dashboard">Continue to dashboard</a>
	{:else if status === 'error'}
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
