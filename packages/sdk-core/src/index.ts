export { createClient } from './client.js';
export type { ClientAPI } from './client.js';
export {
	InvalidCredentialsError,
	NetworkError,
	RateLimitedError,
	RevoAuthError,
	SessionExpiredError,
	UnknownError,
	ValidationError,
	errorFromEnvelope,
} from './errors.js';
export type { RevoAuthErrorCode, RevoAuthErrorJSON } from './errors.js';
export { beginOAuth } from './oauth.js';
export { authenticateWithPasskey, registerPasskey } from './passkey.js';
export type { RegisterPasskeyOptions } from './passkey.js';
export {
	errorEnvelopeSchema,
	sessionResponseSchema,
	sessionSchema,
	userSchema,
} from './schemas.js';
export { confirmTotp, disableTotp, setupTotp, verifyTotp } from './totp.js';
export { magicVerifyUrl, requestMagicLink } from './magic.js';
export type {
	AcceptInviteInput,
	CreateOrgInput,
	EmailVerifyConfirmInput,
	InviteMemberInput,
	ListResponse,
	MagicRequestInput,
	OAuthProvider,
	OkResponse,
	Organization,
	Passkey,
	PasswordResetConfirmInput,
	PasswordResetRequestInput,
	Result,
	RevoAuthConfig,
	RevoAuthLogger,
	RevoAuthSession,
	RevoAuthUser,
	SigninInput,
	SignupInput,
	TotpConfirmInput,
	TotpSetupResult,
	TotpVerifyInput,
} from './types.js';
