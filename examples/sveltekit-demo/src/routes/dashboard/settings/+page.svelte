<script lang="ts">
	import {
		MfaSetup,
		PasskeyButton,
		SessionList,
		AccountLinking
	} from '@revo-auth/ui-sveltekit';
	import { authClient } from '$lib/auth/client';
</script>

<section class="settings">
	<header>
		<h1>Account settings</h1>
		<p>Manage your authentication factors, active sessions, and linked providers.</p>
	</header>

	<article>
		<h2>Passkey</h2>
		<p>Register a passkey for a password-less sign-in experience.</p>
		<PasskeyButton client={authClient()} mode="register" />
	</article>

	<article>
		<h2>Two-factor authentication (TOTP)</h2>
		<p>Scan the QR code with an authenticator app to enable TOTP on sign-in.</p>
		<MfaSetup client={authClient()} />
	</article>

	<article>
		<h2>Active sessions</h2>
		<SessionList client={authClient()} />
	</article>

	<article>
		<h2>Linked accounts</h2>
		<AccountLinking client={authClient()} providers={['google', 'github']} />
	</article>
</section>

<style>
	.settings {
		max-inline-size: 70ch;
		margin-inline: auto;
		padding-block: var(--space-md);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	h1 {
		margin: 0;
		font-size: var(--type-xl);
	}

	h2 {
		margin: 0;
		font-size: var(--type-lg);
	}

	article {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		padding: var(--space-sm);
		border: 1px solid var(--surface-sunken);
		border-radius: var(--radius-md);
		background: var(--surface-raised);
		box-shadow: var(--elev-1);
	}

	p {
		color: var(--text-secondary);
		margin: 0;
	}
</style>
