---
title: Multi-factor authentication
description: TOTP, recovery codes, and step-up prompts for sensitive actions.
---

Revo-Auth ships with multi-factor authentication built in. The default second factor is TOTP (RFC 6238 time-based codes, compatible with 1Password, Authy, Google Authenticator, and every other app on the planet), with recovery codes as a one-time fallback and passkeys as a stronger alternative.

## Enrolling TOTP

The server generates a 160-bit secret, encodes it as a `otpauth://` URI, and hands it to the client. Your SvelteKit app renders the URI as a QR code using `@revo-auth/ui-sveltekit`:

```svelte
<script>
  import { TotpEnroll } from "@revo-auth/ui-sveltekit";
</script>

<TotpEnroll onEnrolled={() => goto("/settings/security")} />
```

The user scans, types the first six-digit code, and the server verifies it before marking the factor active. Unverified secrets are purged after 10 minutes — we never leave a half-enrolled factor lying around.

## Recovery codes

On TOTP enrollment Revo-Auth generates 10 single-use recovery codes. They are Argon2id-hashed before storage. The UI shows them exactly once, with a download-as-text-file button. Each code consumes on use and regeneration invalidates the entire previous batch.

If a user loses their authenticator and their recovery codes, there is no backdoor. An admin in the dashboard can reset MFA for that user — that action is audit-logged with the admin's identity.

## Step-up authentication

Some routes need stronger proof than "the session cookie is valid." Deleting an account, adding an OAuth provider, rotating an API key — all of those should demand a recent second factor.

In a `+page.server.ts`:

```ts
import { requireStepUp } from "@revo-auth/sdk-sveltekit";

export const load = async (event) => {
  await requireStepUp(event, { maxAge: 5 * 60 }); // seconds
  // ... render sensitive page
};
```

If `session.step_up_at` is older than `maxAge`, the helper redirects to `/auth/step-up?returnTo=...`. The built-in step-up page accepts TOTP, passkey, or recovery code and bumps `step_up_at` on success. Users come back to the page they were trying to load, with no state lost.

## Choosing factors

- **Passkey** is always available once registered and satisfies step-up on its own. Prefer it.
- **TOTP** is universal and offline — good for anyone without a passkey-capable device.
- **Recovery codes** are a break-glass factor. Enforce them for admin accounts.

You can disable TOTP per-app via the `features` bit-flags if you want a passkey-only deployment.

## WebAuthn and TOTP side-by-side

A user can have both a passkey and TOTP on the same account. The login screen surfaces both; passkey is the suggested default. Losing access to one factor does not lock the user out as long as the other is registered — that redundancy is the whole point of MFA.
