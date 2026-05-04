// Modelli AI target dichiarabili su un prompt. Lista preset condivisa fra
// Editor (dropdown), Libreria (filtro sidebar), badge detail panel.

export interface ModelloTarget {
  value: string;
  label: string;
  famiglia: "anthropic" | "openai" | "google" | "meta" | "altro";
}

export const MODELLI_TARGET: ModelloTarget[] = [
  { value: "claude-opus", label: "Claude Opus", famiglia: "anthropic" },
  { value: "claude-sonnet", label: "Claude Sonnet", famiglia: "anthropic" },
  { value: "claude-haiku", label: "Claude Haiku", famiglia: "anthropic" },
  { value: "gpt-4", label: "GPT-4", famiglia: "openai" },
  { value: "gpt-4-mini", label: "GPT-4 Mini", famiglia: "openai" },
  { value: "gemini-pro", label: "Gemini Pro", famiglia: "google" },
  { value: "gemini-flash", label: "Gemini Flash", famiglia: "google" },
  { value: "llama-3", label: "Llama 3", famiglia: "meta" },
  { value: "generic", label: "Generico", famiglia: "altro" },
];

export const VALORI_VALIDI = new Set(MODELLI_TARGET.map((m) => m.value));

export function etichettaPerValore(value: string | null | undefined): string {
  if (!value) return "";
  return MODELLI_TARGET.find((m) => m.value === value)?.label ?? value;
}
