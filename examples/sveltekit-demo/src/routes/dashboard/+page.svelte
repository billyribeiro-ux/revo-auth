<script lang="ts">
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	const { data }: Props = $props();
	const user = $derived(data.session.user);
</script>

<section class="dashboard">
	<h1>Welcome back{user.name ? `, ${user.name}` : ''}.</h1>
	<dl>
		<div>
			<dt>User ID</dt>
			<dd><code>{user.id}</code></dd>
		</div>
		<div>
			<dt>Email</dt>
			<dd>{user.email ?? '(none)'}</dd>
		</div>
		<div>
			<dt>Email verified</dt>
			<dd>{user.emailVerified ? 'Yes' : 'No'}</dd>
		</div>
		<div>
			<dt>Created</dt>
			<dd>{new Date(user.createdAt).toLocaleString()}</dd>
		</div>
	</dl>
	<p>
		Head to <a href="/dashboard/settings">settings</a> to manage passkeys, TOTP, active sessions,
		and linked accounts.
	</p>
</section>

<style>
	.dashboard {
		max-inline-size: 60ch;
		margin-inline: auto;
		padding-block: var(--space-md);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	h1 {
		margin: 0;
		font-size: var(--type-xl);
	}

	dl {
		display: grid;
		grid-template-columns: minmax(8rem, max-content) 1fr;
		gap: var(--space-2xs) var(--space-sm);
		margin: 0;
	}

	dl > div {
		display: contents;
	}

	dt {
		color: var(--text-muted);
		font-size: var(--type-sm);
	}

	dd {
		margin: 0;
	}

	code {
		background: var(--surface-sunken);
		padding-inline: var(--space-3xs);
		border-radius: var(--radius-sm);
		font-size: 0.95em;
	}

	a {
		color: var(--accent-base);
	}
</style>
