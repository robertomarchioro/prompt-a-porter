import { computed, onMounted, ref, type ComputedRef, type Ref } from 'vue'

// Rilevamento OS del visitatore per personalizzare le CTA di download
// ("Scarica per Windows/macOS/Linux"). Il sito è prerenderizzato (SSG):
// `navigator` esiste solo nel browser, quindi il valore parte neutro e
// viene aggiornato in onMounted — niente hydration mismatch, e il testo
// neutro resta per no-JS, mobile e OS non riconosciuti.

export type OsDesktop = 'windows' | 'macos' | 'linux'

const NOMI_OS: Record<OsDesktop, string> = {
  windows: 'Windows',
  macos: 'macOS',
  linux: 'Linux',
}

// Pura per testabilità/chiarezza: ritorna null per piattaforme senza una
// build desktop da offrire (Android, iOS/iPadOS, sconosciute).
export function classificaPiattaforma(
  userAgent: string,
  platform: string,
  maxTouchPoints: number,
): OsDesktop | null {
  if (/Android/i.test(userAgent)) return null
  if (/iPhone|iPad|iPod/i.test(platform)) return null
  // iPadOS moderno espone platform "MacIntel": lo distingue il multi-touch.
  if (/Mac/i.test(platform)) return maxTouchPoints > 1 ? null : 'macos'
  if (/Win/i.test(platform)) return 'windows'
  if (/Linux/i.test(platform)) return 'linux'
  return null
}

/**
 * Etichette delle scorciatoie tastiera coerenti con l'OS del visitatore:
 * simboli Mac (⌃⇧P) solo su macOS; Windows, Linux, mobile, no-JS e OS
 * ignoti vedono lo stile "Ctrl + Shift + P" (default SSG, quindi niente
 * hydration mismatch — su Mac l'etichetta si aggiorna in onMounted).
 */
export interface Scorciatoie {
  paletta: ComputedRef<string>
  palettaCompatta: ComputedRef<string>
  compilaIncolla: ComputedRef<string>
}

export function useScorciatoie(): Scorciatoie {
  const isMac = ref(false)
  onMounted(() => {
    isMac.value =
      classificaPiattaforma(
        navigator.userAgent ?? '',
        navigator.platform ?? '',
        navigator.maxTouchPoints ?? 0,
      ) === 'macos'
  })
  return {
    paletta: computed(() => (isMac.value ? '⌃ ⇧ P' : 'Ctrl + Shift + P')),
    palettaCompatta: computed(() => (isMac.value ? '⌃⇧P' : 'Ctrl+Shift+P')),
    compilaIncolla: computed(() => (isMac.value ? '⌃↵' : 'Ctrl+↵')),
  }
}

/**
 * Nome leggibile dell'OS desktop del visitatore ("Windows" | "macOS" |
 * "Linux"), o null se ignoto/mobile — in quel caso la CTA usa il testo
 * neutro.
 */
export function useNomeOsVisitatore(): Ref<string | null> {
  const nome = ref<string | null>(null)
  onMounted(() => {
    const os = classificaPiattaforma(
      navigator.userAgent ?? '',
      navigator.platform ?? '',
      navigator.maxTouchPoints ?? 0,
    )
    nome.value = os === null ? null : NOMI_OS[os]
  })
  return nome
}
