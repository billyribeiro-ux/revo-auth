<script lang="ts">
	import { SignupForm } from '@revo-auth/ui-sveltekit';
	import { authClient } from '$lib/auth/client';
	import { goto, invalidateAll } from '$app/navigation';

	const client = authClient();
	let errorMessage = $state('');

	async function handleSubmit(values: {
		name?: string;
		email: string;
		password: string;
		terms: true;
	}) {
		errorMessage = '';
		const payload: { email: string; password: string; name?: string } = {
			email: values.email,
			password: values.password
		};
		if (values.name !== undefined && values.name !== '') payload.name = values.name;
		const result = await client.signup(payload);
		if (!result.ok) {
			errorMessage = result.error.message;
			return;
		}
		await invalidateAll();
		await goto('/dashboard');
	}
</script>

<section class="panel">
	<h1>Create an account</h1>
	<SignupForm onSubmit={handleSubmit} />
	{#if errorMessage !== ''}
		<p class="error" role="alert">{errorMessage}</p>
	{/if}
	<p class="aux">
		Already have an account? <a href="/login">Sign in</a>
	</p>
</section>

<style>
	.panel {
		max-inline-size: 40ch;
		margin-inline: auto;
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		padding-block: var(--space-lg);
	}

	h1 {
		margin: 0;
		font-size: var(--type-xl);
	}

	.aux {
		color: var(--text-muted);
		font-size: var(--type-sm);
	}

	.aux a {
		color: var(--accent-base);
	}

	.error {
		color: var(--status-danger);
		margin: 0;
	}
</style>
