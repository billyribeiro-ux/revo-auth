import { z } from 'zod';

export const userSchema = z.object({
	id: z.string(),
	email: z.string().email().nullable(),
	emailVerified: z.boolean(),
	name: z.string().nullable(),
	image: z.string().nullable(),
	customFields: z.record(z.unknown()),
	createdAt: z.string(),
});

export const sessionSchema = z.object({
	id: z.string(),
	userId: z.string(),
	expiresAt: z.string(),
	user: userSchema,
});

export const sessionResponseSchema = z.object({
	session: sessionSchema,
});

export const sessionNullableResponseSchema = z.object({
	session: sessionSchema.nullable(),
});

export const sessionsListResponseSchema = z.object({
	items: z.array(sessionSchema),
});

export const okResponseSchema = z.object({
	ok: z.literal(true),
});

export const errorEnvelopeSchema = z.object({
	error: z.object({
		code: z.string(),
		message: z.string(),
		request_id: z.string().optional(),
	}),
});

export const configSchema = z.object({
	serverUrl: z.string().url(),
	appId: z.string().min(1),
	publicKey: z.string().min(1),
	fetchFn: z.custom<typeof fetch>((v) => typeof v === 'function').optional(),
	logger: z
		.object({
			debug: z.function(),
			info: z.function(),
			warn: z.function(),
			error: z.function(),
		})
		.optional(),
	cookie: z.string().optional(),
	csrfToken: z.string().optional(),
});

export const passkeySchema = z.object({
	id: z.string(),
	createdAt: z.string(),
	lastUsedAt: z.string().nullable(),
	name: z.string().nullable(),
});

export const passkeyListResponseSchema = z.object({
	items: z.array(passkeySchema),
});

export const organizationSchema = z.object({
	id: z.string(),
	name: z.string(),
	slug: z.string(),
	role: z.enum(['owner', 'admin', 'member', 'viewer']),
});

export const organizationListResponseSchema = z.object({
	items: z.array(organizationSchema),
});

export const totpSetupResponseSchema = z.object({
	secret: z.string(),
	otpauthUrl: z.string(),
	qrCodeDataUrl: z.string(),
});

export const passkeyRegisterBeginResponseSchema = z.object({
	publicKey: z.record(z.unknown()),
});

export const passkeyRegisterFinishResponseSchema = z.object({
	passkey: passkeySchema,
});

export const passkeyAuthenticateBeginResponseSchema = z.object({
	publicKey: z.record(z.unknown()),
});

export const passkeyAuthenticateFinishResponseSchema = z.object({
	session: sessionSchema,
});

export const magicRequestResponseSchema = z.object({
	ok: z.literal(true),
});

export const oauthAuthorizeResponseSchema = z.object({
	url: z.string().url(),
});
