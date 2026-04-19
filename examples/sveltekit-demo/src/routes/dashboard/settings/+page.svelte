<script lang="ts">
	import { MfaSetup, PasskeyButton } from '@revo-auth/ui-sveltekit';
	import { authClient } from '$lib/auth/client';
	import { registerPasskey } from '@revo-auth/sdk-core';
	import type { Passkey, RevoAuthSession, TotpSetupResult, Organization } from '@revo-auth/sdk-core';
	import { untrack } from 'svelte';

	const client = authClient();

	// -------------------- Sessions --------------------
	let sessions = $state<RevoAuthSession[]>([]);
	let sessionsError = $state('');

	async function loadSessions() {
		sessionsError = '';
		const result = await client.listSessions();
		if (result.ok) sessions = [...result.data.items];
		else sessionsError = result.error.message;
	}

	async function revokeSession(id: string) {
		const result = await client.revokeSession(id);
		if (result.ok) sessions = sessions.filter((s) => s.id !== id);
		else sessionsError = result.error.message;
	}

	// -------------------- Passkeys --------------------
	let passkeys = $state<Passkey[]>([]);
	let passkeyError = $state('');

	async function loadPasskeys() {
		passkeyError = '';
		const result = await client.listPasskeys();
		if (result.ok) passkeys = [...result.data.items];
		else passkeyError = result.error.message;
	}

	async function handleRegisterPasskey() {
		passkeyError = '';
		const result = await registerPasskey(client);
		if (result.ok) await loadPasskeys();
		else passkeyError = result.error.message;
	}

	async function revokePasskey(id: string) {
		const result = await client.revokePasskey(id);
		if (result.ok) passkeys = passkeys.filter((p) => p.id !== id);
		else passkeyError = result.error.message;
	}

	// -------------------- TOTP --------------------
	let totpSetup = $state<TotpSetupResult | null>(null);
	let totpError = $state('');
	let totpEnabled = $state(false);

	async function startTotp() {
		totpError = '';
		const result = await client.totpSetup();
		if (result.ok) totpSetup = result.data;
		else totpError = result.error.message;
	}

	async function confirmTotp(code: string) {
		totpError = '';
		const result = await client.totpConfirm({ code });
		if (result.ok) {
			totpEnabled = true;
			totpSetup = null;
		} else {
			totpError = result.error.message;
			throw new Error(result.error.message);
		}
	}

	async function disableTotp() {
		totpError = '';
		const result = await client.totpDisable();
		if (result.ok) totpEnabled = false;
		else totpError = result.error.message;
	}

	// -------------------- OAuth links --------------------
	const providers = ['google', 'github'] as const;
	let linkError = $state('');

	async function linkProvider(provider: (typeof providers)[number]) {
		linkError = '';
		const result = await client.linkAccount(provider, '/dashboard/settings');
		if (result.ok) window.location.href = result.data.url;
		else linkError = result.error.message;
	}

	async function unlinkProvider(provider: (typeof providers)[number]) {
		linkError = '';
		const result = await client.unlinkAccount(provider);
		if (!result.ok) linkError = result.error.message;
	}

	// -------------------- Orgs (bonus) --------------------
	let orgs = $state<Organization[]>([]);

	async function loadOrgs() {
		const result = await client.listOrgs();
		if (result.ok) orgs = [...result.data.items];
	}

	// Kick off initial loads once.
	$effect(() => {
		untrack(() => {
			void loadSessions();
			void loadPasskeys();
			void loadOrgs();
		});
	});
</script>

<section class="settings">
	<header>
		<h1>Account settings</h1>
		<p>Manage your authentication factors, active sessions, and linked providers.</p>
	</header>

	<article>
		<h2>Passkeys</h2>
		<p>Register a passkey for a password-less sign-in experience.</p>
		<PasskeyButton mode="register" onClick={handleRegisterPasskey} />
		{#if passkeys.length > 0}
			<ul>
				{#each passkeys as passkey (passkey.id)}
					<li>
						<span>{passkey.name ?? passkey.id.slice(0, 8)}</span>
						<small>Added {new Date(passkey.createdAt).toLocaleDateString()}</small>
						<button type="button" onclick={() => revokePasskey(passkey.id)}>Remove</button>
					</li>
				{/each}
			</ul>
		{/if}
		{#if passkeyError !== ''}
			<p class="error" role="alert">{passkeyError}</p>
		{/if}
	</article>

	<article>
		<h2>Two-factor authentication (TOTP)</h2>
		{#if totpSetup}
			<p>Scan the QR code with an authenticator app, then enter the generated code.</p>
			<p>
				Secret: <code data-testid="totp-secret">{totpSetup.secret}</code>
			</p>
			<MfaSetup
				qrUri={totpSetup.otpauthUrl}
				secret={totpSetup.secret}
				recoveryCodes={[]}
				onConfirm={confirmTotp}
			/>
		{:else if totpEnabled}
			<p>TOTP is enabled on this account.</p>
			<button type="button" onclick={disableTotp}>Disable TOTP</button>
		{:else}
			<p>TOTP is not yet enabled on this account.</p>
			<button type="button" onclick={startTotp}>Enable TOTP</button>
		{/if}
		{#if totpError !== ''}
			<p class="error" role="alert">{totpError}</p>
		{/if}
	</article>

	<article>
		<h2>Active sessions</h2>
		{#if sessions.length === 0}
			<p>No other sessions.</p>
		{:else}
			<ul>
				{#each sessions as session (session.id)}
					<li>
						<span>{session.user.email ?? session.user.id}</span>
						<small>Expires {new Date(session.expiresAt).toLocaleString()}</small>
						<button type="button" onclick={() => revokeSession(session.id)}>Revoke</button>
					</li>
				{/each}
			</ul>
		{/if}
		{#if sessionsError !== ''}
			<p class="error" role="alert">{sessionsError}</p>
		{/if}
	</article>

	<article>
		<h2>Linked accounts</h2>
		<div class="providers">
			{#each providers as provider (provider)}
				<div>
					<span class="provider">{provider}</span>
					<button type="button" onclick={() => linkProvider(provider)}>Link</button>
					<button type="button" onclick={() => unlinkProvider(provider)}>Unlink</button>
				</div>
			{/each}
		</div>
		{#if linkError !== ''}
			<p class="error" role="alert">{linkError}</p>
		{/if}
	</article>

	{#if orgs.length > 0}
		<article>
			<h2>Organizations</h2>
			<ul>
				{#each orgs as org (org.id)}
					<li>
						<span>{org.name}</span>
						<small>({org.role})</small>
					</li>
				{/each}
			</ul>
		</article>
	{/if}
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

	ul {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-2xs);
	}

	li {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	small {
		color: var(--text-muted);
		font-size: var(--type-sm);
	}

	button {
		padding-block: var(--space-3xs);
		padding-inline: var(--space-xs);
		border-radius: var(--radius-sm);
		border: 1px solid var(--surface-sunken);
		background: var(--surface-raised);
		color: var(--text-primary);
		font: inherit;
		cursor: pointer;
	}

	button:hover {
		background: var(--surface-sunken);
	}

	.providers {
		display: flex;
		flex-direction: column;
		gap: var(--space-2xs);
	}

	.providers > div {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.provider {
		text-transform: capitalize;
		flex: 1;
	}

	code {
		background: var(--surface-sunken);
		padding-inline: var(--space-3xs);
		border-radius: var(--radius-sm);
		font-size: 0.95em;
	}

	.error {
		color: var(--status-danger);
	}
</style>
