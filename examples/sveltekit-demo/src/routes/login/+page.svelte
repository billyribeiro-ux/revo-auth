<script lang="ts">
	import { LoginForm } from '@revo-auth/ui-sveltekit';
	import { authClient } from '$lib/auth/client';
	import { goto, invalidateAll } from '$app/navigation';
	import { page } from '$app/state';

	const client = authClient();
	const redirect = $derived(page.url.searchParams.get('redirectTo') ?? '/dashboard');

	let errorMessage = $state('');

	async function handleSubmit(values: { email: string; password: string }) {
		errorMessage = '';
		const result = await client.signin(values);
		if (!result.ok) {
			errorMessage = result.error.message;
			return;
		}
		await invalidateAll();
		await goto(redirect);
	}

	async function handleOAuth(provider: 'google' | 'github' | 'microsoft' | 'apple' | 'discord') {
		const result = await client.oauthAuthorizeUrl(provider, redirect);
		if (result.ok) window.location.href = result.data.url;
		else errorMessage = result.error.message;
	}
</script>

<section class="panel">
	<h1>Sign in</h1>
	<LoginForm
		providers={['google', 'github']}
		onSubmit={handleSubmit}
		onOAuth={handleOAuth}
	/>
	{#if errorMessage !== ''}
		<p class="error" role="alert">{errorMessage}</p>
	{/if}
	<p class="aux">
		New here? <a href="/signup">Create an account</a> ·
		<a href="/auth/reset-password">Forgot password?</a>
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
