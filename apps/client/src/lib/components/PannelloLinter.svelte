<script lang="ts">
  /**
   * Pannello "Linter" (Impostazioni). Mostra il catalogo delle regole dal
   * backend (`prompt_lint_regole`, fonte di verità unica) raggruppate per
   * categoria, con toggle a granularità singola regola + toggle famiglia.
   * Un avviso è nascosto se la famiglia (prefisso) O la singola regola (code)
   * è disabilitata — stessa semantica del backend `filtra_disabilitate`.
   */
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Switch from "./Switch.svelte";
  import Toast from "./Toast.svelte";
  import {
    CATEGORIE_LINTER,
    ETICHETTE,
    type CategoriaLinter,
    type ConfigLinter,
    type SeveritaLinter,
    type SoglieLinter,
    DEFAULT_SOGLIE,
    leggiConfig,
    salvaConfig,
    setSeverita,
    setSoglia,
    toggleRegola,
  } from "$lib/preferenze-linter";

  interface RegolaMeta {
    code: string;
    categoria: CategoriaLinter;
    severita_default: SeveritaLinter;
    titolo: string;
    descrizione: string;
    configurabile: boolean;
  }

  // Mappa regola → campo soglia editabile (le sole regole "configurabili").
  const SOGLIA_DI: Record<
    string,
    { campo: keyof SoglieLinter; label: string; min: number }
  > = {
    LEN001: { campo: "len_max_body", label: "Caratteri massimi", min: 1 },
    LEN002: { campo: "len_min_body", label: "Caratteri minimi", min: 0 },
    STY001: { campo: "ngram_threshold", label: "Ripetizioni minime", min: 2 },
  };

  const SEVERITA_OPZIONI: { valore: SeveritaLinter; label: string }[] = [
    { valore: "error", label: "Errore" },
    { valore: "warning", label: "Avviso" },
    { valore: "info", label: "Info" },
  ];

  let catalogo = $state<RegolaMeta[]>([]);
  let cfg = $state<ConfigLinter>({
    disabilitate: [],
    severita_override: {},
    soglie: { ...DEFAULT_SOGLIE },
  });
  let toastVisibile = $state(false);
  let toastTesto = $state("");
  let errore = $state(false);

  onMount(async () => {
    cfg = leggiConfig();
    try {
      catalogo = await invoke<RegolaMeta[]>("prompt_lint_regole");
    } catch (e) {
      console.error("[linter] catalogo regole", e);
      errore = true;
    }
  });

  const perCategoria = $derived(
    CATEGORIE_LINTER.map((cat) => ({
      cat,
      regole: catalogo.filter((r) => r.categoria === cat),
    })).filter((g) => g.regole.length > 0),
  );

  // Quante personalizzazioni attive (per il pulsante "Ripristina tutto").
  const personalizzazioni = $derived(
    cfg.disabilitate.length +
      Object.keys(cfg.severita_override).length +
      (cfg.soglie.len_max_body !== DEFAULT_SOGLIE.len_max_body ? 1 : 0) +
      (cfg.soglie.len_min_body !== DEFAULT_SOGLIE.len_min_body ? 1 : 0) +
      (cfg.soglie.ngram_threshold !== DEFAULT_SOGLIE.ngram_threshold ? 1 : 0),
  );

  function showToast(testo: string): void {
    toastTesto = testo;
    toastVisibile = true;
    setTimeout(() => (toastVisibile = false), 1500);
  }

  // Salva + notifica DiagnosiTab di ri-lintare (altrimenti i risultati restano
  // stale finché l'utente non riedita il body).
  function salvaENotifica(): void {
    salvaConfig(cfg);
    window.dispatchEvent(new CustomEvent("pap:linter-config-cambiata"));
  }

  function famigliaAttiva(cat: CategoriaLinter): boolean {
    return !cfg.disabilitate.includes(cat);
  }

  function regolaAttiva(code: string): boolean {
    return !cfg.disabilitate.includes(code);
  }

  function severitaCorrente(r: RegolaMeta): SeveritaLinter {
    return cfg.severita_override[r.code] ?? r.severita_default;
  }

  function toggleFamiglia(cat: CategoriaLinter): void {
    cfg = { ...cfg, disabilitate: toggleRegola(cat, cfg.disabilitate) };
    salvaENotifica();
    showToast(
      `Famiglia ${cat} ${famigliaAttiva(cat) ? "attivata" : "disattivata"}`,
    );
  }

  function toggleSingola(r: RegolaMeta): void {
    cfg = { ...cfg, disabilitate: toggleRegola(r.code, cfg.disabilitate) };
    salvaENotifica();
    showToast(`${r.code} ${regolaAttiva(r.code) ? "attivata" : "disattivata"}`);
  }

  function cambiaSeverita(r: RegolaMeta, sev: SeveritaLinter): void {
    cfg = setSeverita(cfg, r.code, sev, r.severita_default);
    salvaENotifica();
  }

  function cambiaSoglia(
    campo: keyof SoglieLinter,
    valore: number,
    min: number,
  ): void {
    // Applica il minimo per-campo (allineato al clamp backend) così il valore
    // salvato non diverge da quello effettivamente usato.
    cfg = setSoglia(cfg, campo, valore, min);
    salvaENotifica();
  }

  // Ripristina la singola regola: rimuove override severità + azzera la sua
  // soglia (se ne ha una) al default.
  function ripristinaRegola(r: RegolaMeta): void {
    let next = setSeverita(cfg, r.code, r.severita_default, r.severita_default);
    const s = SOGLIA_DI[r.code];
    if (s) next = setSoglia(next, s.campo, DEFAULT_SOGLIE[s.campo]);
    cfg = next;
    salvaENotifica();
    showToast(`${r.code} ripristinata`);
  }

  function riattivaTutte(): void {
    cfg = {
      disabilitate: [],
      severita_override: {},
      soglie: { ...DEFAULT_SOGLIE },
    };
    salvaENotifica();
    showToast("Tutte le impostazioni ripristinate");
  }

  // La regola è "personalizzata" (mostra il pulsante ripristina) se ha un
  // override severità o una soglia diversa dal default.
  function regolaPersonalizzata(r: RegolaMeta): boolean {
    if (r.code in cfg.severita_override) return true;
    const s = SOGLIA_DI[r.code];
    return s ? cfg.soglie[s.campo] !== DEFAULT_SOGLIE[s.campo] : false;
  }
</script>

<div class="pannello">
  <h3 class="titolo">Linter — regole attive</h3>
  <p class="desc">
    Disattiva i singoli avvisi (o un'intera famiglia), cambia la loro severità
    o regola le soglie numeriche. Le impostazioni sono locali a questo
    dispositivo (no sync server).
  </p>

  {#if errore}
    <p class="msg-err">
      Impossibile caricare il catalogo delle regole dal backend.
    </p>
  {/if}

  {#each perCategoria as gruppo (gruppo.cat)}
    {@const famAttiva = famigliaAttiva(gruppo.cat)}
    <section class="categoria">
      <header class="cat-head">
        <span class="cat-nome">{ETICHETTE[gruppo.cat]}</span>
        <Switch
          attivo={famAttiva}
          etichetta="Famiglia {ETICHETTE[gruppo.cat]}"
          onchange={() => toggleFamiglia(gruppo.cat)}
        />
      </header>

      <ul class="lista" class:famiglia-off={!famAttiva} role="list">
        {#each gruppo.regole as r (r.code)}
          {@const eff = severitaCorrente(r)}
          {@const sogliaInfo = SOGLIA_DI[r.code]}
          {@const ctrlOff = !famAttiva || !regolaAttiva(r.code)}
          <li class="regola">
            <div class="regola-info">
              <div class="regola-top">
                <span class="regola-titolo">{r.titolo}</span>
                <code class="regola-code">{r.code}</code>
                <span class="badge sev-{eff}">{eff}</span>
              </div>
              <div class="regola-desc">{r.descrizione}</div>
              <div class="regola-controlli">
                <label class="ctrl">
                  <span class="ctrl-label">Severità</span>
                  <select
                    class="select"
                    value={eff}
                    disabled={ctrlOff}
                    onchange={(e) =>
                      cambiaSeverita(
                        r,
                        e.currentTarget.value as SeveritaLinter,
                      )}
                  >
                    {#each SEVERITA_OPZIONI as opt (opt.valore)}
                      <option value={opt.valore}>{opt.label}</option>
                    {/each}
                  </select>
                </label>

                {#if sogliaInfo}
                  <label class="ctrl">
                    <span class="ctrl-label">{sogliaInfo.label}</span>
                    <input
                      class="num"
                      type="number"
                      min={sogliaInfo.min}
                      step="1"
                      value={cfg.soglie[sogliaInfo.campo]}
                      disabled={ctrlOff}
                      onchange={(e) =>
                        cambiaSoglia(
                          sogliaInfo.campo,
                          e.currentTarget.valueAsNumber,
                          sogliaInfo.min,
                        )}
                    />
                  </label>
                {/if}

                {#if regolaPersonalizzata(r)}
                  <button
                    type="button"
                    class="reset-regola"
                    disabled={ctrlOff}
                    onclick={() => ripristinaRegola(r)}
                  >
                    Ripristina
                  </button>
                {/if}
              </div>
            </div>
            <Switch
              attivo={regolaAttiva(r.code)}
              disabled={!famAttiva}
              etichetta="Regola {r.code}"
              onchange={() => toggleSingola(r)}
            />
          </li>
        {/each}
      </ul>
      {#if !famAttiva}
        <p class="nota-fam">
          Intera famiglia disattivata: gli avvisi sono nascosti a prescindere
          dalle singole regole.
        </p>
      {/if}
    </section>
  {/each}

  {#if personalizzazioni > 0}
    <div class="actions">
      <button type="button" class="link-btn" onclick={riattivaTutte}>
        Ripristina tutto ({personalizzazioni}
        {personalizzazioni === 1 ? "modifica" : "modifiche"})
      </button>
    </div>
  {/if}
</div>

<Toast visibile={toastVisibile}>{toastTesto}</Toast>

<style>
  .pannello {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3, 1rem);
  }
  .titolo {
    margin: 0;
    font-size: var(--fs-base, 1.125rem);
    font-weight: var(--fw-semibold, 600);
  }
  .desc {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--fs-sm);
    line-height: 1.5;
  }
  .categoria {
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
    overflow: hidden;
  }
  .cat-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
  }
  .cat-nome {
    font-weight: var(--fw-semibold);
    color: var(--text-strong);
    font-size: var(--fs-sm);
  }
  .lista {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }
  .lista.famiglia-off {
    opacity: 0.5;
  }
  .regola {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-top: 1px solid var(--border-subtle);
  }
  .regola:first-child {
    border-top: none;
  }
  .regola-info {
    flex: 1;
    min-width: 0;
  }
  .regola-top {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .regola-titolo {
    font-weight: var(--fw-medium);
    color: var(--text-strong);
    font-size: var(--fs-sm);
  }
  .regola-code {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    background: var(--bg-input);
    border-radius: 4px;
    padding: 1px 5px;
  }
  .regola-desc {
    color: var(--text-muted);
    font-size: var(--fs-xs);
    margin-top: 3px;
    line-height: 1.4;
  }
  .regola-controlli {
    display: flex;
    align-items: flex-end;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 8px;
  }
  .ctrl {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .ctrl-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-subtle);
  }
  .select,
  .num {
    font-family: var(--font-ui);
    font-size: var(--fs-xs);
    color: var(--text-default);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: 3px 6px;
  }
  .num {
    width: 88px;
  }
  .select:disabled,
  .num:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .reset-regola {
    background: none;
    border: none;
    color: var(--accent-team);
    font-size: var(--fs-xs);
    cursor: pointer;
    padding: 3px 2px;
    text-decoration: underline;
  }
  .reset-regola:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    text-decoration: none;
  }
  .reset-regola:not(:disabled):hover {
    color: var(--accent-team-strong);
  }
  .badge {
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.02em;
    border-radius: 4px;
    padding: 0 5px;
    border: 1px solid transparent;
  }
  .sev-error {
    color: var(--danger, #e5484d);
    border-color: color-mix(in srgb, var(--danger, #e5484d) 40%, transparent);
  }
  .sev-warning {
    color: var(--warning, #f5a623);
    border-color: color-mix(in srgb, var(--warning, #f5a623) 40%, transparent);
  }
  .sev-info {
    color: var(--text-muted);
    border-color: var(--border-default);
  }
  .nota-fam {
    margin: 0;
    padding: 8px 14px;
    font-size: var(--fs-xs);
    color: var(--text-subtle);
    background: var(--bg-surface);
  }
  .msg-err {
    margin: 0;
    color: var(--danger, #e5484d);
    font-size: var(--fs-sm);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  .link-btn {
    background: none;
    border: none;
    color: var(--accent-team);
    font-size: var(--fs-xs);
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
  }
  .link-btn:hover {
    color: var(--accent-team-strong);
  }
</style>
