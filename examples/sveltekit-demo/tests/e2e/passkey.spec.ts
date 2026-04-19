import { expect, test } from '@playwright/test';
import { isAuthServerUp, randomEmail } from './helpers';

/**
 * Passkey E2E uses Chrome DevTools Protocol's WebAuthn virtual authenticator
 * to simulate a platform authenticator without real biometrics.
 */
test('user registers and authenticates with a passkey', async ({ page, context, request }) => {
	test.skip(!(await isAuthServerUp(request)), 'auth server not running');

	const email = randomEmail('passkey');
	const password = 'Passkey!234';

	const client = await context.newCDPSession(page);
	await client.send('WebAuthn.enable');
	const { authenticatorId } = await client.send('WebAuthn.addVirtualAuthenticator', {
		options: {
			protocol: 'ctap2',
			transport: 'internal',
			hasResidentKey: true,
			hasUserVerification: true,
			isUserVerified: true,
			automaticPresenceSimulation: true
		}
	});

	// Sign up with password so we have an authenticated session to register from.
	await page.goto('/signup');
	await page.getByLabel(/email/i).fill(email);
	await page.getByLabel(/password/i).fill(password);
	await page.getByRole('button', { name: /create account|sign up/i }).click();
	await expect(page).toHaveURL(/\/dashboard/);

	// Register a passkey.
	await page.goto('/dashboard/settings');
	await page.getByRole('button', { name: /register passkey|add passkey/i }).click();
	await expect(page.getByText(/passkey (registered|added|saved)/i)).toBeVisible();

	// Sign out.
	await page.getByRole('button', { name: /sign out/i }).click();

	// Authenticate with passkey.
	await page.goto('/login');
	await page.getByRole('button', { name: /sign in with passkey|use passkey/i }).click();

	await expect(page).toHaveURL(/\/dashboard/);

	await client.send('WebAuthn.removeVirtualAuthenticator', { authenticatorId });
});
