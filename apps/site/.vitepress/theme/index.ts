import { defineComponent, h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import { useData } from 'vitepress'
import type { Theme } from 'vitepress'
import Landing from './components/landing/Landing.vue'
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
} satisfies Theme
