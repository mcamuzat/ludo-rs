# Pairs, plan 3 : Ghost Town — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ghost Town (2–6 joueurs, jeu à jetons où le pot sert de banque) se joue sur ludo-rs contre l'IA et en ligne, en 5 mains.

**Architecture:** Troisième module de `games/pairs`, `ghost_town/`, sous l'id `pairs-ghost-town`, en suivant la procédure « Ajouter un jeu Pairs » de `CLAUDE.md`. Aucun changement de plateforme ni de `deck.rs`.

**Tech Stack:** Rust stable (rand 0.8, rand_chacha 0.3, serde), React 19 + TypeScript, vitest 3.

**Spec:** `docs/superpowers/specs/2026-10-13-pairs-design.md` pour les principes communs (§2, §3, §6, §7) ; les règles et décisions propres à Ghost Town sont ci-dessous et dans `games/pairs/RULES.md`. Source : *Pairs Companion Book*, chapitre « Ghost Town ».

## Décisions validées par l'utilisateur

- Une partie = 5 mains (`Config { hands: Option<u8> }`, plafond 15). Chacun part de 0 jeton et peut passer en négatif ; classement final aux jetons, égalités conservées.
- Pot insuffisant pour payer une pioche : le joueur prend ce qui reste, la main s'arrête.
- Égalité pour commencer : départage par des cartes de la pioche, ensuite défaussées (livret).
- Pioche vide : la défausse est remélangée.
- Aucune statistique dans la vue ni sur le plateau.

## Global Constraints

- Branche `ghost-town`. `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release` avant chaque commit.
- `apply` rend `Illegal` sans modifier l'état, ne panique jamais. Une vue ne contient ni graine, ni pioche, ni main d'un autre, ni carte choisie par un autre pendant la mise en place.
- Mélanges en `ChaCha8Rng` ; jamais de tirage `usize`.
- Mise 10, jouer une carte coûte 2, main de 5 cartes, 2 à 6 joueurs, `hidden_info: true`, `clock: false`.
- La somme des jetons des joueurs et du pot vaut toujours 0.

## Review Focus

1. **Pot plus petit que la carte piochée** : on ne gagne que ce qui reste, la main s'arrête, la somme reste nulle. → `a_draw_takes_no_more_than_the_pot_holds`.
2. **Jouer chez le voisin quand le voisin de gauche s'est couché** : la cible est le prochain joueur encore en jeu, pas un siège vide. → `the_neighbour_is_the_next_player_still_in`.
3. **Doublons posés dans la rue à la mise en place** : un seul exemplaire reste, les autres sont défaussés, les 55 cartes sont conservées. → `duplicates_in_the_street_are_discarded` et `chips_and_cards_are_conserved`.
4. **Cible ou carte absurde** (`to` hors table, carte absente de la main, coup hors tour) : `Illegal`. → `absurd_plays_are_refused`.
5. **Plateau une fois couché ou éliminé, et en fin de partie** : rien à cliquer. → tests web.

## Tasks

### Task 1: Règles, vue, RULES.md
**Files:** Create `games/pairs/src/ghost_town/{mod.rs,view.rs,testing.rs}`, `games/pairs/tests/ghost_town_rules.rs` ; Modify `games/pairs/src/lib.rs`, `games/pairs/RULES.md`.
**Produces:** `pairs::GhostTown` (`Game`), `ghost_town::{Config, Action { Place{card}, Play{card,to}, Draw, Fold }, Phase, Event, State, View, SeatView, ANTE, PLAY_COST, HAND}`, `State::{active, left_of, unlocated_for, draw_value}`, `testing::{table, setup}`.
- [ ] Écrire `tests/ghost_town_rules.rs` et `testing.rs` ; les voir échouer (module absent).
- [ ] Écrire `mod.rs`, `view.rs`, la section de `RULES.md` ; tous les tests passent.
- [ ] Commit `pairs: Ghost Town rules, view and RULES.md`.

### Task 2: Déterminisation et conformance
**Files:** Create `games/pairs/src/ghost_town/determinize.rs` ; Modify `games/pairs/tests/conformance.rs`.
- [ ] Tests (vue inchangée, mains des autres redistribuées, choix des autres oubliés en mise en place) et cas de conformance de 2 à 6 joueurs ; implémentation ; commit `pairs: Ghost Town determinization and conformance`.

### Task 3: IA et registre
**Files:** Create `games/pairs/src/ghost_town/heuristics.rs` ; Modify `crates/ludo-registry/src/lib.rs`, `CLAUDE.md`.
- [ ] Tests de la politique (pioche quand l'espérance est bonne, se couche quand elle est très mauvaise et qu'il n'y a rien à jouer, action toujours légale, une seule candidate à la mise en place) ; implémentation ; `ludo arena pairs-ghost-town 400 hard easy` ≥ 0,55 à 2 joueurs, consigné dans `CLAUDE.md` ; commit `pairs: Ghost Town AI and registry entry`.

### Task 4: Plateau web et docs
**Files:** Create `web/src/games/pairs/ghost-town/{types.ts,Board.tsx}`, `web/src/games/pairs/__tests__/ghost-town.test.tsx` ; Modify `web/src/games/registry.ts`, `web/src/games/pairs/pairs.css`, `README.md`, spec (§11).
- [ ] Test du plateau (chaque contrôle envoie exactement une action légale ; rien à cliquer hors de son tour, couché, ou en fin de partie ; aucun pourcentage) ; implémentation ; `pnpm test && pnpm build` ; une partie à 6 sièges dans un navigateur sans affichage ; commit `web: Ghost Town board`.
