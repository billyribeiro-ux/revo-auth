import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  site: "https://docs.revo-auth.dev",
  integrations: [
    starlight({
      title: "Revo-Auth",
      description:
        "Production-grade, self-hostable authentication for SvelteKit.",
      logo: {
        src: "./src/assets/logo.svg",
        replacesTitle: false,
      },
      social: {
        github: "https://github.com/revo-auth/revo-auth",
      },
      customCss: ["./src/styles/tokens.css", "./src/styles/docs.css"],
      editLink: {
        baseUrl: "https://github.com/revo-auth/revo-auth/edit/main/apps/docs/",
      },
      lastUpdated: true,
      pagination: true,
      sidebar: [
        {
          label: "Start",
          items: [
            { label: "Quickstart", link: "/quickstart/" },
            {
              label: "Concepts",
              items: [
                { label: "Sessions", link: "/concepts/sessions/" },
                { label: "Multi-tenancy", link: "/concepts/multi-tenancy/" },
                { label: "Passkeys", link: "/concepts/passkeys/" },
                { label: "MFA", link: "/concepts/mfa/" },
                { label: "RBAC", link: "/concepts/rbac/" },
                { label: "Audit log", link: "/concepts/audit-log/" },
              ],
            },
          ],
        },
        {
          label: "SvelteKit",
          items: [
            { label: "Overview", link: "/sveltekit/" },
            { label: "Hooks", link: "/sveltekit/hooks/" },
            { label: "Load guards", link: "/sveltekit/load-guards/" },
            { label: "Session store", link: "/sveltekit/session-store/" },
            { label: "SSR vs CSR", link: "/sveltekit/ssr-vs-csr/" },
          ],
        },
        {
          label: "CLI",
          items: [
            { label: "Overview", link: "/cli/" },
            { label: "init", link: "/cli/init/" },
            { label: "add", link: "/cli/add/" },
            { label: "update", link: "/cli/update/" },
            { label: "ui", link: "/cli/ui/" },
            { label: "dev", link: "/cli/dev/" },
            { label: "doctor", link: "/cli/doctor/" },
            { label: "logout", link: "/cli/logout/" },
          ],
        },
        {
          label: "Providers",
          items: [
            { label: "Overview", link: "/providers/" },
            { label: "Google", link: "/providers/google/" },
            { label: "GitHub", link: "/providers/github/" },
            { label: "Microsoft", link: "/providers/microsoft/" },
            { label: "Discord", link: "/providers/discord/" },
            { label: "Apple", link: "/providers/apple/" },
          ],
        },
        {
          label: "Deployment",
          items: [
            { label: "Fly.io", link: "/deployment/fly-io/" },
            { label: "Docker", link: "/deployment/docker/" },
            { label: "Database", link: "/deployment/database/" },
            { label: "Secrets", link: "/deployment/secrets/" },
          ],
        },
      ],
      components: {},
    }),
  ],
});
