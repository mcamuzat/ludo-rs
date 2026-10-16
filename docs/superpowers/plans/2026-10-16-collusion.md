# Collusion — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal :** ajouter Collusion (`cards-collusion`, jeu de plis à 4 de Parlett, avec phrases toutes faites) à la famille « Cartes traditionnelles ».

**Architecture :** un module `games/cards/src/collusion/` calqué sur `president/`. Deux accroches génériques dans `Heuristics` (`scripted_action`, `random_actions`) laissent l'IA parler sans que la recherche voie les phrases. `deck.rs` gagne une redistribution sous contraintes pour les coupes.

**Tech Stack :** Rust (ludo-core, games/cards, rand_chacha), WASM, React/TypeScript (vitest).

**Spec :** `docs/superpowers/specs/2026-10-16-collusion-design.md`. Les tâches s'y réfèrent par paragraphe (§3, §4…).

## Global Constraints

- Les règles du `CLAUDE.md` du dépôt s'appliquent : `apply` renvoie `Illegal` sans modifier l'état ; `view` ne contient ni la graine ni la main d'un autre ; mélanges en `ChaCha8Rng` ; tirages en `u32` convertis, jamais en `usize`.
- 4 joueurs exactement ; cible `Number` de 50 à 100, défaut 100.
- Bonus : 10 à chacun d'une paire exacte, 20 au plus petit si tous diffèrent, 30 au quatrième si trois sont égaux.
- Sans bonus, si `total + plis >= cible`, on retranche les plis.
- **Aucune statistique** dans la vue ni sur le plateau.
- Commits avec l'e-mail du dépôt (`marc.camuzat@gmail.com`, déjà en config locale), terminés par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Avant chaque commit : `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings`.

## Review Focus

1. Le joueur parle, puis veut jouer sa carte : il est toujours en attente, et `Say` n'est plus légal. Test dans la tâche 3 (`say_once_then_play`).
2. Une seule carte jouable : `legal_actions` ne contient que cette carte (pas de `Say`), et le site la joue seul. Test dans la tâche 3 (`no_talk_with_a_single_playable_card`).
3. La déterminisation ne donne jamais à un joueur une couleur à laquelle il a déjà coupé ; la conformance le vérifie indirectement, un test direct le pin. Test dans la tâche 4 (`determinize_respects_voids`).
4. Un joueur qui atteint la cible avec un bonus pendant qu'un autre retranche : la partie s'arrête, le classement suit les totaux. Test dans la tâche 3 (`game_ends_on_target_reached_with_a_bonus`).
5. Les phrases de la donne précédente ne restent pas affichées. Test dans la tâche 3 (`phrases_clear_on_new_deal`) et dans la tâche 5 (bulles vides après la donne).

---

### Task 1 : accroches d'IA dans `ludo-core`

**Files :**
- Modify : `crates/ludo-core/src/game.rs` (trait `Heuristics`)
- Modify : `crates/ludo-core/src/ai.rs` (`choose`, tests)

**Interfaces :**
- Produces :
  - `fn scripted_action(s: &Self::State, seat: Seat) -> Option<Self::Action>`, par défaut `None`. C'est une action que l'IA joue sans chercher ; elle doit être légale.
  - `fn random_actions(s: &Self::State, seat: Seat) -> Vec<Self::Action>`, par défaut `Self::legal_actions(s, seat)`. Ce sont les coups parmi lesquels Facile tire son coup au hasard ; ensemble non vide, inclus dans `legal_actions`.

- [ ] **Step 1 : tests en échec** dans `ai.rs::tests`, avec un jeu de test (dans `testgames.rs` ou local au module) dont `scripted_action` renvoie `Some(x)` dans un état donné et dont `random_actions` ne renvoie qu'un coup :
  - `scripted_action_is_played_without_search` : `choose` renvoie `x`, même avec `blunder = 1.0` ;
  - `blunder_draws_from_random_actions` : avec `blunder = 1.0` et pas d'action scriptée, `choose` renvoie toujours le seul coup de `random_actions`, sur 50 graines.
- [ ] **Step 2 :** `cargo test -p ludo-core --release ai::` → échec (méthodes absentes).
- [ ] **Step 3 :** ajouter les deux méthodes au trait. Dans `choose`, après le cas `legal.len() == 1` : `if let Some(a) = G::scripted_action(state, seat) { return Ok(a); }`, puis le tirage `blunder` se fait dans `G::random_actions(state, seat)` au lieu de `legal`.
- [ ] **Step 4 :** `cargo test --workspace --release` → tout passe ; les autres jeux sont inchangés (défauts).
- [ ] **Step 5 :** commit `core: accroches d'IA, action scriptée et coups du hasard`.

### Task 2 : redistribution sous contraintes dans `deck.rs`

**Files :**
- Modify : `games/cards/src/deck.rs` (fonction et tests)

**Interfaces :**
- Produces : `pub fn redeal_where<R: Rng + ?Sized>(hands: &mut [&mut Vec<Card>], accepts: &dyn Fn(usize, Card) -> bool, rng: &mut R)`. Elle redistribue les cartes des mains en gardant les tailles, chaque carte allant à une main `i` telle que `accepts(i, carte)`. La donne d'entrée doit satisfaire `accepts` ; elle sert de repli.

- [ ] **Step 1 : tests en échec** :
  - `redeal_where_keeps_sizes_and_cards` : même ensemble de cartes, mêmes tailles ;
  - `redeal_where_respects_the_constraint` : trois mains, la main 0 refuse les cœurs ; sur 200 graines, la main 0 n'a jamais de cœur ;
  - `redeal_where_still_shuffles` : sans contrainte bloquante, au moins une donne sur 20 diffère de l'entrée.
- [ ] **Step 2 :** `cargo test -p cards --release deck::` → échec.
- [ ] **Step 3 :** implémentation. Mélanger le pool, placer d'abord les cartes acceptées par le moins de mains, chacune dans une main tirée au hasard parmi celles qui l'acceptent et ont encore de la place (tirage `u32` converti). En cas d'impasse, recommencer, jusqu'à 20 essais ; au-delà, garder la donne d'entrée.
- [ ] **Step 4 :** tests verts.
- [ ] **Step 5 :** commit `cards: redistribution sous contraintes`.

### Task 3 : règles de Collusion

**Files :**
- Create : `games/cards/src/collusion/mod.rs`, `games/cards/src/collusion/testing.rs`
- Create : `games/cards/tests/collusion_rules.rs`
- Modify : `games/cards/src/lib.rs` (`pub mod collusion; pub use collusion::Collusion;`)
- Modify : `games/cards/RULES.md` (section Collusion, sourcée [C], avec ❓ donneur et ❓ fin de partie, d'après spec §3)

**Interfaces :**
- Produces (dans `collusion`) :
  - `pub struct Collusion;` ; `pub const DEFAULT_TARGET: u32 = 100;`, `MIN_TARGET = 50`, `MAX_TARGET = 100` ;
  - `Config { target: u32 }` (`#[serde(default)]`), avec `Game::options()` = `Number { min: 50, max: 100, default: 100 }` sur la clé `target` et `check_config` qui refuse hors bornes ;
  - `#[serde(tag = "type")] enum Phrase { Aim { n: u8 }, Unsure, WhoAims { n: u8 } }` ;
  - `enum Action { Play { card: Card }, Say { phrase: Phrase } }` ;
  - `pub fn rank_value(rank: Rank) -> u8` : 2 = le plus faible, As = le plus fort ;
  - `pub fn bonuses(tricks: [u8; 4]) -> [u32; 4]` ;
  - `#[derive(Serialize)] pub struct DealLine { pub tricks: u8, pub bonus: u32, pub deducted: bool, pub total: u32 }` et `pub fn score_deal(scores: &mut [u32; 4], tricks: [u8; 4], target: u32) -> [DealLine; 4]` ;
  - `State`, avec au minimum : `config`, `seed`, `deal_no: u32`, `dealer: Seat`, `hands: Vec<Vec<Card>>`, `trick: Vec<(Seat, Card)>`, `last_trick: Option<(Seat, Vec<(Seat, Card)>)>` (gagnant, cartes), `turn: Option<Seat>` (`None` en fin de partie), `taken: [u8; 4]`, `voids: [[bool; 4]; 4]` (siège × `Suit` dans l'ordre `Suit::ALL`), `said: bool` (le joueur au trait a déjà parlé), `talk: Vec<(Seat, Phrase)>` (fil de la donne), `scores: [u32; 4]`, `last_deal: Option<[DealLine; 4]>` ;
  - `pub fn standings(scores: &[u32]) -> Vec<u8>` (repris du Président ou partagé si l'extraction est simple) ;
  - `testing::c`/`cards` (même notation que `president::testing`, à réutiliser plutôt que recopier si possible) et `testing::table(hands: [&str; 4], leader: u8) -> State`.

Le donneur de la donne `k` (à partir de 0) est `Seat(k % 4)`, et `Seat((k + 1) % 4)` entame. Chaque donne vient de son propre flux de la graine, comme `President` (`seed ^ (deal_no + 1) * 0x9E37_79B9_7F4A_7C15`) via `deal_evenly(pack_52(), 4, stream)`.

`legal_actions(seat)` pour le joueur au trait : les cartes jouables (fournir si possible), plus, si `!said` et au moins deux cartes jouables, `Say` pour `Aim { n }` avec `taken[seat] <= n <= taken[seat] + plis restants` (13 moins les plis ramassés), `Unsure` et `WhoAims { n }` pour n de 0 à 13. `Say` pousse dans `talk` et met `said = true` ; `Play` remet `said = false` pour le joueur suivant.

- [ ] **Step 1 : tests en échec** dans `tests/collusion_rules.rs` (un test par règle, valeurs de la spec) :
  - `must_follow_suit` : avec du cœur en main sur une entame cœur, les seules cartes jouables sont les cœurs ;
  - `any_card_when_void` : sans cœur, toutes les cartes sont jouables, et le siège est marqué coupé à cœur ;
  - `highest_of_led_suit_wins_no_trumps` : un As de pique sur une entame cœur ne gagne pas ;
  - `winner_leads_next_trick` ;
  - `dealer_rotates_left` : deuxième donne, donneur `Seat(1)`, entame `Seat(2)` ;
  - `bonuses` : `[5,5,2,1] → [10,10,0,0]` ; `[6,4,2,1] → [0,0,0,20]` ; `[3,3,3,4] → [0,0,0,30]` ; `[0,4,4,5] → [0,10,10,0]` ;
  - `deducts_without_bonus_near_target` : total 95, 6 plis, pas de bonus, cible 100 → 89 ; même cas à 45 avec cible 50 → 39 ;
  - `adds_with_bonus_past_target` : total 95, 5 plis, bonus 10 → 110 ;
  - `game_ends_on_target_reached_with_a_bonus` : après la donne, `pending` vide, `outcome` classe par total, ex æquo partagés ;
  - `say_once_then_play` : après `Say`, le siège est toujours en attente, `Say` n'est plus légal et `apply(Say)` renvoie `Illegal` sans changer l'état ;
  - `aim_bounds` : avec 2 plis pris et 5 plis joués, `Aim` va de 2 à 10 ;
  - `no_talk_with_a_single_playable_card` ;
  - `phrases_clear_on_new_deal` ;
  - `illegal_card_is_refused_unchanged`.
- [ ] **Step 2 :** `cargo test -p cards --release --test collusion_rules` → échec.
- [ ] **Step 3 :** implémenter `mod.rs` (règles, `Game` : `meta` avec `min_players = max_players = 4`, `hidden_info: true`, `clock: true` ; `timeout_action` = la plus faible carte jouable par `rank_value` puis l'ordre `Suit::ALL` ; `outcome` par `standings`). `view` peut renvoyer provisoirement un type minimal ; il est complété à la tâche 4. Écrire la section de `RULES.md`.
- [ ] **Step 4 :** tests verts.
- [ ] **Step 5 :** commit `cards: Collusion, règles et phrases`.

### Task 4 : vue, déterminisation, IA, conformance, registre

**Files :**
- Create : `games/cards/src/collusion/view.rs`, `determinize.rs`, `heuristics.rs`
- Modify : `games/cards/tests/conformance.rs`
- Modify : `crates/ludo-registry/src/lib.rs` (`static CARDS_COLLUSION: GameEntry<cards::Collusion>` et entrée dans `GAMES` après `CARDS_PRESIDENT`)
- Modify : `CLAUDE.md` (ligne d'arène), `README.md` (liste des jeux)

**Interfaces :**
- Consumes : `redeal_where` (tâche 2), `scripted_action`/`random_actions` (tâche 1), `State` et fonctions de la tâche 3.
- Produces : `View` (camelCase côté sérialisé, comme le Président) : `me: Option<Seat>`, `hand: Vec<Card>` (triée par couleur dans l'ordre `Suit::ALL`, puis valeur décroissante), `handCounts: [u8; 4]`, `trick: Vec<(Seat, Card)>`, `lastTrick`, `taken: [u8; 4]`, `scores: [u32; 4]`, `target: u32`, `dealer: Seat`, `dealNo: u32`, `turn: Option<Seat>`, `lastSaid: [Option<Phrase>; 4]`, `talk: Vec<(Seat, Phrase)>`, `lastDeal: Option<[DealLine; 4]>`, `legal: Vec<Action>` (vide si ce n'est pas mon tour). Pas de graine, pas d'autre main, pas de `voids` (déductible, mais ce serait une aide de comptage).

- [ ] **Step 1 : tests en échec** :
  - dans `determinize.rs` : `determinize_respects_voids`, une table où le siège 2 a coupé à cœur ; sur 100 graines, le siège 2 ne reçoit jamais de cœur, la main de l'observateur est intacte, les tailles sont conservées, la graine est changée ;
  - dans `heuristics.rs` : `ai_announces_when_its_estimate_changes` (premier tour : `scripted_action` donne `Say(Aim)` ; après l'avoir dit, `None` tant que l'estimation ne bouge pas) ; `rollout_never_talks` (sur 200 états aléatoires, `rollout_action` est un `Play`) ; `random_actions_are_plays_only` ;
  - dans `tests/conformance.rs` : `collusion_conforms` avec `Config::default()` et `Config { target: 50 }`, 4 joueurs.
- [ ] **Step 2 :** les tests échouent.
- [ ] **Step 3 :** implémenter :
  - `determinize` : retirer de l'observateur, redistribuer les trois autres mains par `redeal_where` avec `accepts = |i, c| !voids[siège i][couleur c]`, puis `d.seed = rng.gen()` ;
  - `Heuristics` : `search_actions` = cartes jouables seulement ; `random_actions` = idem ; `rollout_action` = au choix (spec §4), jamais `Say` ; `evaluate` s'arrête quand `deal_no` dépasse celui de la racine, en rendant `Outcome::Ranked(standings(&scores)).rewards()` ;
  - `scripted_action` : si `Say` est légal et que l'estimation diffère du dernier `Aim` de ce siège dans `talk` (ou qu'il n'en a pas), `Some(Say { Aim { n: estimation } })`. Estimation : plis pris plus le nombre de cartes de sa main qui sont la plus haute carte encore en jeu de leur couleur (cartes non vues dans les plis ramassés ni le pli en cours), bornée à l'intervalle de `Aim`.
- [ ] **Step 4 :** `cargo test --workspace --release` → vert.
- [ ] **Step 5 :** `cargo run --release -p ludo-cli -- arena cards-collusion 200 hard easy 4` → Difficile ≥ 0.55. Noter le score mesuré, l'ajouter dans « Commandes » du `CLAUDE.md`, et ajouter Collusion à la liste des jeux du `README.md`.
- [ ] **Step 6 :** commit `cards: Collusion, vue, déterminisation, IA, conformance et registre`.

### Task 5 : plateau web

**Files :**
- Create : `web/src/games/cards/collusion/types.ts`, `Board.tsx`, `rules.ts`
- Create : `web/src/games/cards/__tests__/collusion.test.tsx`
- Modify : `web/src/games/registry.ts` (entrée `{ id: "cards-collusion", family: "cards", Board: CollusionBoard, rules: COLLUSION_RULES }`)
- Modify : `web/src/games/options.ts` (`"cards-collusion.target": "Points à atteindre"`)

**Interfaces :**
- Consumes : `View` et `Action` de la tâche 4 (types identiques dans `types.ts`), `PlayingCard`/`cardName` de `../PlayingCard`.

Libellés des phrases : `Aim` → « Je vise N pli(s) », `Unsure` → « Je ne sais pas encore », `WhoAims` → « Qui vise N ? ». Le reste suit la spec §5 : quatre sièges, pli au centre, plis/total/badge donneur sous chaque siège, bulle de la dernière phrase, fil replié de la donne, bouton « Parler » (valeurs de N prises dans `legal`), cartes non jouables grisées, récapitulatif de fin de donne depuis `lastDeal`. Le nom affiché vient de `meta().name` = « Collusion ».

- [ ] **Step 1 : tests en échec** dans `collusion.test.tsx` (sur le modèle de `president.test.tsx`) :
  - une carte non légale est désactivée, un clic sur une carte légale envoie `{ Play: { card } }` (forme exacte de la sérialisation serde de `Action`) ;
  - « Parler » n'apparaît que si `legal` contient un `Say`, et propose « Je vise 2 » à « Je vise 10 » quand `legal` les contient ;
  - la bulle d'un siège montre sa dernière phrase ; avec `lastSaid` vide, aucune bulle ;
  - le récapitulatif affiche « retranché » pour une `DealLine` avec `deducted: true`.
- [ ] **Step 2 :** `cd web && pnpm test collusion` → échec.
- [ ] **Step 3 :** implémenter `types.ts`, `Board.tsx`, `rules.ts` (d'accord avec `RULES.md`), l'entrée du registre et le libellé de l'option.
- [ ] **Step 4 :** `./scripts/build-wasm.sh`, puis `cd web && pnpm test && pnpm build` → vert. Vérifier dans le navigateur (`pnpm preview` + playwright-core et le Chromium de `~/.cache/ms-playwright`) une partie contre 3 IA : les bulles s'affichent, on peut parler puis jouer, la donne se termine sur son récapitulatif.
- [ ] **Step 5 :** commit `web: plateau de Collusion`.
