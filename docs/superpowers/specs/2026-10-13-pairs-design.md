# ludo-rs — Jeux du deck Pairs (spec 3)

Date : 2026-10-13
Statut : en relecture
Précédent : `2026-09-26-riichi-design.md` (cycle 2)

## 1. Objectif

Jouer sur ludo-rs aux jeux du deck Pairs (55 cartes : un 1, deux 2, … dix 10),
contre l'IA ou en ligne, et pouvoir ajouter ensuite un autre jeu de la famille
en n'écrivant que ses règles, sa politique d'IA et son plateau.

Décisions prises :

- **Dans ludo-rs**, pas de dépôt séparé : la plateforme (trait `Game`, ISMCTS,
  P2P, lobby) est réutilisée telle quelle.
- **Deux jeux dans ce cycle**, dans cet ordre : Continuous Pairs, puis
  L'Auberge des Pirates (Regent multijoueur dans son édition française, sans
  jetons).
- **Une seule crate `games/pairs`**, un module par jeu, le matériel commun
  dans un module partagé.
- **Sources des règles** : *Pairs Companion Book, PDF Edition* (Hip Pocket
  Games, 2018) pour Continuous Pairs ; la règle Origames de *Pairs – L'Auberge
  des Pirates* (2017) pour l'Auberge.
- **L'Auberge se joue en plusieurs manches**, classement par manches gagnées.
- **ngrok** : `web/vite.config.ts` autorise déjà `.ngrok-free.dev` et
  `.ngrok-free.app` ; rien à ajouter, seulement à ne pas casser.

### Critères de réussite

- Une partie complète de chaque jeu se joue contre des IA et en ligne, à tout
  nombre de joueurs permis, sans erreur moteur.
- La conformance `ludo-core` passe pour les deux jeux et pour chaque nombre
  de joueurs (dont : aucune vue ne contient la main d'un autre siège ni
  l'ordre de la pioche).
- `ludo arena pairs-continuous 400 hard easy` et
  `ludo arena pairs-auberge 400 hard easy` : Difficile marque ≥ 0,55. Ces
  jeux ont une forte part de hasard ; le chiffre mesuré est noté dans
  `CLAUDE.md`, et si le plafond réel d'un jeu est plus bas le seuil y est
  consigné avec la mesure qui le justifie.
- Ajouter un troisième jeu Pairs ne touche que : un module dans
  `games/pairs/src/`, une ligne dans `ludo-registry`, un dossier de plateau
  et une ligne dans `web/src/games/registry.ts`.

### Hors périmètre

Formulaire d'options par jeu dans le lobby (les réglages existent dans la
`Config` du moteur, avec leurs valeurs par défaut, mais ne sont pas réglables
depuis le site) ; chrono en ligne pour ces jeux (`clock: false`) ; variantes
Calamities et Pieces of Eight ; jeu à jetons (Regent du livret, variantes
Seattle/Tacoma/…) ; les autres jeux du livret ; illustrations des éditions
commerciales ; animations et sons.

## 2. Structure

```
games/pairs/
  Cargo.toml
  RULES.md              règles sourcées des deux jeux, points ❓
  src/
    lib.rs              réexporte les jeux
    deck.rs             matériel commun
    continuous/         Continuous Pairs  (id `pairs-continuous`)
    auberge/            L'Auberge des Pirates (id `pairs-auberge`)
  tests/conformance.rs
web/src/games/pairs/
  cards.tsx, pairs.css  composants partagés (carte, pile, table)
  continuous/           types.ts, Board.tsx
  auberge/              types.ts, Board.tsx
```

Chaque jeu est un type distinct qui implémente `Game`, `Determinize` et
`Heuristics`, avec son propre id, son propre état, ses propres actions et sa
propre vue. Il n'y a pas de type « Pairs » générique dont une option
choisirait le jeu : les états n'ont rien en commun au-delà des cartes.

`crates/ludo-registry` reçoit deux `static` et deux entrées dans `GAMES`.

## 3. Matériel commun (`deck.rs`)

- `Rank` : entier 1–10, validé à la construction et à la désérialisation.
  Une carte n'a pas d'autre identité que son rang.
- `full_deck()` : les 55 cartes.
- `Supply` : la pioche et la défausse.
  - `draw(rng)` rend la carte du dessus. Si la pioche est vide, la défausse
    est mélangée et devient la pioche, puis `burn` cartes (réglage du jeu :
    5 pour Continuous Pairs, 0 pour l'Auberge) passent face cachée à la
    défausse. Rend `None` seulement si pioche et défausse sont vides.
  - `discard(cards)`.
  - La défausse distingue les cartes vues de tous de celles que personne n'a
    vues (cartes brûlées, mains défaussées face cachée).
- Fonctions pures : `has_rank(cards, rank)`, `pair_in(cards)` (le plus haut
  rang présent deux fois), `lowest(cards)`.
- `redeal_unseen(...)` : aide à la déterminisation. Reçoit les cartes que
  l'observateur ne peut pas situer et les emplacements cachés (pioche, cartes
  non vues de la défausse, mains des autres avec leur taille) et les
  redistribue au hasard. Chaque jeu décide ce qui est « non vu » ; l'aide
  garantit que le total reste de 55 cartes.

Le hasard vient de la graine donnée à `new`, gardée dans l'état, comme pour
les autres jeux. L'état n'apparaît jamais dans une vue.

## 4. Continuous Pairs (`pairs-continuous`, 2–8 joueurs)

Source : Companion Book, « How to Play Pairs », « Breaking a Tie for Low »,
« Continuous Pairs ».

### Règles

- **Début** : mélanger, brûler 5 cartes. Une carte visible à chaque joueur.
  La plus petite commence. En cas d'égalité, les joueurs à égalité reçoivent
  une carte de plus, comparée entre eux, et ainsi de suite ; une carte de
  départage qui ferait une paire est défaussée et remplacée. Les cartes de
  départage restent dans la pile du joueur.
- **Tour** : le joueur choisit `Hit` ou `Fold`. Le tour passe ensuite à
  gauche.
  - `Hit` : il reçoit une carte visible. Si elle fait une paire avec sa pile,
    il marque le rang de la paire, garde une des deux cartes comme carte de
    score, et le reste de sa pile est défaussé.
  - `Fold { seat, card }` : il prend n'importe quelle carte en jeu (sa pile
    ou celle d'un autre), la garde comme carte de score, marque son rang, et
    sa propre pile est défaussée.
  - Un joueur sans carte ne choisit pas : le moteur lui donne une carte
    (« automatically hit ») et passe au suivant.
- **Les cartes de score sortent du jeu** : elles ne retournent pas dans la
  pioche.
- **Fin** : dès qu'un joueur atteint le score cible, la partie s'arrête. Il
  est classé dernier ; tous les autres sont premiers ex æquo.

  | Joueurs | 2 | 3 | 4 | 5 | 6–8 |
  |---|---|---|---|---|---|
  | Cible | 31 | 21 | 16 | 13 | 11 |

- ❓ **Aucune carte à piocher** (pioche et défausse vides, cas théorique) :
  `Hit` n'est pas proposé ; un joueur sans carte passe son tour.

### Modèle

- `Config { target: Option<u8> }` : `None` = la table ci-dessus.
- `Action` : `Hit`, `Fold { seat: Seat, card: Rank }`.
- `pending` : un seul siège, celui dont c'est le tour.
- `View` (identique pour tous, le jeu n'a pas de main cachée) : piles de
  chaque siège, scores et cartes de score, cible, siège au tour, taille de la
  pioche et de la défausse, nombre de cartes de chaque rang que personne n'a
  encore situées (calculé par le moteur, pour l'affichage des risques), et
  les derniers événements (carte reçue, paire, abandon, pioche automatique,
  remélange) pour que le plateau montre ce qui vient de se passer.
- `hidden_info: true` : l'ordre de la pioche et les cartes brûlées sont
  inconnus. `determinize` remélange la pioche avec les cartes brûlées.

### IA

- `rollout_action` : compare le coût sûr d'un abandon (plus petite carte en
  jeu) au coût espéré d'une carte (pour chaque rang de sa pile, probabilité
  de le tirer × rang), pondérés par la distance à la cible ; un peu de hasard
  entre deux choix proches.
- `search_actions` : `Hit` et les abandons sur les deux plus petites cartes
  en jeu, pour ne pas élargir la recherche avec des abandons absurdes.
- `evaluate` : 1 − (score / cible) relatif aux autres sièges, pour couper
  les simulations longues.

## 5. L'Auberge des Pirates (`pairs-auberge`, 2–8 joueurs)

Source : règle Origames. Les points que la règle ne tranche pas sont marqués
❓ et consignés dans `RULES.md`.

### Règles d'une manche

- **Mise en place** : mélanger ; 3 cartes en main à chacun. Tous choisissent
  en même temps une carte de leur main et la posent visible devant eux. La
  plus petite commence ; à égalité, tirage au sort entre les joueurs à
  égalité (la règle dit « le plus âgé »).
- **Tour** : une action parmi :
  - `Draw` (Piocher) : une carte de la pioche va dans sa main, sans être
    montrée. Tour au joueur à gauche.
  - `Attack { target, card }` (Attaquer) : une carte de sa main est posée
    visible devant `target`. Interdit si `target` a déjà ce rang devant lui.
    Tour à `target`.
  - `Dare { target }` (Défier) : la carte du dessus de la pioche est posée
    visible devant `target`. Si elle fait une paire avec les cartes visibles
    de `target`, elle est défaussée, le joueur actif est éliminé et le tour
    va à sa gauche. Sinon tour à `target`.
  - `Accuse { target }` (Accuser) : `target` montre sa main au seul
    accusateur. S'il existe deux cartes de même rang parmi sa main et les
    cartes devant lui, `target` est éliminé et le tour va à gauche de
    l'accusateur. Sinon l'accusateur est éliminé et le tour va à `target`.
- « À gauche » saute les joueurs éliminés.
- **Élimination** : les cartes devant le joueur sont défaussées visibles, sa
  main est défaussée face cachée (❓ la règle ne dit pas si elle est montrée).
- **Fin de manche** : le dernier joueur en jeu gagne la manche.
- ❓ **Accuser un joueur sans carte en main** : non proposé (il ne peut pas
  avoir de paire ; le livret anglais l'interdit pour la même raison).
- ❓ **Pioche vide** : la défausse est remélangée. Si pioche et défausse sont
  vides, `Draw` et `Dare` ne sont pas proposés ; si un joueur n'a alors
  aucune action possible, la manche s'arrête et les joueurs encore en jeu se
  la partagent.

### Partie

`Config { rounds: u8 }`, 5 par défaut. Tous les joueurs reviennent à chaque
manche. Le vainqueur d'une manche marque 1 point (une manche partagée donne
une part égale à chacun). Classement final par points, les égalités
restent des égalités.

### Modèle

- `Action` : `Place { card }` (mise en place), `Draw`, `Attack`, `Dare`,
  `Accuse`.
- `pending` : tous les sièges qui n'ont pas encore posé pendant la mise en
  place ; ensuite un seul siège. La carte posée par un siège n'apparaît dans
  les vues des autres qu'une fois toutes les cartes posées.
- `View` pour un siège : sa main ; pour chaque siège, les cartes devant lui,
  la taille de sa main, s'il est éliminé, ses points ; siège au tour ; numéro
  de manche ; tailles de la pioche et de la défausse ; cartes visibles de la
  défausse ; derniers événements. Après une accusation, la main montrée
  figure dans les événements de l'accusateur seulement ; les autres voient
  qui a accusé qui et le résultat.
- L'état retient, par siège, les cartes cachées qu'il a vues (mains montrées
  par une accusation). `determinize` les laisse en place et redistribue le
  reste : mains des autres, pioche, défausse face cachée.
- `hidden_info: true`, `clock: false`.

### IA

- `rollout_action` : estime pour chaque adversaire la probabilité qu'il ait
  une paire, d'après la taille de sa main, ses cartes visibles et les cartes
  non situées ; accuse au-dessus d'un seuil ; attaque avec une carte qui fait
  paire dans sa propre main quand c'est possible (elle se débarrasse d'une
  paire) ; défie un joueur qui a peu de rangs visibles ; pioche sinon.
- La recherche ISMCTS générique fait le reste ; `search_actions` garde toutes
  les actions légales (le nombre reste petit).

## 6. Web

- `cards.tsx` : `Card` (un rang, face visible ou dos), `Stack`, et une table
  qui dispose jusqu'à 8 sièges autour du centre (pioche, défausse). Dessin
  propre au projet : un rang et une couleur par rang, aucune illustration
  des éditions commerciales.
- Un `Board` par jeu : il affiche la vue et propose les actions de
  `legalActions`. Il ne calcule rien : les risques affichés, les cibles
  permises et les événements viennent de la vue.
- `types.ts` de chaque jeu reflète les types sérialisés du moteur, mis à
  jour dans le même changement que le moteur.
- Textes de l'interface en français.

## 7. Tests

- `deck.rs` : 55 cartes, répartition par rang ; `Supply` conserve le total à
  travers remélanges et brûlages ; `Rank` refuse 0 et 11.
- Continuous Pairs : l'exemple de départage du livret ; paire → score et
  pile défaussée, les autres piles intactes ; abandon sur la carte d'un autre
  joueur ; pioche automatique d'un joueur sans carte ; cible par nombre de
  joueurs ; fin et classement.
- Auberge : mise en place simultanée et premier joueur ; passage du tour
  pour chacune des quatre actions, y compris après élimination ; attaque
  refusée si elle ferait une paire visible ; accusation juste et fausse ;
  la main montrée n'apparaît que dans la vue de l'accusateur ; enchaînement
  des manches et classement.
- `apply` rend `Illegal` sans modifier l'état pour toute action hors de
  `legal_actions`.
- `tests/conformance.rs` : `ludo_core::conformance::check` pour les deux
  jeux, de 2 à 8 joueurs.
- Web : `pnpm test` et `pnpm build` passent ; un test par plateau qui rend
  une vue d'exemple et vérifie qu'une action soumise est une de
  `legalActions`.

## 8. Livraison

Deux plans, chacun fusionnable seul :

1. `deck.rs`, Continuous Pairs (moteur, IA, registre), composants de cartes
   partagés et plateau.
2. L'Auberge des Pirates (moteur, IA, registre, plateau), puis la section
   « Ajouter un jeu Pairs » de `CLAUDE.md`.

`NOTICE` et `README.md` citent les deux sources ; `RULES.md` reformule les
règles et ne recopie pas les livrets, qui ne sont pas versionnés.

## 9. Ajustements faits au plan 1

- `Supply::draw()` ne prend pas de `rng` : la graine des remélanges est dans
  le `Supply`, pour qu'un remélange se rejoue à l'identique depuis le
  `MatchLog`. `determinize` la retire au hasard (`redeal_hidden`).
- Au remélange, on ne brûle jamais la dernière carte.
- Continuous Pairs ne fournit pas `Heuristics::evaluate` : les simulations
  vont jusqu'au bout, elles sont courtes.
- `pair_in`, la défausse face cachée d'une main et la redistribution des
  mains arrivent avec l'Auberge (plan 2).
