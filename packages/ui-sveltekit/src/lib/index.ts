// Component exports
export { default as LoginForm } from './components/LoginForm.svelte';
export { default as SignupForm } from './components/SignupForm.svelte';
export { default as OAuthButton } from './components/OAuthButton.svelte';
export { default as PasskeyButton } from './components/PasskeyButton.svelte';
export { default as MagicLinkForm } from './components/MagicLinkForm.svelte';
export { default as MfaSetup } from './components/MfaSetup.svelte';
export { default as MfaChallenge } from './components/MfaChallenge.svelte';
export { default as PasswordField } from './components/PasswordField.svelte';
export { default as PasswordStrengthMeter } from './components/PasswordStrengthMeter.svelte';
export { default as SessionList } from './components/SessionList.svelte';
export { default as AccountLinking } from './components/AccountLinking.svelte';
export { default as VerifyEmailBanner } from './components/VerifyEmailBanner.svelte';

// Library exports
export { createForm } from './form.svelte';
export type {
	CreateFormOptions,
	FormController,
} from './form.svelte';
export {
	loginSchema,
	signupSchema,
	totpSchema,
	magicRequestSchema,
} from './validation';
export type {
	LoginValues,
	SignupValues,
	TotpValues,
	MagicRequestValues,
} from './validation';
