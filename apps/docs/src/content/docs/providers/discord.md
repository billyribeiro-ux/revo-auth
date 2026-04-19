---
title: Discord
description: Enable Discord OAuth for Revo-Auth — Developer Portal setup, redirect URIs, and env vars.
---

Discord OAuth is the right call for gaming communities, creator tooling, and anything where your users already live in Discord. Setup is short.

## 1. Create a Discord application

Open the [Discord Developer Portal](https://discord.com/developers/applications), click **New Application**, give it a name. The name is what users see on the consent screen.

Under **OAuth2 → General**:

- Copy the **Client ID**.
- Click **Reset Secret** (or **Copy**) to get the **Client Secret**.
- Under **Redirects**, add:
  ```
  https://<your-revo-auth-server>/v1/oauth/discord/callback
  ```

![Discord Developer Portal — OAuth2 settings with redirect URL filled in](/screenshots/providers/discord-01-oauth.png)

*Screenshot placeholder: Discord's OAuth2 settings page.*

Save changes.

## 2. Set env vars on the Revo-Auth server

```sh
REVO_AUTH_DISCORD_CLIENT_ID="..."
REVO_AUTH_DISCORD_CLIENT_SECRET="..."
```

Standard secret management for your deployment target (Fly, Docker, systemd env file).

## 3. Enable the provider

```sh
pnpm dlx @revo-auth/cli add provider discord
```

## Scopes

Revo-Auth requests `identify` and `email` by default. That gives you the Discord user ID, username, avatar, and verified email address.

Add scopes via the dashboard if you need more — `guilds` for server membership, `connections` for linked accounts, etc. Users will see the expanded consent screen on next login.

## Email verification

Discord requires email verification to surface the email on the OAuth response. Users who haven't verified will sign in with a null email; Revo-Auth rejects those with `email_required` unless you've explicitly opted into anonymous-email accounts in the dashboard (rarely a good idea).

## Avatar and username

Discord avatars change. Revo-Auth fetches the avatar URL on every login and updates the `users.avatar_url` column, so stale avatars self-heal. Usernames follow the same refresh pattern. If you cache them in your own database, set a short TTL.

## Verifying

Click the button. You'll see Discord's "Authorize" screen with your app's icon and name. After authorizing, you're returned signed in. Failures are usually one of:

- Redirect URL not added in the Developer Portal.
- Secret mismatch — if you've reset the secret, update your env vars.
- User's email isn't verified on Discord — they have to verify before they can sign in.
