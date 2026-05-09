# Blueprint F9 — Routing/cleanup + Onboarding consolidato

> Cumulativo per le 3 sub-PR autonome F9. Riferimenti:
> [Piano F9](../redesign-v08.md#f9--routingcleanup) (5 gg FT).

## Obiettivo

Sostituire il routing attuale a 5 rami (`?redesign-shell`, `?demo`, label palette, onboarding, libreria) con un flusso a 2 stati:

```
+----------------+              +-----------------+
| App.svelte     |  --not-auth->| Onboarding.svelte
| (routing root) |              | (consolidato)   |
|                |              +-----------------+
|                |  --auth----->+-----------------+
|                |              | Shell.svelte    |
+----------------+              +-----------------+
```

I 5 modali (F8) restano gestiti dallo store globale `modale.svelte.ts` già esistente (creato in F8 PR-A; il piano originale lo chiamava `modalState.ts` ma il nome adottato in F8 prevale).

## Sub-PR

| Sub-PR | Scope | Stima |
|---|---|---|
| **F9 PR-A** | `Onboarding.svelte` consolidato (login + recupera + reset + wizard + vault unlock) | 2 gg |
| **F9 PR-B** | `App.svelte` routing nuovo (default: Shell+Onboarding) + DELETE `DemoComponenti.svelte` | 2 gg |
| **F9 PR-C** | Cleanup superfici legacy: DELETE 4 Auth* + Libreria.svelte (se Shell copre tutto) | 1 gg |

---

## 1. F9 PR-A — Onboarding consolidato

### Path

`apps/client/src/lib/superfici/Onboarding.svelte` (NEW)

### Scope

Component nuovo che assorbe 4 superfici legacy (totale ~1581 righe → target ~700 righe):

| Legacy file | Righe | Funzione |
|---|---|---|
| `AuthLogin.svelte` | 294 | Login workspace remoto |
| `AuthResetPassword.svelte` | 355 | Reset password via token email |
| `AuthRecuperaWorkspace.svelte` | 399 | Recupero workspace email |
| `OnboardingWizard.svelte` | 533 | Setup iniziale vault locale + nome + master pwd |

### Stati gestiti (state machine)

```
caricamento ─► blocco ─► sblocco-pwd ─► aperto (→ App segnala auth=true)
              ▲
              │
            wizard ─► nuovo-vault ─► aperto
                      ─► join-workspace ─► login-remoto ─► aperto
              
            login-remoto ─► (link "recupera workspace") ─► recupera ─► (← back)
            login-remoto ─► (link "reset password") ─► reset ─► (← back)
```

### Decisioni design

- **Stato unico `passo`** (discriminated union) invece di 4 component separati.
- **Layout**: container centrato `max-width: 480px`, header con logo + nome app, footer con switch tema/lingua minimal.
- **Wizard riutilizza i 5 step legacy** semplificati: benvenuto → master pwd → conferma pwd → modalità (locale/remoto) → completato.
- **Cmd Tauri invariati**:
  - `vault_apri({password})` per unlock
  - `vault_crea({password, profilo})` per nuovo vault
  - `auth_login`, `auth_recupera_workspace`, `auth_reset_password` (esistenti)

### Props

```typescript
interface Props {
  oncompletato: () => void; // notifica App quando auth=true
}
```

### Stima

2 gg

---

## 2. F9 PR-B — App.svelte routing nuovo + DELETE DemoComponenti

### Path

`apps/client/src/App.svelte` (riscritto)

### Scope

Riscrittura completa di App.svelte per nuovo routing:

```svelte
{#if etichetta === "palette"}
  <CommandPalette /> <!-- window legacy: rimossa solo in F11 -->
{:else if !authCompleted}
  <Onboarding oncompletato={() => authCompleted = true} />
{:else}
  <Shell />
{/if}
```

Cambiamenti rispetto al routing attuale:

- ❌ Rimosso `?demo` (DemoComponenti DELETED)
- ❌ Rimosso `?redesign-shell` (Shell è il default per auth)
- ❌ Rimosso `<Libreria />` come default (sostituito da `<Shell />`)
- ✅ Mantenuto `etichetta === "palette"` per la window separata legacy (CommandPalette ha già il suo replacement modale via PaletteModal F8 PR-E; la window legacy resta come fallback hotkey globale OS-level, sarà rimossa in F11)
- ✅ Rinominato `onboardingCompletato` → `authCompleted` per chiarezza

### File DELETED in PR-B

- `apps/client/src/lib/superfici/DemoComponenti.svelte` (273 righe — kill, era solo dev-tool)

### Cmd Tauri

Nessuno nuovo, riusa logiche esistenti.

### Stima

2 gg

---

## 3. F9 PR-C — Cleanup legacy (DELETE 4 Auth* + Libreria)

### Path

DELETE-only. Nessun NEW file.

### File DELETED

| File | Righe | Sostituito da |
|---|---|---|
| `apps/client/src/lib/superfici/AuthLogin.svelte` | 294 | Onboarding (PR-A) |
| `apps/client/src/lib/superfici/AuthResetPassword.svelte` | 355 | Onboarding (PR-A) |
| `apps/client/src/lib/superfici/AuthRecuperaWorkspace.svelte` | 399 | Onboarding (PR-A) |
| `apps/client/src/lib/superfici/OnboardingWizard.svelte` | 533 | Onboarding (PR-A) |
| `apps/client/src/lib/superfici/Libreria.svelte` | 2418 | Shell (F1-F7) + 5 modali (F8) |

**Totale cancellato:** ~3999 righe.

### Verifiche pre-DELETE per Libreria

Prima di cancellare Libreria.svelte verifica che Shell copra:

1. ✅ Lista prompt + filtri (vista/folder/tag/model) → ListPane (F3)
2. ✅ Detail prompt 5 tab → DetailPane (F4-F5)
3. ✅ Compilatore → CompilaModal (F8 PR-A)
4. ✅ Cronologia / Confronto → tab in DetailPane (F5 PR-D/F)
5. ✅ Insight / Regressioni → modali (F8 PR-B/C)
6. ✅ Impostazioni → ImpostazioniModal (F8 PR-D1+D2)
7. ✅ Palette ricerca → PaletteModal (F8 PR-E)
8. ⚠️ **Workspace switcher** (placeholder F2) — Libreria ha logica login/logout workspace. Verifica che Shell.workspaceSwitcher placeholder sia sufficiente per ora; se manca login → rimanda DELETE Libreria a F11 e in PR-C cancella solo i 4 Auth*.

### Stima

1 gg (solo cleanup + verifiche)

---

## 4. Pattern comune di integrazione

Per ogni sub-PR:

1. Per file NEW: usa pattern primitive Modale (F8) per consistency styling
2. Per file DELETE: rimuovi import + reference da Libreria (se ancora referenziata) PRIMA di cancellare
3. Verifica `npm run check` 0 errors + `npm test` 88/88 dopo ogni edit
4. Commit + push + PR autonoma + Monitor CI + auto-merge appena verde

### Out of scope F9

- Workspace switcher login/logout funzionale (placeholder F2 mantenuto)
- E2E test (no E2E suite esiste oggi; pianificato in F11)
- Window separata `palette` Tauri (rimossa solo in F11 cleanup)
- ConflittoSync.svelte modale aux (resta come legacy event-driven)

---

> **Stato blueprint**: 1.0 — pronto per esecuzione iterativa autonoma.
