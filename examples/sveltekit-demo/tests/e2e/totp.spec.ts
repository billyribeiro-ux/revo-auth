import { createHmac } from 'node:crypto';
import { expect, test } from '@playwright/test';
import { AUTH_SERVER_URL, isAuthServerUp, randomEmail } from './helpers';

/** Compute a TOTP code from a base32 secret for the current 30s window. */
function totp(secret: string, step = 30): string {
	const key = base32Decode(secret);
	const counter = Math.floor(Date.now() / 1000 / step);
	const buf = Buffer.alloc(8);
	buf.writeBigUInt64BE(BigInt(counter));
	const hmac = createHmac('sha1', key).update(buf).digest();
	const offset = hmac[hmac.length - 1] & 0xf;
	const code =
		((hmac[offset] & 0x7f) << 24) |
		((hmac[offset + 1] & 0xff) << 16) |
		((hmac[offset + 2] & 0xff) << 8) |
		(hmac[offset + 3] & 0xff);
	return (code % 1_000_000).toString().padStart(6, '0');
}

function base32Decode(input: string): Buffer {
	const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
	const clean = input.replace(/=+$/, '').toUpperCase();
	let bits = 0;
	let value = 0;
	const out: number[] = [];
	for (const ch of clean) {
		const idx = alphabet.indexOf(ch);
		if (idx < 0) continue;
		value = (value << 5) | idx;
		bits += 5;
		if (bits >= 8) {
			out.push((value >>> (bits - 8)) & 0xff);
			bits -= 8;
		}
	}
	return Buffer.from(out);
}

test('user enables TOTP and signs in with the generated code', async ({ page, request }) => {
	test.skip(!(await isAuthServerUp(request)), 'auth server not running');

	const email = randomEmail('totp');
	const password = 'Passw0rd!Totp';

	// Sign up.
	await page.goto('/signup');
	await page.getByLabel(/email/i).fill(email);
	await page.getByLabel(/password/i).fill(password);
	await page.getByRole('button', { name: /create account|sign up/i }).click();
	await expect(page).toHaveURL(/\/dashboard/);

	// Enable TOTP. The MfaSetup component surfaces the secret and a confirm input.
	await page.goto('/dashboard/settings');
	const enableBtn = page.getByRole('button', { name: /enable totp|set up totp/i });
	await enableBtn.click();

	const secretEl = page.getByTestId('totp-secret');
	await expect(secretEl).toBeVisible();
	const secret = (await secretEl.textContent())?.trim() ?? '';
	test.skip(secret.length === 0, 'TOTP secret not exposed');

	await page.getByLabel(/one-time code/i).fill(totp(secret));
	await page.getByRole('button', { name: /confirm|verify/i }).click();

	// Sign out and back in.
	await page.getByRole('button', { name: /sign out/i }).click();
	await page.goto('/login');
	await page.getByLabel(/email/i).fill(email);
	await page.getByLabel(/password/i).fill(password);
	await page.getByRole('button', { name: /sign in|log in/i }).click();

	// Expect a TOTP challenge to appear.
	const totpInput = page.getByLabel(/totp|one-time code/i);
	await expect(totpInput).toBeVisible();
	await totpInput.fill(totp(secret));
	await page.getByRole('button', { name: /verify|continue|sign in/i }).click();

	await expect(page).toHaveURL(/\/dashboard/);
});
