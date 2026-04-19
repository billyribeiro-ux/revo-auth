---
title: Apple
description: Enable Sign in with Apple for Revo-Auth — Developer portal setup, the Services ID flow, and env vars.
---

Sign in with Apple is the most fiddly provider to set up, and it's mandatory if you ship an iOS app that also supports any other OAuth provider (Apple's App Store Review Guideline 4.8). Budget 30 minutes the first time.

## 1. Enroll as an Apple Developer

You need a paid Apple Developer account ($99/year). Free accounts cannot create Services IDs. Open [developer.apple.com](https://developer.apple.com/) and sign in.

## 2. Create an App ID

**Certificates, Identifiers & Profiles → Identifiers → +**. Pick **App IDs → App**. Give it a description and a Bundle ID (reverse-DNS, e.g. `dev.example.auth`). In the Capabilities list, enable **Sign In with Apple**.

## 3. Create a Services ID

Repeat the **Identifiers → +** flow, this time pick **Services IDs**. The Services ID is what Revo-Auth uses as the OAuth client ID.

- **Description:** user-facing on the consent sheet.
- **Identifier:** reverse-DNS, e.g. `dev.example.auth.signin`. Different from the App ID.

![Apple Developer — Services ID with Sign in with Apple configuration](/screenshots/providers/apple-01-services-id.png)

*Screenshot placeholder: the Services ID configuration screen with Sign in with Apple enabled.*

Enable **Sign In with Apple** on the Services ID and click **Configure**:

- **Primary App ID:** the App ID you made in step 2.
- **Domains and Subdomains:** your Revo-Auth server's domain (no scheme, no path).
- **Return URLs:**
  ```
  https://<your-revo-auth-server>/v1/oauth/apple/callback
  ```

Save.

## 4. Create a Sign in with Apple key

**Keys → +**. Name it, enable **Sign In with Apple**, and click **Configure** to bind it to the same App ID. Register. Apple generates a `.p8` file — **download it now**, Apple won't let you download it again.

Note the **Key ID** shown on the key's page and your **Team ID** (top right of the developer portal).

## 5. Set env vars on the Revo-Auth server

```sh
REVO_AUTH_APPLE_CLIENT_ID="dev.example.auth.signin"  # the Services ID
REVO_AUTH_APPLE_TEAM_ID="ABCDE12345"
REVO_AUTH_APPLE_KEY_ID="ABCDE67890"
REVO_AUTH_APPLE_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----"
```

The private key is the contents of the `.p8` file with literal `\n` sequences. Revo-Auth uses it to sign a JWT as the client secret per Apple's protocol — unlike every other provider, you don't paste a secret Apple issued to you, you generate one on every token exchange.

For Fly.io, store the key with `fly secrets set REVO_AUTH_APPLE_PRIVATE_KEY="$(cat AuthKey_ABCDE67890.p8)"` — Fly preserves newlines.

## 6. Enable the provider

```sh
pnpm dlx @revo-auth/cli add provider apple
```

"Sign in with Apple" appears on your login page.

## Name and email on first sign-in

Apple only sends the user's name and email on the **very first** authorization for your app. Subsequent logins return only the stable user ID. Revo-Auth persists the name and email immediately — if the first attempt fails (network error, user closes the tab mid-flow), you've lost them permanently for that account. The user has to revoke the app in Apple ID settings and sign in again.

## Private relay emails

Apple offers users a "Hide My Email" option — they sign in with a per-app relay address like `abc123@privaterelay.appleid.com`. Mail sent there forwards to their real address. Revo-Auth stores the relay address as-is; the user's actual email is never available to you. Plan for this if you do email marketing.

## Verifying

Click the button. Apple's sheet appears (different on Safari vs other browsers — Safari shows a passkey prompt, others show a QR or password flow). After authorizing, you're signed in. If it fails:

- Verify the Services ID, Team ID, Key ID, and private key are all consistent — one typo and the JWT signing fails.
- Check the return URL in the Services ID configuration matches exactly.
- Your domain must be verified via the file Apple asks you to host at `.well-known/apple-developer-domain-association.txt`. Revo-Auth serves this file automatically for the configured domain.
