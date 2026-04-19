import { handleAuth } from '@revo-auth/sdk-sveltekit';
import { sequence } from '@sveltejs/kit/hooks';
import type { Handle } from '@sveltejs/kit';
import { authConfig } from '../auth.config.js';

const auth = handleAuth({ config: authConfig });

/**
 * Example slot for consumer-added middleware. We pass a pass-through handle so
 * future additions to `sequence()` are ergonomic — drop new handlers in here.
 */
const appHandle: Handle = async ({ event, resolve }) => resolve(event);

export const handle = sequence(auth, appHandle);
