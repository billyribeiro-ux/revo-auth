<script lang="ts">
	import { SignupForm } from '@revo-auth/ui-sveltekit';
	import { authClient } from '$lib/auth/client';
	import { goto } from '$app/navigation';

	async function handleSuccess() {
		await goto('/dashboard');
	}
</script>

<section class="panel">
	<h1>Create an account</h1>
	<SignupForm client={authClient()} onSuccess={handleSuccess} />
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
</style>
