import { z } from 'zod';

export const loginSchema = z.object({
	email: z.string().email('Enter a valid email address'),
	password: z.string().min(1, 'Password is required')
});

export const signupSchema = z.object({
	name: z.string().min(1, 'Name is required').optional(),
	email: z.string().email('Enter a valid email address'),
	password: z
		.string()
		.min(8, 'Password must be at least 8 characters')
		.max(128, 'Password must be at most 128 characters'),
	terms: z.literal<true>(true, {
		errorMap: () => ({ message: 'You must accept the terms to continue' })
	})
});

export const totpSchema = z.object({
	code: z
		.string()
		.regex(/^\d{6}$/, 'Enter the 6-digit code from your authenticator')
});

export const magicRequestSchema = z.object({
	email: z.string().email('Enter a valid email address')
});

export type LoginValues = z.infer<typeof loginSchema>;
export type SignupValues = z.infer<typeof signupSchema>;
export type TotpValues = z.infer<typeof totpSchema>;
export type MagicRequestValues = z.infer<typeof magicRequestSchema>;
