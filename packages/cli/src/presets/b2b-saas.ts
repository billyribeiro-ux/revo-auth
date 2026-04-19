import type { Preset } from './types.js';

const preset: Preset = {
	name: 'b2b-saas',
	methods: ['email', 'google', 'microsoft', 'passkeys', 'totp'],
	session: 'cookie',
	features: ['organizations', 'audit-log', 'custom-fields'],
};

export default preset;
