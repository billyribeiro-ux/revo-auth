---
title: Passkeys
description: WebAuthn-backed, device-bound credentials that let your users sign in without a password.
---

Passkeys are the future Revo-Auth is built for. Under the hood they are WebAuthn credentials — asymmetric keypairs generated on a platform authenticator (Touch ID, Face ID, Windows Hello, a hardware key) where the private half never leaves the device.

## Why passkeys

- **Phishing-resistant.** The credential is bound to the Relying Party ID (your domain). A lookalike site cannot coax the authenticator into signing a challenge because the browser refuses to present the credential.
- **No shared secret.** There is nothing on the server to leak. A breached `passkeys` table gives an attacker public keys, which are useless without the private key on the device.
- **Cross-device sync.** iCloud Keychain, Google Password Manager, and 1Password roam passkeys across a user's devices. Your users register once and sign in everywhere.

## The Revo-Auth model

Each passkey is a row in the `passkeys` table, scoped to an app and a user. Revo-Auth stores:

- `credential_id` — the authenticator's opaque identifier.
- `public_key` — COSE-encoded public key.
- `sign_count` — the authenticator's counter, used to detect cloned credentials.
- `aaguid` — identifies the authenticator model (e.g. "iCloud Keychain").
- `transports` — how the browser found the authenticator (`internal`, `hybrid`, `usb`, `nfc`, `ble`).

There is no password table. A user who signs up with a passkey has no fallback secret to phish.

## Registration flow

1. Authenticated user clicks "Add passkey" in your settings page.
2. Your SvelteKit app calls `sdk.passkey.beginRegistration()`. The server returns a challenge, the RP ID, and the user handle.
3. The SDK hands the challenge to `navigator.credentials.create()` via `@simplewebauthn/browser`.
4. The authenticator prompts the user (biometric, PIN, or hardware tap), generates a keypair, and returns an attestation response.
5. The SDK posts the attestation to `sdk.passkey.finishRegistration()`. The server verifies it, stores the credential, and returns the updated user.

## Login flow

1. The user clicks "Sign in with a passkey." No email required — the authenticator advertises which credentials it knows about for this RP.
2. `sdk.passkey.beginAuthentication()` returns a challenge.
3. `navigator.credentials.get()` returns an assertion.
4. `sdk.passkey.finishAuthentication()` verifies the signature against the stored public key, checks the sign counter, and issues a session.

The whole flow is two round-trips and zero typing.

## Step-up and recovery

Revo-Auth treats a fresh passkey assertion as satisfying MFA. Sensitive routes can call `requireStepUp` and the SvelteKit helper will prompt for a passkey even if the user is already signed in.

Every user with a passkey also gets **10 recovery codes** on first registration. They are hashed at rest (Argon2id — these are low-entropy compared to session tokens). Losing every passkey without recovery codes means account deletion, by design.

## Browser support

All evergreen browsers plus Safari 16+ support passkeys. The SDK detects support via `PublicKeyCredential.isConditionalMediationAvailable()` and falls back gracefully — your UI should show password/OAuth options alongside the passkey button until adoption catches up.
