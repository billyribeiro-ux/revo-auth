export type AuthMethod =
	| 'email'
	| 'google'
	| 'github'
	| 'microsoft'
	| 'apple'
	| 'discord'
	| 'passkeys'
	| 'totp'
	| 'magic-link';

export type SessionStrategy = 'cookie' | 'bearer';

export type AuthFeature =
	| 'organizations'
	| 'audit-log'
	| 'custom-fields'
	| 'anonymous-mode'
	| 'no-analytics'
	| 'no-third-party-oauth';

export interface Preset {
	name: string;
	methods: AuthMethod[];
	session: SessionStrategy;
	features: AuthFeature[];
}
