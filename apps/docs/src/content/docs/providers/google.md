---
title: Google
description: Enable Google OAuth for Revo-Auth — Cloud Console setup, redirect URIs, and env vars.
---

Google OAuth is the default choice for consumer apps. Setup lives in Google Cloud Console; the flow takes about ten minutes end to end.

## 1. Create a Cloud Console project

Open [console.cloud.google.com](https://console.cloud.google.com/) and create a new project (or reuse one). Note the project name — you'll see it in consent screens.

![Google Cloud Console — new project screen](/screenshots/providers/google-01-new-project.png)

*Screenshot placeholder: new project dialog in Google Cloud Console.*

## 2. Configure the OAuth consent screen

Navigate to **APIs & Services → OAuth consent screen**. Pick **External** (unless you're a Workspace tenant locking to your domain). Fill the app name, support email, and developer contact. Add the scopes `openid`, `email`, `profile`.

Publish the app (or add test users if you want to keep it in testing mode for now).

## 3. Create OAuth credentials

**APIs & Services → Credentials → Create credentials → OAuth client ID**. Pick **Web application**.

- **Name:** anything — it's internal.
- **Authorized redirect URIs:** add the Revo-Auth callback:
  ```
  https://<your-revo-auth-server>/v1/oauth/google/callback
  ```
  For local dev against `revo-auth dev`, that's `http://localhost:4000/v1/oauth/google/callback`.

![Google Cloud Console — OAuth client ID form with redirect URI filled in](/screenshots/providers/google-02-redirect-uri.png)

*Screenshot placeholder: OAuth client ID creation form.*

Save. Copy the **Client ID** and **Client secret** — you won't see the secret again without regenerating.

## 4. Set env vars on the Revo-Auth server

```sh
REVO_AUTH_GOOGLE_CLIENT_ID="..."
REVO_AUTH_GOOGLE_CLIENT_SECRET="..."
```

These live on the **server**, not your SvelteKit app. If you're using Fly.io, `fly secrets set REVO_AUTH_GOOGLE_CLIENT_ID=... REVO_AUTH_GOOGLE_CLIENT_SECRET=...`. Docker, use an env file. Local dev, add them to `.revo-auth/dev.env`.

## 5. Enable the provider

```sh
pnpm dlx @revo-auth/cli add provider google
```

Or toggle the provider in the dashboard. The "Sign in with Google" button will appear on your login page immediately — no code changes needed on the SvelteKit side.

## Verifying

Click the button. You should be bounced to `accounts.google.com`, grant consent, and land back in your app signed in. If you land in an error, check:

- Redirect URI matches exactly — scheme, host, port, path. Google is strict.
- Scopes include `email` and `profile`.
- Your consent screen is published or your Google account is listed as a test user.

Run `pnpm dlx @revo-auth/cli doctor` — it checks provider reachability and flags common misconfigurations.
