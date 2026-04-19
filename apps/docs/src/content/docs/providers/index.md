---
title: OAuth providers
description: How Revo-Auth supports Google, GitHub, Microsoft, Discord, and Apple — plus the shape of the per-provider guides.
---

Revo-Auth ships first-class OAuth support for Google, GitHub, Microsoft, Discord, and Apple. Each provider has its own dedicated guide in this section covering client-ID creation, redirect URIs, and the env vars the CLI looks for.

## The common shape

Every provider guide walks the same three steps:

1. **Create credentials in the provider's portal.** Different consoles, same goal: get a client ID, a client secret, and (sometimes) set up a redirect URI.
2. **Set the env vars.** Revo-Auth reads per-provider keys like `REVO_AUTH_GOOGLE_CLIENT_ID` and `REVO_AUTH_GOOGLE_CLIENT_SECRET`. These live on the **server**, not in your SvelteKit app's env — the OAuth flow is server-mediated so the secret never reaches the browser.
3. **Enable in the dashboard (or via the CLI).** `pnpm dlx @revo-auth/cli add provider <name>` or toggle in the dashboard. The provider appears on the login page once enabled.

## Redirect URIs

The redirect URI is always:

```
https://<your-revo-auth-server>/v1/oauth/<provider>/callback
```

For the hosted dev instance, that's `https://auth.revo-auth.dev/v1/oauth/google/callback`. For your own self-hosted deployment, swap the hostname. You'll paste this exact URL into each provider's portal.

Revo-Auth *never* calls back to your SvelteKit app directly. The server receives the OAuth code, exchanges it for tokens, fetches the user profile, creates or updates the user, issues a session, and then redirects the browser to your app's `returnTo` URL with the cookie set. Your app never sees the provider's access token unless you explicitly ask for it via the API.

## Scopes

Revo-Auth requests the minimum scopes needed to get a stable user ID, email, and display name. You can request additional scopes per provider in the dashboard — those broader grants are then available to your server via `sdk.oauth.getAccessToken(userId, provider)` if you need to call provider APIs on behalf of the user.

## Choosing providers

- **Google** — biggest consumer reach, mandatory for most B2C apps.
- **GitHub** — essential for developer tools, weak for general consumer.
- **Microsoft** — covers Entra ID (the artist formerly known as Azure AD); essential for enterprise.
- **Discord** — gaming, creator communities, younger demographics.
- **Apple** — required by App Store rules if you offer any other OAuth in an iOS app.

You don't have to enable all of them. Start with the one your users actually use.
