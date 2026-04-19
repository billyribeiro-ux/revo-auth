import type { Preset } from './types.js';

const preset: Preset = {
	name: 'privacy-first',
	methods: ['email', 'passkeys', 'totp', 'magic-link'],
	session: 'cookie',
	features: ['no-analytics', 'no-third-party-oauth', 'anonymous-mode'],
};

export default preset;
