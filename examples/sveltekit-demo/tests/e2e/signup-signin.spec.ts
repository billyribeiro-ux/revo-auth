import { expect, test } from '@playwright/test';
import { isAuthServerUp, randomEmail } from './helpers';

test('user can sign up, sign out, and sign back in', async ({ page, request }) => {
	test.skip(!(await isAuthServerUp(request)), 'auth server not running');

	const email = randomEmail('signup');
	const password = 'Sup3rSecret!23';

	await page.goto('/signup');
	await page.getByLabel(/email/i).fill(email);
	await page.getByLabel(/password/i).fill(password);
	await page.getByRole('button', { name: /create account|sign up/i }).click();

	await expect(page).toHaveURL(/\/dashboard/);
	await expect(page.getByText(email)).toBeVisible();

	await page.getByRole('button', { name: /sign out/i }).click();
	await expect(page).toHaveURL('/');

	await page.goto('/login');
	await page.getByLabel(/email/i).fill(email);
	await page.getByLabel(/password/i).fill(password);
	await page.getByRole('button', { name: /sign in|log in/i }).click();

	await expect(page).toHaveURL(/\/dashboard/);
});
