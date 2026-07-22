# Prompt à Porter — regole di progetto

- **Release e changelog**: usa la skill `/bump` (`.claude/skills/bump/SKILL.md`).
  Mai aggiungere heading o voci a `CHANGELOG.md` fuori dal processo di bump: una
  versione nel changelog deve sempre corrispondere a un tag. Per preparare le voci
  senza rilasciare: `/bump --solo-voci`.
- **CI**: prima di aprire una PR, mappa i path modificati sui workflow con
  `docs/contribuire/ci-workflows.md` — non attendere CI su PR che non ne attivano.
- **Lingua e commit**: copy, commenti, commit e PR in italiano; conventional commits
  (`feat:`, `fix:`, `chore:`, …) senza righe di attribuzione.
