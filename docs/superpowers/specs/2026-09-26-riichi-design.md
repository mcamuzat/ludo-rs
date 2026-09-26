# ludo-rs — Mahjong Riichi (spec 2)

Date : 2026-09-26
Statut : en relecture
Précédent : `2026-09-26-plateforme-yavalath-design.md` (plateforme, cycle 1)

## 1. Objectif

Jouer au Mahjong Riichi sur la plateforme ludo-rs : contre 3 IA, ou en ligne
avec des amis (sièges mixtes humains/IA), sans règle fausse, sans que l'IA
triche, et sans qu'un joueur lent bloque la table.

Décisions prises :

- **Règles** : Mahjong Soul, telles qu'implémentées par `riichienv-core`
  (`GameRule::default_mjsoul()`).
- **Format** : hanchan uniquement (tours de l'Est et du Sud), 25 000 points
  de départ.
- **Joueurs** : exactement 4 (`min_players = max_players = 4`) ; pas de
  hot-seat (information cachée).
- **Chrono en ligne**, façon Mahjong Soul.
- **Réutiliser une librairie** plutôt qu'écrire les règles.

### Critères de réussite

- Une hanchan complète se joue contre 3 IA et en ligne, sans erreur moteur.
- La conformance passe (dont : aucune fuite d'information dans les vues).
- `ludo arena riichi 40 hard easy 4` donne un score ≥ 0,60 pour Difficile.
- En ligne, une décision non prise à temps est jouée par l'hôte.

### Hors périmètre

Sanma (3 joueurs), options de règles, formulaire d'options par jeu, tonpuusen,
tutoriel, replays illustrés, réseau de neurones, animations et sons.

## 2. Choix de la librairie

Évaluées : `riichienv-core`, `mahc`, `xiangting`, `agari`, `riichi`,
`riichi-tools-rs`, `riichi_hand`, `shanten-dp`, `riichi-elements`.

Retenue : **`riichienv-core` 0.4.10**
(<https://github.com/smly/RiichiEnv>, Apache-2.0). C'est un moteur complet
(pioche, défausse, chi/pon/kan, riichi, ron/tsumo, furiten, nuls abortifs,
pao, yaku/han/fu/points) et pas un simple calculateur. Vérifié le 2026-09-26 :

- compile pour `wasm32-unknown-unknown` avec `default-features = false`,
  `features = ["wasm"]` et `getrandom 0.4` feature `wasm_js` ;
- 200 hanchans à coups légaux aléatoires jouées sans erreur, 10,8 µs par
  coup (observation comprise), clonage de l'état ≈ 0,7 µs ;
- `GameState::step(&HashMap<u8, Action>)` reçoit les réponses simultanées ;
  seuls les joueurs qui ont un appel possible sont sollicités
  (`active_players`) ;
- `get_observation(pid)` masque les mains adverses et fournit attentes et
  tenpai ; `WinResult` fournit yaku (identifiants, noms anglais/japonais
  dans `yaku.rs`), han, fu, points ;
- `WallState::load_wall` permet des tests sur un mur choisi ;
- maintenu (dernier commit en septembre 2026), 114 tests.

Limites, traitées dans notre crate :

- `GameState` n'est pas sérialisable → on sérialise graine + actions et on
  reconstruit par rejeu (§3.2) ;
- pas de redistribution de l'information cachée → notre `determinize` (§3.4) ;
- version 0.x → dépendance épinglée `=0.4.10`, montée de version volontaire.

Écartée : écrire nos propres règles avec `mahc` (score) et `xiangting`
(shanten) — des semaines de travail et un risque élevé d'erreurs de règles.

Licence : Apache-2.0 compatible MIT ; attribution dans `NOTICE` et le README.
Tuiles : SVG de <https://github.com/FluffyStuff/riichi-mahjong-tiles>,
domaine public (CC0), copiés dans `web/public/tiles/` avec leur licence.

## 3. Le crate `games/riichi`

### 3.1 Dépendances

- `games/riichi` : `riichienv-core = { version = "=0.4.10", default-features = false, features = ["wasm"] }`,
  `ludo-core`, `rand`, `serde`.
- `crates/ludo-wasm` seulement : `getrandom = { version = "0.4", features = ["wasm_js"] }`.
- `games/riichi` reste natif (règle de la plateforme) : pas de dépendance
  navigateur.

### 3.2 L'état

```rust
pub struct State {
    seed: u64,
    /// Actions appliquées à `inner` (après regroupement), dans l'ordre.
    steps: Vec<Vec<(Seat, Action)>>,
    /// Réponses déjà reçues pour la décision en cours, pas encore appliquées.
    answers: BTreeMap<Seat, Action>,
    /// Le moteur riichienv, reconstruit à la désérialisation.
    inner: GameState,
}
```

- `new` : `GameState::new(2 /* hanchan */, true /* pas de log mjai */, Some(seed), 0, GameRule::default_mjsoul())`.
- Sérialisation : `{ seed, steps, answers }` ; la désérialisation recrée
  `inner` et rejoue `steps` (≈ 10 ms pour une hanchan). Toute étape refusée
  au rejeu est une erreur de désérialisation.
- `pending` : `inner.active_players` privés des sièges présents dans
  `answers` ; vide exactement quand `inner.is_done`.
- `apply(seat, action)` :
  - refuse si `seat` n'est pas en attente ou si l'action n'est pas dans ses
    actions légales (état inchangé) ;
  - enregistre la réponse ; quand tous les `active_players` ont répondu,
    appelle `inner.step(&answers)`, pousse l'étape dans `steps` et vide
    `answers` ;
  - si `inner.last_error` est posé après `step` alors que toutes les réponses
    étaient légales, c'est un bug de riichienv ou de notre conversion, pas une
    entrée invalide : `apply` panique avec le message, et la conformance le
    détecte. Une entrée invalide, elle, est toujours refusée par `Illegal`
    avant `step`.
- `legal_actions(seat)` : actions de `inner.get_observation(seat)` converties
  (§3.3), vide si le siège n'est pas en attente.
- `outcome` : quand `inner.is_done`, rangs par score décroissant ; égalité
  départagée par l'ordre des sièges au départ (règle Mahjong Soul).

### 3.3 Les actions

Notre enum sérialisable, tuiles en notation mjai (`"1m"`…`"9s"`, `"5mr"` pour
le cinq rouge, `"E" "S" "W" "N" "P" "F" "C"`) :

```rust
pub enum Action {
    Discard { tile: Tile },
    Riichi,                       // suivie d'une défausse, comme dans riichienv
    Chi { tile: Tile, with: [Tile; 2] },
    Pon { tile: Tile, with: [Tile; 2] },
    Daiminkan { tile: Tile },
    Ankan { tiles: [Tile; 4] },
    Kakan { tile: Tile },
    Tsumo,
    Ron,
    KyushuKyuhai,
    Pass,
}
```

- Conversion dans les deux sens avec `riichienv_core::action::Action`
  (identifiants de tuile 0..135). La conversion est injective sur les actions
  légales : deux actions légales distinctes restent distinctes (le cinq rouge
  compte).
- `Kita` (sanma) n'existe pas ici.

### 3.4 L'information cachée

- **`view(Seat(i))`** construite depuis `inner.get_observation(i)` : sa main,
  toutes les combinaisons exposées, toutes les défausses (drapeaux riichi et
  tsumogiri), indicateurs de dora, scores, bâtonnets de riichi, honba, vent de
  la manche, numéro de main, vent de chaque siège, tuiles restantes au mur,
  sa tuile piochée ; aide : shanten, tenpai, attentes. Les réponses des autres
  sièges dans `answers` n'y figurent pas (seulement « a répondu »).
- En fin de main, la vue contient le résultat : mains révélées des gagnants,
  yaku (nom anglais et japonais), han, fu, points, variations de score, ou le
  motif du nul ; indicateurs d'ura-dora pour un gagnant en riichi.
- **`view(Spectator)`** : comme un siège sans main propre.
- **`determinize(s, observer)`** :
  1. rassembler les tuiles invisibles pour `observer` : mains des 3 autres,
     tuiles du mur restant, tuiles non révélées du mur mort ;
  2. les rebattre et les redistribuer aux mêmes emplacements (tailles de
     main et nombre de tuiles du mur conservés, indicateurs de dora déjà
     révélés inchangés) ;
  3. remplacer les réponses des autres sièges déjà dans `answers` par une
     action tirée au hasard parmi leurs actions légales dans l'état
     redistribué.
  La position exacte des tuiles du mur mort dans `WallState.tiles` se lit dans
  le code de riichienv au moment du plan ; les positions révélées ne bougent
  pas.

### 3.5 L'action par défaut (chrono)

`timeout_action(s, seat)` (§5.1) : `Pass` si le siège répond à un appel ;
sinon défausse de la tuile piochée si elle est légale ; sinon première
défausse légale ; sinon première action légale.

### 3.6 `RULES.md`

Renvoie aux règles Mahjong Soul implémentées par riichienv-core (preset
`default_mjsoul`, hanchan, 25 000 points) et ne décrit en propre que ce que
ludo ajoute : action par défaut au chrono, classement final.

## 4. L'IA

L'ISMCTS générique de `ludo-core` est réutilisé, avec trois ajustements.

1. **Coupure au changement de main.** `Heuristics::evaluate` devient
   `fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>>` (défaut
   `None`, Yavalath inchangé) et est consulté à chaque pas de simulation (la
   constante « 200 coups » disparaît). Riichi : dès que la main simulée
   diffère de la main de `root`, estimation du classement final par les
   scores (récompenses du classement courant, ex æquo à moitié).
2. **Politique de simulation** (`rollout_action`) : gagner (Ron/Tsumo) si
   possible ; déclarer riichi si tenpai ; défausser ce qui minimise le
   shanten (module `shanten` de riichienv), à égalité ce qui garde le plus de
   tuiles utiles ; passer les appels sauf un pon de dragon ou de vent de
   manche ou de siège (qui donne un yaku). Cette politique, avec la
   probabilité d'erreur de `Level::Easy`, constitue le niveau **Facile**
   (aucune recherche).
3. **Budgets par jeu.** `Heuristics::budget(level, timed: bool) -> Option<Budget>`
   (défaut : les budgets actuels de `Level`, en ms si `timed`, en itérations
   sinon) ; `None` = politique de simulation seule, sans recherche. Riichi :
   Facile `None`, Moyen 300 ms / 30 itérations, Difficile 1 000 ms / 300
   itérations. La probabilité d'erreur reste celle de `Level`. Une décision à une
   seule action légale est jouée sans recherche (déjà le cas).

Critère : `ludo arena riichi 40 hard easy 4` (Difficile contre 3 Facile,
sièges tournants) ≥ 0,60. En dessous, on améliore l'heuristique, pas le
seuil.

## 5. Ajouts à la plateforme

### 5.1 Action par défaut

`Game::timeout_action(s, seat) -> Option<Action>`, défaut `None` (le jeu ne
propose pas de chrono). La conformance vérifie qu'elle est légale pour chaque
siège en attente quand elle existe. Exposée par `AnyGame`/`ludo-wasm`.

### 5.2 Chrono, tenu par l'hôte

- Option du lobby en ligne, proposée seulement si le jeu a un
  `timeout_action` : « 5 + 20 s » (défaut), « 10 + 40 s », « Aucun ». Pas de
  chrono en local.
- Pour chaque décision d'un siège humain : le temps de base se décompte, puis
  la réserve du joueur (valable pour toute la partie). À épuisement, l'hôte
  applique `timeout_action` via le chemin normal (`Table.submit`, donc
  journalisé). Les IA ne sont pas chronométrées.
- `Update` gagne `clocks: { seat, baseMs, bankMs }[]` pour les sièges
  humains en attente, en **temps restant** au moment de l'envoi (les
  horloges des machines ne sont pas synchronisées) ; l'interface décompte
  localement.
- La réserve de chaque siège est sauvegardée avec la partie hébergée
  (reprise).
- Logique dans `Table` (côté hôte) ; le moteur reste sans horloge.

### 5.3 Action unique jouée d'office

Si un siège humain en attente n'a qu'une seule action légale, `Table` la
joue après ≈ 300 ms (par exemple la défausse forcée après riichi). Tous les
jeux.

### 5.4 Corrections du cycle 1 utiles à 4 joueurs

- **M1** : une connexion ne peut tenir qu'un siège ; un second `hello` sur
  une connexion déjà assise est refusé.
- **M2** : un siège IA dont la recherche échoue (erreur autre que « coup
  périmé ») réessaie une fois ; au second échec, il joue `timeout_action`
  s'il existe, sinon la première action légale, et l'erreur est signalée.

## 6. Interface

`web/src/games/riichi/{types.ts, Board.tsx, tiles.ts}`, disposition façon
Mahjong Soul :

- toi en bas, les autres à droite, en face, à gauche ; pour chacun, défausses
  en rangées de 6 (tuile de riichi couchée, tsumogiri grisées) et
  combinaisons exposées ;
- centre : manche (« Est 2 »), honba, bâtonnets, tuiles restantes, indicateurs
  de dora, scores avec vent de siège, chrono du siège actif ;
- ta main triée, tuile piochée à part, clic pour défausser ; les tuiles non
  défaussables sont celles absentes des actions légales (aucun calcul côté
  interface) ;
- boutons présents seulement s'ils sont légaux : Chi (choix si plusieurs
  combinaisons), Pon, Kan, Riichi, Tsumo, Ron, Neuf terminaux, Passer ;
  riichi = « Riichi » puis la défausse ;
- aide masquable : shanten, attentes si tenpai ;
- panneau de fin de main (main gagnante, yaku, han, fu, points, variations ou
  motif du nul), fermé par « Continuer » ou seul après 8 s ;
- fin de partie : classement final du cadre commun ;
- cible : écran d'ordinateur ; téléphone en paysage utilisable sans plus.

## 7. Tests

- **`games/riichi`** : aller-retour serde (rejeu identique) ; réponses
  simultanées (ordre indifférent, doublon refusé, `step` seulement quand tous
  ont répondu) ; conversion des actions dans les deux sens pour chaque type ;
  scénarios sur mur chargé passant par notre enveloppe (ron sur défausse, pon,
  priorité ron > pon sur une même défausse) ; `timeout_action` légale.
- **Conformance** : `check::<Riichi>` sur 20 hanchans à 4 joueurs, avec la
  vérification ajoutée de `timeout_action`.
- **IA** : critère d'arène (§4) ; non-triche (même coup quelles que soient les
  mains adverses) ; la politique gagne quand elle peut et fait baisser le
  shanten.
- **Plateforme** : Vitest pour le chrono (base, réserve, action par défaut) et
  l'action unique jouée d'office, sans PeerJS ; M1 ; M2.
- **Navigateur** (script headless) : hanchan complète contre 3 IA Facile ;
  partie en ligne à 2 humains + 2 IA avec un appel et un chrono qui expire ;
  reprise après rechargement de l'hôte.
