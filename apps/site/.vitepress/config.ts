import { defineConfig } from "vitepress";

// VitePress legge i .md direttamente da docs/ (root del monorepo).
// Sidebar e navbar filtrano cosa esporre nel sito pubblico.
//
// Sintassi PaP `{{nome}}`, `{{global ...}}`, `{{import "..."}}` nei docs:
// va SEMPRE scritta dentro backtick (code span/blocco) — VitePress applica
// v-pre automatico al codice, quindi resta testo letterale. NON ridefinire
// i delimitatori Vue globali (`vue.template.compilerOptions.delimiters`):
// si applicano anche agli SFC del default theme e ne rompono le
// interpolazioni (search box, outline, edit link → mustache letterali).
export default defineConfig({
  title: "Prompt a Porter",
  description:
    "Libreria locale per prompt AI — template parametrici, vault cifrato, sync opzionale",
  lang: "it",
  srcDir: "../../docs",
  base: "/prompt-a-porter/",
  cleanUrls: true,
  lastUpdated: true,

  // Sito pubblico = solo utente/, test/ e index. Le doc tecniche
  // (architettura, contribuire, operativo, roadmap) restano accessibili
  // su GitHub ma non vengono publicate al sito.
  srcExclude: [
    "README.md",
    "architettura/**",
    "contribuire/**",
    "operativo/**",
    "roadmap/**",
    // Ricette d'esempio: fuori tema per il sito del prodotto (scelta
    // 2026-07-12); restano leggibili su GitHub.
    "utente/casi-uso/**",
  ],

  // Disabilita parsing di tag HTML inline cosi' che frammenti come
  // `<prefix>`, `<id>`, `<data>` nei doc utente (placeholder di esempio)
  // non vengano interpretati come tag aperti senza chiusura dal compiler
  // Vue.
  markdown: {
    html: false,
    config(md) {
      // VitePress applica v-pre ai blocchi di codice recintati ma NON ai
      // code span inline: un `{{nome}}` inline verrebbe interpretato come
      // interpolazione Vue (build error o testo vuoto). Forziamo v-pre su
      // tutti gli inline code — è la ragione per cui NON servono delimitatori
      // Vue custom (che romperebbero il default theme, vedi sopra).
      const renderInlineCode =
        md.renderer.rules.code_inline ??
        ((tokens, idx, options, _env, self) =>
          `<code${self.renderAttrs(tokens[idx])}>${md.utils.escapeHtml(tokens[idx].content)}</code>`)
      md.renderer.rules.code_inline = (tokens, idx, options, env, self) =>
        renderInlineCode(tokens, idx, options, env, self).replace(/^<code/, "<code v-pre")
    },
  },

  // Mappa ogni `README.md` a `index.html` cosi' i path delle sezioni
  // (es. `/utente/`, `/utente/casi-uso/`) risolvono al README esistente
  // senza dover duplicare in `index.md`.
  rewrites: {
    "utente/README.md": "utente/index.md",
    "test/README.md": "test/index.md",
  },
  ignoreDeadLinks: [
    // Link interni a doc tecniche (architettura/operativo/roadmap/contribuire)
    // e alle ricette casi-uso, non inclusi nel sito pubblico utente.
    // Restano accessibili su GitHub.
    /\/(architettura|operativo|roadmap|contribuire)\//,
    /casi-uso/,
  ],

  // Nota: i font Google e i meta og: della landing sono nel frontmatter di
  // docs/index.md (head per-pagina), NON qui — l'head globale finirebbe
  // iniettato su ogni pagina di documentazione.
  head: [
    // Icone { P } definitive (palette viola) — asset in docs/public/icons/,
    // copiati dal handoff docs/roadmap/icons/violet/. Il site.webmanifest ha
    // path relativi: i file devono restare tutti nella stessa cartella.
    ["link", { rel: "icon", href: "/prompt-a-porter/icons/favicon.ico", sizes: "any" }],
    ["link", { rel: "icon", type: "image/png", sizes: "32x32", href: "/prompt-a-porter/icons/icon-32.png" }],
    ["link", { rel: "icon", type: "image/png", sizes: "16x16", href: "/prompt-a-porter/icons/icon-16.png" }],
    ["link", { rel: "apple-touch-icon", sizes: "180x180", href: "/prompt-a-porter/icons/apple-touch-icon.png" }],
    ["link", { rel: "manifest", href: "/prompt-a-porter/icons/site.webmanifest" }],
    ["meta", { name: "theme-color", content: "#646cff" }],
  ],

  themeConfig: {
    siteTitle: "Prompt a Porter",

    nav: [
      { text: "Guida", link: "/utente/getting-started" },
      { text: "Sintassi", link: "/utente/glossario-sintassi" },
      { text: "Test", link: "/test/" },
      {
        text: "GitHub",
        link: "https://github.com/robertomarchioro/prompt-a-porter",
      },
    ],

    sidebar: {
      "/utente/": [
        {
          text: "Inizio",
          items: [
            { text: "Cos'è Prompt a Porter", link: "/utente/" },
            { text: "Getting started", link: "/utente/getting-started" },
            { text: "Glossario sintassi", link: "/utente/glossario-sintassi" },
            { text: "Scorciatoie tastiera", link: "/utente/scorciatoie-tastiera" },
            { text: "Troubleshooting", link: "/utente/troubleshooting" },
          ],
        },
        {
          text: "Funzionalità avanzate",
          collapsed: true,
          items: [
            { text: "Prompt componibili (import)", link: "/utente/prompt-componibili" },
            { text: "Varianti A/B", link: "/utente/varianti-prompt" },
            { text: "Fork prompt", link: "/utente/fork-prompt" },
            { text: "Rating prompt", link: "/utente/rating-prompt" },
            { text: "Ricerca semantica", link: "/utente/ricerca-semantica" },
            { text: "Linting regole", link: "/utente/linting-regole" },
            { text: "Cartelle", link: "/utente/cartelle" },
            { text: "Regression testing", link: "/utente/regression-testing" },
          ],
        },
        {
          text: "Integrazioni",
          collapsed: true,
          items: [
            { text: "CLI pap", link: "/utente/cli" },
            { text: "MCP server", link: "/utente/mcp" },
            { text: "Export/Import JSON", link: "/utente/formato-export-json" },
            { text: "Markdown import/export", link: "/utente/markdown-import-export" },
            { text: "Auto-update", link: "/utente/auto-update" },
          ],
        },
      ],
      "/test/": [
        {
          text: "Test plan utente",
          items: [
            { text: "Introduzione", link: "/test/" },
            { text: "Catalogo test cases", link: "/test/test-cases" },
          ],
        },
      ],
    },

    socialLinks: [
      {
        icon: "github",
        link: "https://github.com/robertomarchioro/prompt-a-porter",
      },
    ],

    footer: {
      message:
        'Rilasciato sotto licenza <a href="https://github.com/robertomarchioro/prompt-a-porter/blob/main/LICENSE">AGPL-3.0-only</a>.',
      copyright: "Copyright © 2026 Roberto Marchioro",
    },

    editLink: {
      pattern:
        "https://github.com/robertomarchioro/prompt-a-porter/edit/main/docs/:path",
      text: "Modifica questa pagina su GitHub",
    },

    search: {
      provider: "local",
      options: {
        locales: {
          root: {
            translations: {
              button: {
                buttonText: "Cerca",
                buttonAriaLabel: "Cerca nei documenti",
              },
              modal: {
                noResultsText: "Nessun risultato per",
                resetButtonTitle: "Cancella query",
                footer: {
                  selectText: "per selezionare",
                  navigateText: "per navigare",
                  closeText: "per chiudere",
                },
              },
            },
          },
        },
      },
    },

    docFooter: {
      prev: "Pagina precedente",
      next: "Pagina successiva",
    },

    outline: {
      label: "In questa pagina",
    },

    lastUpdatedText: "Ultimo aggiornamento",

    darkModeSwitchLabel: "Tema",
    lightModeSwitchTitle: "Passa al tema chiaro",
    darkModeSwitchTitle: "Passa al tema scuro",
    sidebarMenuLabel: "Menu",
    returnToTopLabel: "Torna in cima",
  },
});
