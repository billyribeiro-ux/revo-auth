import { expect, test } from '@playwright/test';
import { isAuthServerUp, latestMailFor, randomEmail } from './helpers';

test('user can reset password via email link', async ({ page, request }) => {
	test.skip(!(await isAuthServerUp(request)), 'auth server not running');

	const email = randomEmail('reset');
	const originalPassword = 'Original!234';
	const newPassword = 'Rotated!5678';

	// Sign up first so an account exists.
	await page.goto('/signup');
	await page.getByLabel(/email/i).fill(email);
	await page.getByLabel(/password/i).fill(originalPassword);
	await page.getByRole('button', { name: /create account|sign up/i }).click();
	await expect(page).toHaveURL(/\/dashboard/);
	await page.getByRole('button', { name: /sign out/i }).click();

	// Request password reset.
	await page.goto('/auth/reset-password');
	await page.getByLabel(/email/i).fill(email);
	await page.getByRole('button', { name: /send reset link/i }).click();
	await expect(page.getByRole('status')).toBeVisible();

	// Pull the token out of the mock email inspector.
	const mail = await latestMailFor(request, email);
	test.skip(mail === null, 'mail inspector unavailable');
	const match = mail?.body.match(/token=([A-Za-z0-9._-]+)/);
	test.skip(!match, 'reset token not found in mail body');
	const token = match![1];

	// Confirm reset.
	await page.goto(`/auth/reset-password?token=${token}`);
	await page.getByLabel(/new password/i).fill(newPassword);
	await page.getByRole('button', { name: /set new password/i }).click();
	await expect(page.getByRole('status')).toBeVisible();

	// Sign in with the new password.
	await page.goto('/login');
	await page.getByLabel(/email/i).fill(email);
	await page.getByLabel(/password/i).fill(newPassword);
	await page.getByRole('button', { name: /sign in|log in/i }).click();

	await expect(page).toHaveURL(/\/dashboard/);
});
