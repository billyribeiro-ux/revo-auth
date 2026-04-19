<script lang="ts">
	import { authClient } from '$lib/auth/client';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';

	type Phase = 'request' | 'confirm';
	type Status = 'idle' | 'pending' | 'success' | 'error';

	const token = $derived(page.url.searchParams.get('token') ?? '');
	const phase: Phase = $derived(token === '' ? 'request' : 'confirm');

	let email = $state('');
	let password = $state('');
	let status: Status = $state('idle');
	let message = $state('');

	async function handleRequest(event: SubmitEvent) {
		event.preventDefault();
		status = 'pending';
		const result = await authClient().passwordResetRequest({ email });
		if (result.ok) {
			status = 'success';
			message = 'If an account exists, a reset link has been emailed.';
		} else {
			status = 'error';
			message = result.error.message;
		}
	}

	async function handleConfirm(event: SubmitEvent) {
		event.preventDefault();
		status = 'pending';
		const result = await authClient().passwordResetConfirm({ token, password });
		if (result.ok) {
			status = 'success';
			message = 'Password updated. Redirecting to sign-in…';
			setTimeout(() => void goto('/login'), 750);
		} else {
			status = 'error';
			message = result.error.message;
		}
	}
</script>

<section class="panel">
	<h1>Reset password</h1>

	{#if phase === 'request'}
		<form onsubmit={handleRequest}>
			<label>
				<span>Email</span>
				<input
					type="email"
					required
					autocomplete="email"
					bind:value={email}
					disabled={status === 'pending'}
				/>
			</label>
			<button type="submit" class="btn primary" disabled={status === 'pending'}>
				{status === 'pending' ? 'Sending…' : 'Send reset link'}
			</button>
		</form>
	{:else}
		<form onsubmit={handleConfirm}>
			<label>
				<span>New password</span>
				<input
					type="password"
					minlength="8"
					required
					autocomplete="new-password"
					bind:value={password}
					disabled={status === 'pending'}
				/>
			</label>
			<button type="submit" class="btn primary" disabled={status === 'pending'}>
				{status === 'pending' ? 'Updating…' : 'Set new password'}
			</button>
		</form>
	{/if}

	{#if status === 'success'}
		<p class="success" role="status">{message}</p>
	{:else if status === 'error'}
		<p class="error" role="alert">{message}</p>
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

	form {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	label {
		display: flex;
		flex-direction: column;
		gap: var(--space-3xs);
	}

	label span {
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}

	input {
		padding-block: var(--space-2xs);
		padding-inline: var(--space-xs);
		border: 1px solid var(--surface-sunken);
		border-radius: var(--radius-sm);
		background: var(--surface-raised);
		color: var(--text-primary);
		font: inherit;
	}

	.btn {
		align-self: flex-start;
		padding-block: var(--space-2xs);
		padding-inline: var(--space-sm);
		border-radius: var(--radius-sm);
		border: none;
		cursor: pointer;
		font: inherit;
	}

	.primary {
		background: var(--accent-base);
		color: var(--accent-text);
	}

	.success {
		color: var(--status-success);
	}

	.error {
		color: var(--status-danger);
	}
</style>
