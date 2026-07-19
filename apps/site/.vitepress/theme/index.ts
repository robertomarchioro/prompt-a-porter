import { defineComponent, h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import { useData } from 'vitepress'
import type { Theme } from 'vitepress'
import Landing from './components/landing/Landing.vue'
// Font self-hosted (§2.3 istruzioni landing): niente CDN Google Fonts —
// privacy (IP verso terzi) oltre che performance. Pesi usati dal design.
import '@fontsource/newsreader/300.css'
import '@fontsource/newsreader/400.css'
import '@fontsource/newsreader/300-italic.css'
import '@fontsource/newsreader/400-italic.css'
import '@fontsource/inter/400.css'
import '@fontsource/inter/500.css'
import '@fontsource/inter/600.css'
import '@fontsource/jetbrains-mono/400.css'
import '@fontsource/jetbrains-mono/500.css'
import '@fontsource/jetbrains-mono/700.css'
import './styles/landing.css'

// `markdown.html: false` (config.ts) impedisce di montare componenti
// direttamente nei .md: il layout custom `landing` aggira il vincolo —
// la home dichiara `layout: landing` nel frontmatter e il Layout del tema
// rende la landing al posto del default theme.
const Layout = defineComponent({
  name: 'PapLayout',
  setup() {
    const { frontmatter } = useData()
    return () =>
      frontmatter.value.layout === 'landing' ? h(Landing) : h(DefaultTheme.Layout)
  },
})

export default {
  extends: DefaultTheme,
  Layout,
  enhanceApp({ router }) {
    // Matomo è iniettato in head solo in produzione (config.ts): dopo il
    // primo pageview, le navigazioni SPA vanno tracciate a mano.
    if (typeof window === 'undefined') return
    // L'hook scatta anche all'idratazione iniziale: quella pagina è già
    // tracciata dallo snippet in head → si deduplica sull'URL corrente.
    let ultimoUrl = window.location.pathname
    const originale = router.onAfterRouteChange
    router.onAfterRouteChange = (to: string) => {
      originale?.(to)
      if (to === ultimoUrl) return
      ultimoUrl = to
      const paq = (window as unknown as { _paq?: unknown[][] })._paq
      if (Array.isArray(paq)) {
        paq.push(['setCustomUrl', to])
        paq.push(['setDocumentTitle', document.title])
        paq.push(['trackPageView'])
      }
    }
  },
} satisfies Theme
