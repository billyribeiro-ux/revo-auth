import { requireAuth } from '@revo-auth/sdk-sveltekit';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = (event) => {
	const session = requireAuth(event);
	return { session };
};
