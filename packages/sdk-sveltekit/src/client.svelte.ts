import type { ClientAPI, RevoAuthSession } from '@revo-auth/sdk-core';

export interface SessionStore {
	readonly current: RevoAuthSession | null;
	readonly loading: boolean;
	refresh(): Promise<void>;
}

/**
 * Create a Svelte 5 rune-backed session store. Exposes reactive getters
 * (NOT a writable store). Call `refresh()` to re-fetch from the server.
 *
 * @param client  An SDK client created via `createClient`.
 * @param initial Optional session to seed (e.g. from SSR `event.locals.session`).
 */
export function createSessionStore(
	client: ClientAPI,
	initial: RevoAuthSession | null = null,
): SessionStore {
	let _session = $state<RevoAuthSession | null>(initial);
	let _loading = $state<boolean>(false);

	async function refresh(): Promise<void> {
		_loading = true;
		try {
			const result = await client.session();
			if (result.ok) {
				_session = result.data;
			} else {
				_session = null;
			}
		} finally {
			_loading = false;
		}
	}

	return {
		get current() {
			return _session;
		},
		get loading() {
			return _loading;
		},
		refresh,
	};
}
