# Black Vienna Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Jouer à Black Vienna (3–6 joueurs) sur ludo-rs, contre une IA dont la force de déduction suit le niveau, avec une feuille d'enquête à l'écran.

**Architecture:** Nouvelle crate `games/black-vienna` (id `black-vienna`) qui implémente `Game`, `Determinize`, `Heuristics` comme les autres jeux. Un module `knowledge` porte les déductions par niveau et un solveur à retour arrière, utilisés par l'IA et par `determinize`. `ludo-core` gagne un crochet `Heuristics::policy_action` qui reçoit le niveau. Plateau React dans `web/src/games/black-vienna/`.

**Tech Stack:** Rust (ludo-core, serde, rand 0.8, rand_chacha), React 19 + TypeScript, vitest.

**Spec:** `docs/superpowers/specs/2026-10-20-black-vienna-design.md` (sources [R] [C] [F] [U] et ❓1–7 : les citer dans `RULES.md` et dans les commentaires, comme les autres jeux).

## Global Constraints

- Règles du dépôt (`CLAUDE.md`) : le moteur possède toute la logique ; `apply` renvoie `Illegal` sans toucher l'état et ne panique jamais ; une vue ne contient ni la graine ni une donnée cachée ; mélanges du moteur avec `ChaCha8Rng`, jamais `SmallRng` ; ne jamais tirer un `usize` (tirer un `u32`).
- 27 personnages A…Z puis Ö ; 36 cartes Enquête exactement : ACL AGM AOS APQ BCY BHV BLM BQT CFI CSX DHR DJZ DLS DVY EGW ENQ EÖR EUV FÖY FRX FSZ GKO GPX HNÖ HUZ IÖW IPR ITZ JMO JQX JTY KMU KNT KPW LNV OUW ; 3 piles de 12 ; 40 jetons.
- Mains : 8 à 3 joueurs, 6 à 4, 4 à 6 ; à 5, le donneur 4 et les autres 5 ; distribution à partir de la gauche du donneur.
- Points : accusation juste 3 × réserve ; trio juste après coup 1 × réserve ; dernier joueur 1 × réserve.
- Accusation seulement à son tour [U]. Notes automatiques = faits directs seulement [U].
- Seuils d'accusation de l'IA : trios possibles ≤ 3 (Facile), ≤ 2 (Moyen), = 1 (Difficile) [U].
- Textes du site en français ; aucune statistique affichée ; cartes avec la lettre seule [U].
- Commits avec la configuration du dépôt (e-mail perso), message en français, terminé par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Avant chaque commit : `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings` sans erreur.

## Review Focus

1. **Notes en local à plusieurs humains sur un écran** : les notes manuelles d'un siège ne doivent jamais s'afficher pour un autre siège → clé par siège (Task 8).
2. **Accusateur fautif** : il voit le trio, et `determinize` le garde fixe pour lui seul (Task 5).
3. **Réserve presque vide** : enquête spéciale qui demande 2 réponses avec 4 jetons restants → on pose ce qui reste, la partie s'arrête sans vainqueur, sauf pendant la question finale (Task 3).
4. **Trios saisis dans le désordre ou en double** : `[C, A, B]` vaut `[A, B, C]` ; trois lettres distinctes obligatoires (Tasks 4 et 8).
5. **Difficile en début de partie** : la déduction complète doit répondre vite, même avec presque tous les trios encore possibles (Task 6).

---

### Task 1: Crochet de niveau dans ludo-core

**Files:**
- Modify: `crates/ludo-core/src/ai.rs` (`Search`, `Search::for_game`, `choose`, littéraux `Search { … }` des tests)
- Modify: `crates/ludo-core/src/game.rs` (`Heuristics`)

**Interfaces:**
- Produces: `Search { budget, blunder, seed, level: Level }` ; `Heuristics::policy_action<R: Rng + ?Sized>(s: &Self::State, seat: Seat, level: Level, rng: &mut R) -> Option<Self::Action>` (défaut `None`). `choose` l'appelle juste après le cas « une seule action légale », **avant** `scripted_action` et la gaffe aléatoire ; si `Some`, c'est le coup joué.

- [ ] **Step 1: Test qui échoue** dans `ai.rs` : `policy_action_is_played_at_its_level` — un jeu de test (copie de `SteeredNim` ou de `Nim`) dont `policy_action` renvoie `Some(1)` à `Level::Easy`, `None` sinon ; `choose` avec `Search { level: Level::Easy, blunder: 1.0, .. }` renvoie 1 ; avec `Level::Hard`, le comportement actuel.
- [ ] **Step 2:** `cargo test -p ludo-core` → échec de compilation (champ `level` absent).
- [ ] **Step 3:** ajouter le champ, le crochet (doc : « depends only on what `seat` can see »), l'appel dans `choose`, `level` dans `for_game` et dans les littéraux de test.
- [ ] **Step 4:** `cargo test --workspace --release` → tout passe (aucun autre jeu ne change).
- [ ] **Step 5:** commit `ludo-core : Heuristics::policy_action, une politique selon le niveau`.

### Task 2: Matériel et donne

**Files:**
- Create: `games/black-vienna/Cargo.toml` (comme `games/volltreffer`, sans `cards`), `games/black-vienna/src/lib.rs`, `games/black-vienna/src/material.rs`, `games/black-vienna/tests/rules.rs`
- Modify: `Cargo.toml` (membre + `black-vienna = { path = "games/black-vienna" }`)

**Interfaces:**
- Produces (dans `material.rs`, réexportés) :
  - `pub struct Suspect(u8)` (0 = A … 25 = Z, 26 = Ö), `Suspect::new(i: u8) -> Option<Suspect>`, `letter(self) -> char`, `index(self) -> usize`, `all()`. Sérialisé comme une chaîne d'une lettre (`"A"`, `"Ö"`), `try_from` refuse le reste.
  - `pub struct CardId(u8)` (0..36) ; `pub fn suspects(card: CardId) -> [Suspect; 3]` ; `pub const INVESTIGATIONS: [&str; 36]` (la liste des Global Constraints, dans cet ordre).
  - `pub fn hand_sizes(players: u8, dealer: Seat) -> Vec<usize>`.
  - `pub struct Deal { pub trio: [Suspect; 3] /* trié */, pub hands: Vec<Vec<Suspect>>, pub piles: [Vec<CardId>; 3] /* dessus = dernier */ }` ; `pub fn deal(players: u8, dealer: Seat, seed: u64) -> Deal` (ChaCha8Rng ; personnages : 3 premiers = trio, puis un par un à partir de `dealer + 1`).

- [ ] **Step 1: Tests** dans `tests/rules.rs` :
  - `every_suspect_is_on_four_investigations` : chaque `Suspect` apparaît 4 fois, 36 cartes, 3 lettres distinctes par carte.
  - `hands_by_table_size` : `hand_sizes(3,_) == [8,8,8]`, `(4,_) == [6;4]`, `(6,_) == [4;6]`, `(5, Seat(2)) == [5,5,4,5,5]`.
  - `the_deal_splits_everything` : pour 3..=6, 3 + somme des mains = 27, sans doublon ; 3 piles de 12, 36 cartes distinctes.
  - `suspects_serialize_as_letters` : `serde_json::to_string(&Suspect::new(26))` = `"\"Ö\""` ; `"0"` et `"ß"` refusés.
  - `the_deal_of_a_seed_is_pinned` : trio et main du siège 0 pour `deal(4, Seat(0), 7)` (valeurs relevées au premier passage, puis figées).
- [ ] **Step 2:** `cargo test -p black-vienna` → échec.
- [ ] **Step 3:** implémenter `material.rs`.
- [ ] **Step 4:** tests verts.
- [ ] **Step 5:** commit `black vienna : personnages, cartes Enquête et donne`.

### Task 3: Règles — tours, enquêtes, réserve, fin sans vainqueur

**Files:**
- Modify: `games/black-vienna/src/lib.rs`
- Create: `games/black-vienna/src/testing.rs` (tables écrites à la main), `games/black-vienna/RULES.md`
- Test: `games/black-vienna/tests/rules.rs`

**Interfaces:**
- Consumes: Task 2.
- Produces :
  - `pub struct BlackVienna;` `Config {}` ; meta `id: "black-vienna"`, `name: "Black Vienna"`, 3–6, `hidden_info: true`, `clock: false`.
  - `pub const TOKENS: u8 = 40;` `pub const SPECIAL_AFTER: usize = 6;`
  - `pub enum Action { Ask { pile: Option<u8>, target: Seat, extra: Option<CardId> }, Accuse { trio: [Suspect; 3] }, Guess { trio: [Suspect; 3] } }` (trios triés).
  - `pub enum Phase { Turn, Guess, Over }`.
  - `pub struct Answer { pub by: Seat, pub target: Seat, pub card: CardId, pub count: u8, pub placed: u8, pub special: bool }` (`count` = vrai nombre, `placed` = jetons pris, ❓6).
  - `State` avec au moins : `players, dealer, seed, hands, trio, piles, answers: Vec<Answer>, reserve: u8, asked: Vec<bool>, wrong: Vec<bool>, phase, turn, last: Option<Seat>` (dernier joueur, ❓4), `final_asked: bool`, `guesses: Vec<Option<[Suspect; 3]>>`, `accuser: Option<Seat>`, `scores: Vec<u16>`, `events: Vec<Event>`.
  - `pub fn reusable(s: &State) -> Vec<CardId>` : cartes jouées dont la **dernière** réponse vaut 0 ; `pub fn special_allowed(s: &State) -> bool` (chacun interrogé au moins une fois et ≥ 6 cartes jouées).
  - `testing::table(hands: &[&str], trio: &str, piles: [&str; 3]) -> State` : mains et trio en lettres, piles en suites de cartes (`"BLM ACL"`, dessus en dernier), siège 0 au trait, réserve 40.

- [ ] **Step 1: Tests de règles** (dans `tests/rules.rs`) :
  - `the_dealer_starts_and_the_answerer_plays_next` : `Ask` du siège 0 vers 2 → `turn == Seat(2)`.
  - `an_answer_counts_the_suspects_in_hand` : main de 2 = `B…` sans L ni M, carte BLM → `count == 1`, réserve 39.
  - `one_cannot_ask_oneself` : `Ask { target: Seat(0) }` refusé, état inchangé.
  - `special_needs_everyone_asked_and_six_cards` : `extra` absent des actions légales avant ; présent ensuite pour une carte à 0 jeton, jamais pour une carte à ≥ 1 ; l'ancienne carte reçoit une nouvelle `Answer` devant le nouvel interrogé (❓2) et n'est plus réutilisable si elle prend un jeton.
  - `a_short_reserve_gives_what_is_left_and_ends_the_game` : réserve 5, enquête spéciale à 3 + 3 → `placed` total 5, `Phase::Over`, aucun point (❓5, ❓6).
  - `three_tokens_left_ends_without_winner` : réserve 4, réponse à 1 → fin, scores nuls.
  - `empty_piles_leave_only_specials_or_the_end` (❓7) : piles vides, pas de carte réutilisable, joueur ayant déjà accusé à tort → `Phase::Over` sans vainqueur.
- [ ] **Step 2:** échec.
- [ ] **Step 3:** implémenter `new`, `pending`, `legal_actions` (pour `Ask` : chaque pile non vide × chaque autre siège × `None` ou une carte de `reusable` si `special_allowed`, et `pile: None` seulement avec `extra: Some`), `apply` (refus sans modification), `outcome` (classement aux points, égalités conservées). Écrire `RULES.md` (sections Mise en place, Tour, Enquête spéciale, Fin sans vainqueur, ❓1–7).
- [ ] **Step 4:** tests verts.
- [ ] **Step 5:** commit `black vienna : tours, enquêtes, réserve`.

### Task 4: Règles — accusations, trios après coup, dernier joueur

**Files:** `games/black-vienna/src/lib.rs`, `RULES.md`, `tests/rules.rs`

**Interfaces:**
- Produces : `Event` (`#[serde(tag = "type")]`) avec au moins `Asked { by, target, cards: Vec<(CardId, u8)> }`, `Accused { seat, right: bool }`, `Revealed { trio, guesses: Vec<Option<[Suspect; 3]>>, scores }` ; règles d'accusation et de fin.

- [ ] **Step 1: Tests** :
  - `accusing_is_offered_only_at_the_start_of_ones_turn` : `Accuse` légal au début du tour, plus après son `Ask` (le tour est passé), jamais pour un siège qui a accusé à tort.
  - `a_trio_is_unordered_and_distinct` : `Accuse { trio: [C, A, B] }` refusé (non trié), `[A, A, B]` refusé ; 2925 accusations légales au début.
  - `a_wrong_accuser_keeps_playing_without_accusing` : après une fausse accusation, il doit encore interroger, répond aux enquêtes, ne marque rien ; les autres ne voient que `right: false`.
  - `a_right_accusation_asks_the_others_for_a_trio` : `Phase::Guess`, `pending` = les autres sièges qui n'ont pas accusé ; les trios des autres ne sont pas dans la vue d'un siège tant que tous n'ont pas répondu ; scores = 3 × réserve pour l'accusateur, 1 × réserve pour chaque trio juste, 0 sinon.
  - `the_last_player_may_ask_once_then_must_accuse` (❓4) : à 3 joueurs, 1 et 2 accusent à tort → `last == Seat(0)` ; il peut `Ask` une fois (sans fin à 3 jetons, ❓5) ou accuser tout de suite ; après sa question, seules des `Accuse` sont légales ; juste → 1 × réserve.
- [ ] **Step 2:** échec. **Step 3:** implémenter ; compléter `RULES.md` (Accusation, Fin). **Step 4:** vert.
- [ ] **Step 5:** commit `black vienna : accusations et fin de partie`.

### Task 5: Vue, notes automatiques et déterminisation

**Files:**
- Create: `games/black-vienna/src/view.rs`, `games/black-vienna/src/knowledge.rs` (première partie), `games/black-vienna/src/determinize.rs`, `games/black-vienna/tests/conformance.rs`
- Modify: `crates/ludo-registry/src/lib.rs`, `crates/ludo-registry/Cargo.toml`

**Interfaces:**
- Produces :
  - `pub enum Column { Seat(Seat), Trio }` ; `pub enum Mark { Plus, Minus }` ; `pub struct Note { pub suspect: Suspect, pub column: Column, pub mark: Mark }`.
  - `pub fn direct_facts(s: &State, seat: Seat) -> Vec<Note>` (dans `knowledge.rs`) : ses cartes (+ chez lui, − ailleurs et dans `Trio`) ; réponse à 0 → − chez l'interrogé ; réponse à 3 → + chez l'interrogé ; si `seat` a accusé à tort ou en `Over`, le trio (+ dans `Trio`). Rien d'autre.
  - `View` (camelCase) : `phase, turn, players, dealer, hand, handSizes, reserve, piles: [PileView; 3]` (`{ top: Option<[Suspect; 3]>, topId: Option<CardId>, left }`), `answers` (avec les `suspects` de chaque carte), `wrong: Vec<bool>`, `canAccuse`, `specialAllowed`, `reusable`, `last`, `myGuess`, `trio: Option<[Suspect; 3]>` (accusateur fautif ou `Over`), `guesses` (seulement en `Over`), `scores`, `notes: Vec<Note>` (= `direct_facts`), `events`.
  - `pub fn sample_world<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> (Vec<Vec<Suspect>>, [Suspect; 3])` (dans `knowledge.rs`) : mains des autres et trio tirés au hasard parmi ceux compatibles avec la main de l'observateur, les tailles de main, toutes les réponses (`count`) et le trio s'il le connaît ; retour arrière avec ordre aléatoire.

- [ ] **Step 1: Tests** :
  - `knowledge.rs` : `direct_facts_are_only_direct` — main `ABC`, réponses BLM→0 chez 1 et DHR→3 chez 2, une réponse 1 sur ACL chez 1 → exactement les notes attendues, aucune pour l'ACL à 1.
  - `determinize.rs` : `a_sampled_world_fits_every_answer` — sur 50 tirages, recompter chaque réponse donne `count` ; la main de l'observateur et les tailles sont inchangées ; `the_wrong_accuser_keeps_the_trio` (pour lui seul) ; les autres trios varient.
  - `tests/conformance.rs` : `check::<BlackVienna>(&Config::default(), players, 30, …)` pour 3..=6.
- [ ] **Step 2:** échec. **Step 3:** implémenter `view.rs`, `direct_facts`, `sample_world`, `Determinize` (oublie aussi les `guesses` des autres en phase `Guess`, comme Deadfall oublie les portes) ; inscrire le jeu au registre (`static BLACK_VIENNA` et l'entrée dans `GAMES`). **Step 4:** vert, conformance comprise.
- [ ] **Step 5:** commit `black vienna : vue, notes automatiques, déterminisation, conformance et registre`.

### Task 6: Déductions par niveau

**Files:** `games/black-vienna/src/knowledge.rs` (suite), tests dans ce fichier.

**Interfaces:**
- Produces :
  - `pub struct Knowledge { cells: Vec<[Option<bool>; 27]> }` : une rangée par colonne (sièges puis `Trio`), une case par `Suspect` ; `pub fn knowledge(s: &State, seat: Seat, level: Level) -> Knowledge` ; `Knowledge::get(column, suspect) -> Option<bool>`.
  - `pub fn possible_trios(s: &State, seat: Seat, level: Level) -> Vec<[Suspect; 3]>`.
  - Facile : `direct_facts` seulement ; trios possibles = triplets sans personnage marqué « + » ailleurs (ni chez soi).
  - Moyen : les règles du §6 de la spec appliquées jusqu'au point fixe (un + exclut les autres colonnes ; une réponse dont le compte égale les inconnus restants les met en + ; égale aux + connus, met le reste en − ; un siège dont tous les + sont connus a − partout ailleurs, dont tous les − sont connus a + sur le reste ; un personnage − partout est dans le trio, et le trio a 3 cartes exactement) ; trios possibles = triplets compatibles avec ces cases.
  - Difficile : trios pour lesquels `sample_world`-style recherche trouve au moins un monde compatible (même solveur, déterministe, avec élagage par les cases de Moyen d'abord).

- [ ] **Step 1: Tests** :
  - `medium_finds_the_last_suspect_of_an_answer` : 1 jeton sur BLM, L et M en − chez lui → B en + chez lui (Facile ne le trouve pas).
  - `medium_closes_a_full_hand` : siège à 4 cartes dont 4 + connus → − partout ailleurs chez lui.
  - `hard_crosses_answers` : situation où seul le croisement de deux réponses à 1 et 2 jetons fixe le trio → `possible_trios(Hard).len() == 1`, `possible_trios(Medium).len() > 1`.
  - `levels_never_exclude_the_true_trio` : sur 30 parties aléatoires jouées à mi-chemin, le vrai trio est dans `possible_trios` à chaque niveau.
  - `hard_is_fast_at_the_start` : 6 joueurs, après 2 enquêtes, `possible_trios(Hard)` en moins de 500 ms en `--release` (mesure `Instant`).
- [ ] **Step 2–4:** échec, implémentation, vert.
- [ ] **Step 5:** commit `black vienna : déductions selon le niveau`.

### Task 7: IA

**Files:** `games/black-vienna/src/heuristics.rs`, `CLAUDE.md` (ligne d'arène), `README.md`

**Interfaces:**
- Consumes: Tasks 1, 5, 6.
- Produces : `impl Heuristics for BlackVienna` avec `policy_action` (toujours `Some`), `budget` → `None`, `rollout_action` = `policy_action` au niveau Moyen (pour les cas où le niveau manque), `random_actions` sans `Accuse`/`Guess` (une gaffe n'accuse jamais au hasard).
  - Accuser à son tour quand `possible_trios(level).len() <= seuil` (3 / 2 / 1) en nommant le plus plausible : celui dont les personnages ont le plus de colonnes exclues, égalités au hasard. Même choix pour le dernier joueur forcé et pour `Guess`.
  - Question : Facile au hasard parmi les `Ask` légaux ; Moyen et Difficile, l'`Ask` qui maximise le nombre de cases inconnues (chez l'interrogé, pour les personnages de la carte et de l'éventuelle carte spéciale), égalités au hasard ; le dernier joueur pose sa question finale si une case inconnue peut changer ses trios possibles.

- [ ] **Step 1: Tests** : `accuses_at_its_threshold` (Facile accuse à 3 trios possibles, Difficile pas avant 1) ; `never_blunders_an_accusation` (`random_actions` sans accusation) ; `policy_is_always_legal` (parties complètes à chaque niveau, 3..=6 joueurs).
- [ ] **Step 2–4:** échec, implémentation, vert.
- [ ] **Step 5: Arène** : `cargo run --release -p ludo-cli -- arena black-vienna 200 hard easy 3`, puis `… 5`, puis `arena black-vienna 200 medium easy 3`. Attendu : Difficile ≥ 0.55 contre Facile ; Moyen > 0.5 contre Facile. Noter les mesures dans `CLAUDE.md` (« Commandes ») et dans la spec (§6).
- [ ] **Step 6:** commit `black vienna : IA selon le niveau, mesures de l'arène`.

### Task 8: Plateau web et feuille d'enquête

**Files:**
- Create: `web/src/games/black-vienna/types.ts`, `Board.tsx`, `Sheet.tsx`, `rules.ts`, `black-vienna.css`, `__tests__/black-vienna.test.tsx`
- Modify: `web/src/games/registry.ts` (`{ id: "black-vienna", family: "modern", Board: BlackViennaBoard, rules: BLACK_VIENNA_RULES }`), `README.md`

**Interfaces:**
- Consumes: la `View` et les `Action` de Tasks 3–5 (types miroirs ; `Suspect` = chaîne d'une lettre).
- Produces :
  - `Sheet({ view, me, names, marks, onCycle })` : 27 lignes A…Ö, une colonne par siège, ligne Black Vienna ; les `view.notes` s'affichent comme automatiques (non cliquables) ; les autres cases montrent `marks` et appellent `onCycle(suspect, column)` (vide → + → − → ? → vide).
  - `useSheetMarks(gameKey: string, seat: number)` : état local, mémorisé en `sessionStorage` sous une clé qui contient le siège (try/catch), vidé en fin de partie.
  - `pickTrio(letters: string[]): [string, string, string] | null` : trois lettres distinctes triées dans l'ordre A…Ö, sinon `null`.

- [ ] **Step 1: Tests vitest** :
  - chaque contrôle de question soumet une action de `legalActions` et aucune autre ; l'enquête spéciale n'est proposée que si `specialAllowed` ;
  - `pickTrio(["C","A","B"])` → `["A","B","C"]`, `pickTrio(["A","A","B"])` → `null`, `pickTrio(["Ö","A","Z"])` → `["A","Z","Ö"]` ;
  - une case de `view.notes` n'a pas de `onClick` ; une autre passe par + − ? vide ;
  - les marques du siège 0 ne s'affichent pas pour le siège 1 (deux rendus avec des `marks` de clés différentes) ;
  - rien de cliquable hors de son tour, en spectateur, en `Over` ; le trio et les points s'affichent en `Over` ; aucun `%` dans le texte.
- [ ] **Step 2:** `cd web && pnpm exec vitest run src/games/black-vienna` → échec.
- [ ] **Step 3:** implémenter (cartes = carrés avec la lettre, style des cartes Pairs ; historique des enquêtes sous la feuille ; bouton « Black Vienna ! » qui ouvre le choix de trois lettres) ; `rules.ts` en 5 lignes au plus, d'accord avec `RULES.md`.
- [ ] **Step 4:** `pnpm exec vitest run` et `pnpm exec tsc --noEmit` sans erreur ; `./scripts/build-wasm.sh` passe.
- [ ] **Step 5:** commit `web : Black Vienna, feuille d'enquête à notes automatiques`.
