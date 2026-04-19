import { z } from 'zod';

export const methodSchema = z.enum([
	'email',
	'google',
	'github',
	'microsoft',
	'apple',
	'discord',
	'passkeys',
	'totp',
	'magic-link',
]);

export const featureSchema = z.enum([
	'organizations',
	'audit-log',
	'custom-fields',
	'anonymous-mode',
	'no-analytics',
	'no-third-party-oauth',
]);

export const sessionSchema = z.enum(['cookie', 'bearer']);

export const initConfigSchema = z.object({
	serverUrl: z.string().url(),
	preset: z.string(),
	methods: z.array(methodSchema),
	features: z.array(featureSchema),
	session: sessionSchema,
	appId: z.string().default('app_local_dev'),
});

export type InitConfig = z.infer<typeof initConfigSchema>;
