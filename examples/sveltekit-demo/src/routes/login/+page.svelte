<script lang="ts">
	import { LoginForm } from '@revo-auth/ui-sveltekit';
	import { authClient } from '$lib/auth/client';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	const redirect = $derived(page.url.searchParams.get('redirectTo') ?? '/dashboard');

	async function handleSuccess() {
		await goto(redirect);
	}
</script>

<section class="panel">
	<h1>Sign in</h1>
	<LoginForm client={authClient()} onSuccess={handleSuccess} />
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
</style>
