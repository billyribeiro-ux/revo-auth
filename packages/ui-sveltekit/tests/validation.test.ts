import { describe, expect, it } from 'vitest';
import {
	loginSchema,
	signupSchema,
	totpSchema,
	magicRequestSchema
} from '../src/lib/validation';

describe('loginSchema', () => {
	it('accepts a valid email and password', () => {
		const parsed = loginSchema.safeParse({
			email: 'user@example.com',
			password: 'hunter2'
		});
		expect(parsed.success).toBe(true);
	});

	it('rejects a bad email', () => {
		const parsed = loginSchema.safeParse({ email: 'nope', password: 'x' });
		expect(parsed.success).toBe(false);
	});
});

describe('signupSchema', () => {
	it('requires terms to be true', () => {
		const parsed = signupSchema.safeParse({
			email: 'a@b.com',
			password: 'longenough',
			terms: false
		});
		expect(parsed.success).toBe(false);
	});

	it('requires an 8+ character password', () => {
		const parsed = signupSchema.safeParse({
			email: 'a@b.com',
			password: 'short',
			terms: true
		});
		expect(parsed.success).toBe(false);
	});
});

describe('totpSchema', () => {
	it('accepts a 6-digit code', () => {
		expect(totpSchema.safeParse({ code: '123456' }).success).toBe(true);
	});

	it('rejects non-digit codes', () => {
		expect(totpSchema.safeParse({ code: '12a456' }).success).toBe(false);
	});
});

describe('magicRequestSchema', () => {
	it('validates an email', () => {
		expect(magicRequestSchema.safeParse({ email: 'a@b.com' }).success).toBe(
			true
		);
	});
});
