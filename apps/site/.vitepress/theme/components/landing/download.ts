import { onMounted, ref, type Ref } from 'vue'
import { RELEASES } from './links'
import { classificaPiattaforma, type OsDesktop } from './os'

// Link diretto all'installer dell'ultima release per l'OS del visitatore.
// I nomi degli asset contengono la versione (es. `…_0.8.36_x64-setup.exe`),
// quindi non esiste un URL stabile per-OS: si interroga l'API GitHub
// `releases/latest` (CORS aperto, 60 req/h per IP — ampiamente sufficiente
// per una landing) e si sceglie l'asset col pattern giusto. In qualsiasi
// caso di errore — API giù, rate limit, asset mancante, OS ignoto/mobile —
// il link resta la pagina Releases: la CTA non si rompe mai.

const API_LATEST =
  'https://api.github.com/repos/robertomarchioro/prompt-a-porter/releases/latest'

// Pattern dell'installer per OS (il `$` esclude le firme `.sig`).
// Linux: AppImage — il formato distro-agnostico, primo nella doc utente.
const PATTERN_ASSET: Record<OsDesktop, RegExp> = {
  windows: /_x64-setup\.exe$/i,
  macos: /_universal\.dmg$/i,
  linux: /\.AppImage$/i,
}

interface AssetRelease {
  name?: unknown
  browser_download_url?: unknown
}

// Una sola fetch condivisa fra tutte le CTA della pagina.
let assetsPromise: Promise<AssetRelease[]> | null = null

function caricaAssets(): Promise<AssetRelease[]> {
  assetsPromise ??= fetch(API_LATEST, {
    headers: { Accept: 'application/vnd.github+json' },
  })
    .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
    .then((json: { assets?: unknown }) =>
      Array.isArray(json?.assets) ? (json.assets as AssetRelease[]) : [],
    )
  return assetsPromise
}

async function risolviUrlInstaller(os: OsDesktop): Promise<string | null> {
  const assets = await caricaAssets()
  const pattern = PATTERN_ASSET[os]
  for (const a of assets) {
    if (
      typeof a?.name === 'string' &&
      typeof a?.browser_download_url === 'string' &&
      pattern.test(a.name)
    ) {
      return a.browser_download_url
    }
  }
  return null
}

/**
 * URL di download per la CTA: parte dalla pagina Releases (valida sempre,
 * anche in SSR/no-JS) e diventa il link diretto all'installer quando l'OS
 * del visitatore è riconosciuto e l'asset esiste.
 */
export function useLinkDownload(): Ref<string> {
  const link = ref(RELEASES)
  onMounted(() => {
    const os = classificaPiattaforma(
      navigator.userAgent ?? '',
      navigator.platform ?? '',
      navigator.maxTouchPoints ?? 0,
    )
    if (os === null) return
    risolviUrlInstaller(os)
      .then((url) => {
        if (url) link.value = url
      })
      .catch(() => {
        // Fallback silenzioso: il link resta la pagina Releases.
      })
  })
  return link
}
