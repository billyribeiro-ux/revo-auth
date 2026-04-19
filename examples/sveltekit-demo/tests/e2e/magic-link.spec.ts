import { expect, test } from '@playwright/test';
import { AUTH_SERVER_URL, isAuthServerUp, latestMagicTokenFor, randomEmail } from './helpers';

test('user receives a magic link and lands signed in', async ({ page, request }) => {
	test.skip(!(await isAuthServerUp(request)), 'auth server not running');

	const email = randomEmail('magic');

	// Request a magic link directly via the server API (mirrors what the UI does).
	const requestResult = await request.post(`${AUTH_SERVER_URL}/v1/magic/request`, {
		data: { email, redirect: '/dashboard' }
	});
	expect(requestResult.ok()).toBeTruthy();

	// Fetch the token from the admin inspector.
	const token = await latestMagicTokenFor(request, email);
	test.skip(token === null, 'magic inspector unavailable');

	// Visit the magic-link proxy route — should land on /dashboard authenticated.
	await page.goto(`/magic?token=${token}`);
	await expect(page).toHaveURL(/\/dashboard/);
});
