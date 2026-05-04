// Dataset di test per Spike 3 — qualitative ranking embedding models
// per prompt PaP (uso reale: marketing, dev, business writing, technical).

export const prompts = [
  // === Email & business writing (IT) ===
  { id: 'p01', lang: 'it', body: 'Riscrivi questa email in tono formale mantenendo i punti chiave.' },
  { id: 'p02', lang: 'it', body: 'Trasforma il seguente messaggio in stile business professionale per un cliente.' },
  { id: 'p03', lang: 'it', body: 'Genera una email di follow-up cortese dopo riunione commerciale.' },
  { id: 'p04', lang: 'it', body: 'Scrivi una risposta diplomatica a un reclamo di un cliente arrabbiato.' },

  // === Email & business writing (EN) ===
  { id: 'p05', lang: 'en', body: 'Rewrite this email in a more formal tone while preserving key points.' },
  { id: 'p06', lang: 'en', body: 'Convert this Slack message into a polished business email.' },
  { id: 'p07', lang: 'en', body: 'Draft a polite follow-up email after a sales meeting.' },

  // === Code & dev (EN) ===
  { id: 'p08', lang: 'en', body: 'Convert this Python function to idiomatic TypeScript with full type annotations.' },
  { id: 'p09', lang: 'en', body: 'Generate a unit test for this function using vitest.' },
  { id: 'p10', lang: 'en', body: 'Refactor this code to use async/await instead of promises.' },
  { id: 'p11', lang: 'en', body: 'Explain this regex pattern step by step.' },

  // === Code & dev (IT) ===
  { id: 'p12', lang: 'it', body: 'Spiega cosa fa questa funzione Python come se fossi un giunior.' },
  { id: 'p13', lang: 'it', body: 'Aggiungi commenti JSDoc a questa funzione TypeScript.' },

  // === Analysis & summarization (IT) ===
  { id: 'p14', lang: 'it', body: 'Estrai i KPI principali dal seguente report aziendale e ordinali per impatto.' },
  { id: 'p15', lang: 'it', body: 'Riassumi questa trascrizione di meeting in 5 bullet point con action item.' },
  { id: 'p16', lang: 'it', body: 'Analizza il bilancio fornito e identifica anomalie significative.' },

  // === Analysis & summarization (EN) ===
  { id: 'p17', lang: 'en', body: 'Extract key performance indicators from this quarterly report.' },
  { id: 'p18', lang: 'en', body: 'Summarize this meeting transcript into bullet points with action items.' },
  { id: 'p19', lang: 'en', body: 'Identify financial anomalies in the attached balance sheet.' },

  // === Creative writing (IT) ===
  { id: 'p20', lang: 'it', body: 'Scrivi una headline accattivante per un articolo su intelligenza artificiale.' },
  { id: 'p21', lang: 'it', body: 'Genera 5 varianti di slogan pubblicitario per un nuovo gelato artigianale.' },

  // === Creative writing (EN) ===
  { id: 'p22', lang: 'en', body: 'Write a catchy headline for an AI-related blog post.' },
  { id: 'p23', lang: 'en', body: 'Generate five tagline variants for an artisanal ice cream brand.' },

  // === Technical/structured output ===
  { id: 'p24', lang: 'en', body: 'Output a strict JSON schema following the format provided.' },
  { id: 'p25', lang: 'it', body: 'Restituisci un JSON con i campi specificati e nessun altro testo.' },

  // === Recipes & misc (rumore) ===
  { id: 'p26', lang: 'it', body: 'Ricetta della pasta alla carbonara tradizionale romana per 4 persone.' },
  { id: 'p27', lang: 'en', body: 'Suggest a workout routine for someone with limited time and no equipment.' },
  { id: 'p28', lang: 'it', body: 'Pianifica un viaggio di 5 giorni a Lisbona con budget medio.' },
  { id: 'p29', lang: 'en', body: 'Recommend three sci-fi novels published after 2020.' },
  { id: 'p30', lang: 'it', body: 'Genera una lista della spesa settimanale per dieta mediterranea.' },
];

// Query di test. Ognuna ha un'expected_match ideale (gli id che dovrebbero
// stare nel top-3 in un mondo perfetto). Mix linguistico voluto per stressare
// i modelli sulle capacità multilingue.
export const queries = [
  {
    q: 'rendi questa email più professionale',
    lang: 'it',
    note: 'IT pure, dovrebbe trovare p01/p02/p05',
    expected: ['p01', 'p02', 'p05', 'p06'],
  },
  {
    q: 'make this message sound more business-like',
    lang: 'en',
    note: 'EN pure, dovrebbe trovare p05/p06/p02',
    expected: ['p05', 'p06', 'p02', 'p01'],
  },
  {
    q: 'follow-up dopo riunione di vendita',
    lang: 'mixed',
    note: 'IT con anglicismo, p03/p07 attesi',
    expected: ['p03', 'p07'],
  },
  {
    q: 'convert Python to TypeScript',
    lang: 'en',
    note: 'tecnico EN, p08 deve essere primo',
    expected: ['p08', 'p10', 'p13'],
  },
  {
    q: 'tradurre codice Python in altro linguaggio',
    lang: 'it',
    note: 'tecnico IT — challenge crosslingue, p08 atteso',
    expected: ['p08', 'p10', 'p12'],
  },
  {
    q: 'estrai KPI da un report finanziario',
    lang: 'it',
    note: 'IT analitico, p14/p17 attesi',
    expected: ['p14', 'p17', 'p16', 'p19'],
  },
  {
    q: 'summarize meeting notes with actions',
    lang: 'en',
    note: 'EN analitico, p18/p15 attesi',
    expected: ['p18', 'p15'],
  },
  {
    q: 'titolo accattivante per articolo tech',
    lang: 'it',
    note: 'IT creativo, p20/p22 attesi',
    expected: ['p20', 'p22'],
  },
  {
    q: 'output JSON only',
    lang: 'en',
    note: 'EN tecnico, p24/p25 attesi',
    expected: ['p24', 'p25'],
  },
  {
    q: 'come si fa la carbonara',
    lang: 'it',
    note: 'rumore IT, deve trovare p26 e nessun email/dev',
    expected: ['p26'],
  },
];
