import b2bSaas from './b2b-saas.js';
import minimal from './minimal.js';
import privacyFirst from './privacy-first.js';
import tradingPlatform from './trading-platform.js';
import type { Preset } from './types.js';

export const PRESETS: Record<string, Preset> = {
	'trading-platform': tradingPlatform,
	'b2b-saas': b2bSaas,
	'privacy-first': privacyFirst,
	minimal,
};

export type PresetName = keyof typeof PRESETS;

export function getPreset(name: string): Preset | undefined {
	return PRESETS[name];
}

export function listPresets(): {
	value: string;
	label: string;
	hint: string;
}[] {
	return [
		{
			value: 'trading-platform',
			label: 'Trading platform',
			hint: 'email + google + TOTP, organizations, audit log',
		},
		{
			value: 'b2b-saas',
			label: 'B2B SaaS',
			hint: 'SSO providers, passkeys, orgs, audit log',
		},
		{
			value: 'privacy-first',
			label: 'Privacy first',
			hint: 'No 3rd-party OAuth, no analytics',
		},
		{ value: 'minimal', label: 'Minimal', hint: 'Just email/password' },
	];
}

export type { Preset } from './types.js';
