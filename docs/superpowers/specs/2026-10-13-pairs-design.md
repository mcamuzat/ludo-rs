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
- Après la revue finale : les mélanges utilisent `ChaCha8Rng` (identique en
  wasm32 et en natif) ; une cible de config est plafonnée à 31 ; le risque de
  paire compte toute la défausse quand la pioche est vide.

## 10. Ajustements faits au plan 2 (L'Auberge des Pirates)

- `Config { rounds: Option<u8> }` : `None` ou 0 = 5 manches, plafond à 15.
- Une manche bloquée ne rapporte de point à personne (au lieu d'être
  partagée) : les points restent des entiers.
- Pendant la mise en place, un siège ne voit que sa propre attente : la
  déterminisation refait le choix des autres.
- Le résultat d'une accusation est public ; la main montrée n'apparaît que
  chez l'accusateur et l'accusé, et le rang de la paire n'est pas annoncé.
- La table web est carrée et le journal défile, pour que le centre ne
  recouvre pas les sièges.
- À la demande de l'utilisateur après essai : la vue et le plateau de
  l'Auberge ne donnent aucune statistique (ni paire estimée par joueur, ni
  compte des cartes non situées). Seule reste l'alerte sur sa propre paire.
  L'estimation sert toujours à l'IA.

## 11. Ghost Town (plan 3)

Ajouté après les deux jeux du périmètre initial, sans changement de
plateforme ni de `deck.rs`. Décisions de l'utilisateur : 5 mains par partie
(`Config { hands }`, plafond 15), soldes de jetons partant de 0 et pouvant
être négatifs, classement aux jetons ; le risque de paire n'est plus affiché
non plus dans Continuous Pairs. Les points non tranchés par le livret sont
dans `games/pairs/RULES.md`.

## 12. Zombie Life (plan 4)

Variante Origames de Pairs (`pairs-zombie-life`, 3–6 joueurs), ajoutée sans
changement de plateforme. Source : la règle Origames en 4 pages
(*Pairs – Zombie Life*). Aucune statistique affichée, comme les autres jeux.

### Règles

- **But** : premier à 21 points (`TARGET`), en plusieurs manches.
- **Première manche** : mélanger, une carte visible dans le Refuge de chaque
  joueur. La plus petite carte commence.
- **Tour** : une action parmi trois.
  - `Draw` : piocher une carte, que le joueur voit, puis `Place { zone }`
    dans son Refuge ou dans la Horde (zone commune), là où le rang est
    absent. Rang présent dans les deux : le joueur est éliminé de la manche,
    son Refuge et la carte piochée sont défaussés.
  - `Take { rank }` : prendre dans la Horde une carte dont le rang est absent
    de son Refuge. Interdit au dernier joueur en jeu.
  - `Pass` : le joueur ne joue plus de la manche. Il retourne face cachée
    autant de cartes de son Refuge qu'il reste de joueurs en jeu ; à chaque
    élimination ensuite, il en remet une face visible.
- **Fin de manche** : quand tous ont passé ou sont éliminés, chaque joueur
  qui a passé marque 1 point par carte visible. Un joueur à 21 ou plus : la
  partie s'arrête, le plus de points gagne. Sinon nouvelle manche.
- **Nouvelle manche** : le joueur au plus petit score (à égalité, le premier
  éliminé de la manche) pioche autant de cartes qu'il y a de joueurs et en
  place une devant chacun, une à une : `Give { card }` sert le prochain siège
  non servi, à partir de la gauche du donneur, le donneur en dernier. Des
  cartes toutes identiques sont données sans demander. La plus petite carte
  commence.

### Points non tranchés par la règle

- Cartes retournées en passant : le nombre de joueurs encore en jeu **sans**
  celui qui passe, plafonné à la taille de son Refuge.
- Toutes les cartes valent 1 point : l'état ne garde qu'un **nombre** de
  cartes cachées par siège, pas un choix de cartes.
- Fin de manche : la Horde est défaussée avec les Refuges.
- Pioche : mélangée une seule fois ; les défausses sont remélangées quand
  elle est vide (`Supply`).
- Égalité sur la plus petite carte (« le plus âgé ») : tirage sur la graine.
- Égalité pour le donneur sans aucun éliminé parmi les ex æquo : tirage sur
  la graine.
- Égalité au score final : victoire partagée.

### Modèle

`games/pairs/src/zombie_life/` (`mod.rs`, `view.rs`, `determinize.rs`,
`heuristics.rs`, `testing.rs`) sur `deck.rs`. Phases : `Dealing`, `Play`,
`Placing` (carte piochée en attente), `Over`. La carte piochée et les cartes
du donneur sont publiques : personne d'autre n'agit avant qu'elles soient
posées, les cacher ne changerait aucune décision. Seule information cachée :
l'ordre de la pioche (`determinize` = `redeal_hidden` et nouvelle graine).
`Config { target }` (21 par défaut).

### Livraison

`RULES.md`, `tests/zombie_life_rules.rs`, conformité à 3, 4, 5 et 6 joueurs,
entrée `ludo-registry`, `web/src/games/pairs/zombie-life/` (`types.ts`,
`Board.tsx`, test), entrée `registry.ts`,
`ludo-cli arena pairs-zombie-life 400 hard easy`.

## 13. Goblin Poker (plan 5)

Jeu à jetons du livret (`pairs-goblin-poker`, 3–8 joueurs), ajouté sans
changement de plateforme. Source : *Pairs Companion Book*, chapitre « Goblin
Poker ». Aucune statistique affichée.

### Règles d'une main

- **Mise en place** : chacun mise 1 jeton (`ANTE`) et reçoit 6 cartes
  (`HAND`). Tous choisissent une carte en secret (`Place { card }`) ;
  révélées ensemble, elles commencent la pile visible de chacun.
- **Tour de table** : en partant de la pile la plus basse, puis vers la
  gauche, chaque joueur encore en jeu choisit :
  - `Fold` : payer au pot la plus petite carte en jeu, et sortir de la main ;
  - `Pitch { card }` : jeter une carte de sa main, face cachée, au centre.
- **Redistribution** : quand tous ont agi, les cartes jetées sont mélangées
  et données face visible, une par joueur restant, en partant de la pile la
  plus basse puis vers la gauche. Qui reçoit une paire sort et paie le rang
  au pot. Les éliminations se font dans l'ordre : dès qu'il ne reste qu'un
  joueur, il gagne, même s'il allait recevoir une paire.
- **Fin de la main** : le dernier joueur en jeu prend le pot. Sinon nouveau
  tour de table, cinq au plus ; les survivants du cinquième se partagent le
  pot, le reste de la division restant au pot pour la main suivante.
- **Pile la plus basse** : la plus petite carte, puis la deuxième plus
  petite, etc. ; une carte en plus est plus basse que pas de carte (2-3-6
  est plus bas que 2-3).

### Points non tranchés par le livret

- Partie en 5 mains (`Config { hands }`, plafond 15), soldes partant de 0 et
  pouvant être négatifs, classement aux jetons, égalités conservées : comme
  Ghost Town. Les jetons restés au pot après la dernière main ne vont à
  personne.
- « Plus petite carte en jeu » : la plus petite carte visible des piles des
  joueurs encore en jeu, celle du joueur qui se couche comprise.
- La pile la plus basse est recalculée au début de chaque tour de table, et
  de nouveau avant la redistribution, parmi les joueurs restants.
- Égalité parfaite pour la pile la plus basse : chaque pile à égalité reçoit
  une carte de la pioche, ajoutée à la pile ; une carte qui ferait une paire
  est défaussée et remplacée. Les autres égalités ne sont pas départagées.
- Pioche vide pendant un départage : le premier des ex æquo dans l'ordre
  des sièges. La pioche n'est pas remélangée.
- Un joueur qui sort défausse sa pile face visible et sa main face cachée.

### Modèle

`games/pairs/src/goblin_poker/` (`mod.rs`, `view.rs`, `determinize.rs`,
`heuristics.rs`, `testing.rs`) sur `deck.rs`. Phases : `Setup`, `Play`,
`Over`. Information cachée : les mains, la carte choisie par un autre
pendant la mise en place, les cartes jetées. `determinize` redistribue les
mains des autres, leurs cartes jetées, la pioche et les défausses cachées.
Pendant la mise en place, chacun ne se voit que lui-même en attente.

### Livraison

`RULES.md`, `tests/goblin_poker_rules.rs`, conformité de 3 à 8 joueurs,
entrée `ludo-registry`, `web/src/games/pairs/goblin-poker/` (`types.ts`,
`Board.tsx`, test), entrée `registry.ts`,
`ludo-cli arena pairs-goblin-poker 400 hard easy 3`.

## 14. To the Moon! (plan 6)

Jeu de plis de la série mensuelle Cheapass 2018 (`pairs-to-the-moon`, 2–6
joueurs), ajouté sans changement de plateforme. Source : fiche *To the
Moon!* (`~/Téléchargements/pairs/5CAGToTheMoon.pdf`). Règles et points non
tranchés : `games/pairs/RULES.md`. Aucune statistique affichée.

### Modèle

`games/pairs/src/to_the_moon/` sur `deck.rs`. Actions : `Build { card }`,
`Test`, `Withdraw { take: Option<Rank> }`. Information cachée : les mains,
sauf les cartes `known` que tout le monde a vues y entrer, et l'ordre de la
pioche. `determinize` redistribue la partie inconnue des autres mains et la
pioche. Les cartes gagnées vont aux défausses de la `Supply`, jamais
remélangées (le tirage est gardé par `deck_len() > 0`).

### IA

Politique de simulation : tester tant que le risque d'appariement (vu du
siège) est faible, sinon construire le plus haut rang, sinon tester si le
risque reste modéré, sinon se retirer avec la plus petite carte.

## 15. Deadfall (plan 7)

Jeu de bluff du livret (`pairs-deadfall`, 2–7 joueurs), règles de base sans
variantes, ajouté sans changement de plateforme. Source : *Pairs Companion
Book*, chapitre « Deadfall ». Règles et points non tranchés :
`games/pairs/RULES.md`. Aucune statistique affichée.

### Modèle

`games/pairs/src/deadfall/` sur `deck.rs`. Phases : `Door` (choix
simultané, rejoué tant que l'égalité pour la plus basse demeure), `Play`,
`Over`. Actions : `Door { card }`, `Play { card }`, `Call { seat }`. Chaque
pile garde l'horodatage de sa carte du dessus (`stamps`, `clock`) pour la
restriction d'appel. Information cachée : les mains, les portes choisies par
les autres, les cartes non distribuées (pioche de la `Supply`).

### IA

Politique de simulation : appeler la carte du dessus la plus sûrement morte
(probabilité que toutes ses copies invisibles soient parmi les cartes non
distribuées) si elle dépasse 0,7 ou si la main est vide ; sinon jouer la
carte la plus sûrement vivante (une paire d'abord). La recherche ne délibère
pas la porte : la plus basse.

## 16. Captain's Orders (plan 8)

Jeu de bluff et de déduction de la série mensuelle Cheapass 2018
(`pairs-captains-orders`, 4–8 joueurs), ajouté sans changement de
plateforme. Source : `~/Téléchargements/pairs/8CAGCapsOrders.pdf`. Règles et
points non tranchés : `games/pairs/RULES.md`. Aucune statistique affichée.

### Modèle

`games/pairs/src/captains_orders/` sur `deck.rs`. Phases : `Turn` (pas
d'ordre sur la table), `Respond` (un ordre circule), `Give { asker }` (le
joueur sollicité choisit la carte qu'il donne), `Discard { after }` (le
receveur défausse face cachée), `Over`. Actions : `Order { card }` (jouer un
ordre, ou annuler l'ordre en cours avec une carte plus basse), `Follow
{ card }`, `Pass { card, to }`, `Ask { from }`, `Give { card }`, `Discard
{ card }`, `Accuse { seat }`. La déclaration de Capitaine quand une main se
vide est automatique (ne pas se déclarer ne peut rien rapporter).

Information cachée : les mains, les cartes non distribuées, les défausses
face cachée (chacun garde les siennes), les cartes passées (connues de qui
donne et de qui reçoit). Le journal de la manche (`log`) garde chaque carte ;
la vue masque celles que le spectateur ne connaît pas. Le résultat de la
manche précédente reste affiché (`last_round`).

### IA

Première version : ISMCTS sur une redistribution uniforme des cartes
cachées (les défausses de l'observateur restent en place). Politique de
simulation : un joueur qui se croit Capitaine (probabilité qu'aucune autre
main ne tienne plus bas que sa plus basse ≥ ½) se vide la main en gardant sa
plus basse pour la fin ; un Mutin joue sa plus basse (suivre ou annuler),
ordonne avec une carte haute, demande sinon une carte, défausse et donne sa
plus basse. Les accusations ne sont délibérées que par la recherche.

Seconde version : la lecture. Les gestes publics de la manche trahissent
un rôle (`reading.rs`) : passer une carte est un geste de Capitaine,
demander une carte ou ordonner de soi-même un geste de Mutin (annuler ne dit
rien). Dès qu'il y en a, la déterminisation tire 8 donnes et en garde une
avec une probabilité proportionnelle à leur vraisemblance (×0,3 par geste
qui ne cadre pas avec le rôle du joueur dans la donne). Mesuré en
Difficile contre Difficile sur les mêmes parties à 4 joueurs : 0,566 contre
0,470 pour l'ancienne IA face à elle-même ; à 8 joueurs le gain se perd
dans le bruit (0,478 contre 0,455 sur 200 parties : chacun y joue peu de
coups par manche, donc trahit peu). Les variantes 0,15 et/ou 16
donnes font moins bien.

Bluff : deux pistes essayées puis écartées. Restreindre les accusations
(dans la recherche, ou des seuls adversaires simulés) aux joueurs qui se
sont trahis fait perdre (0,510, et 0,46 sans la lecture) : la recherche
sous-estime alors des adversaires qui accusent bien au compte des cartes.
Des accusations sur soupçon dans les simulations ne changent aucune
décision. L'ISMCTS à observateur unique ne prête pas de croyances aux
adversaires simulés : bluffer n'y rapporte rien, et il faudrait un autre
algorithme (MO-ISMCTS, ou une recherche qui modélise les croyances) pour
qu'il apprenne à le faire.

## 17. Pendleton (plan 9)

Jeu à deux du livret (`pairs-pendleton`), ajouté sans changement de
plateforme. Source : *Pairs Companion Book*, chapitre « Pendleton ». Règles
et points non tranchés : `games/pairs/RULES.md`.

### Modèle

`games/pairs/src/pendleton/` sur `deck.rs`. Une seule action, `Play
{ column }` : la carte est piochée par le moteur au début du tour
(`drawn`, visible de tous), et une carte injouable termine la manche aussitôt.
Seul l'ordre de la pioche est caché : `determinize` la remélange et tire une
nouvelle graine pour les donnes à venir. Les cibles sont tirées d'un premier
mélange, la pioche des 50 autres cartes d'un second.

### IA

Politique de simulation : la colonne où la carte fait gagner le plus de
points nets (au hasard parmi les ex æquo). `evaluate` arrête la simulation
à la fin de la manche en cours, sur l'écart de points qu'elle a donné.
Mesuré : Difficile 0,93 contre Facile (100 parties).

## 18. Kitty (plan 10)

Jeu d'enchères du livret (`pairs-kitty`, 3–6 joueurs), variante douce hors
4 joueurs, ajouté sans changement de plateforme. Source : *Pairs Companion
Book*, chapitres « Kitty » et « Kitty Variants ». Règles et points non
tranchés : `games/pairs/RULES.md`.

### Modèle

`games/pairs/src/kitty/` sur `deck.rs`, sans `Supply` (une seule donne, pas
de remélange). Phases : `Pass` et `Bid` (choix simultanés, la carte choisie
reste en main jusqu'à la révélation), `Claim` (les mises gagnantes, de la
plus basse à la plus haute ; une prise sans choix se fait seule), `Over`.
Actions : `Pass { card }`, `Bid { card }`, `Take { card }`, `Give { card,
to }`. Information cachée : les mains, l'ordre de la kitty, les cartes
écartées, les choix des autres. Chacun sait ce qu'il a passé et reçu à la
manche : `determinize` laisse la carte passée par l'observateur chez son
voisin de gauche tant qu'il ne l'a pas misée, et redistribue le reste.

### IA

Politique de simulation, à partir de ce que le siège voit : l'envie d'une
carte est son rang, multiplié par un plus le nombre déjà en main. Passer la
moins voulue ; miser sa plus basse carte si la cible vaut au moins 12,
sinon une des trois moins voulues ; prendre la plus voulue ; donner la plus
basse à un joueur au hasard. Mesuré : Difficile contre Facile 0,69 à 4
joueurs, 0,69 à 3, 0,58 à 6 (200 parties).

## 19. Octopus (plan 11)

Jeu de main haute du livret (`pairs-octopus`, 4–6 joueurs), ajouté sans
changement de plateforme. Source : *Pairs Companion Book*, chapitre
« Octopus ». Règles et points non tranchés : `games/pairs/RULES.md`.
Aucune statistique affichée.

### Modèle

`games/pairs/src/octopus/` sur `deck.rs`, une donne mélangée par main
(`mix(seed, main)`), sans `Supply`. Phases : `Declare` (chacun à son tour),
`Play` et `Pass` (choix simultanés, la carte reste en main jusqu'à la
révélation), `Call` (la plus basse carte unique), `Over`. Actions : `Stay`,
`Fold`, `Play { card }`, `CallPass`, `Show { best: Kind }`, `Pass
{ card }`. `strength` classe une main (type choisi, puis l'autre type non
sale, puis sale ; à type égal, cartes triées décroissantes, plus bas =
mieux). Information cachée : les mains, les cartes non distribuées, les
choix des autres ; `tracked` garde la carte que chacun a passée tant que
son receveur n'a pas joué ce rang, et `determinize` la laisse en place.

### IA

Politique de simulation, à partir de ce que le siège voit : rester ; jouer
et passer la carte dont la perte laisse la main la plus forte (sous le type
qui lui convient le mieux) ; appeler l'abattage dans ce type si la main
n'est pas sale et que sa plus haute carte vaut 7 au plus, sinon faire
passer. Mesuré : Difficile contre Facile 0,82 à 4 joueurs, 0,78 à 6 (200
parties).

## 20. Port (plan 12)

Jeu de stop-ou-encore du livret (`pairs-port`, 2–7 joueurs), ajouté sans
changement de plateforme. Source : *Pairs Companion Book*, chapitre
« Port ». Règles et points non tranchés : `games/pairs/RULES.md`. Aucune
statistique affichée.

### Modèle

`games/pairs/src/port/` sur `deck.rs`. Le départage d'ouverture de Pairs
(une carte chacun, cartes de plus aux ex æquo, une carte qui ferait paire
remplacée) vit désormais dans `deck::open_round`, partagé avec Jour de
chance. Actions : `Hit`, `Lock`. Chaque siège a un `Status` (`Playing`,
`Locked`, `Busted`, `Out` pour qui ne joue pas une manche de départage).
Seuls l'ordre de la pioche et les cartes brûlées sont cachés.

### IA

Politique de simulation : s'arrêter si le risque de paire (vu de tous) est
d'au moins ½, ou d'au moins ¼ quand le total dépasse déjà toutes les piles
arrêtées ; sinon prendre. Mesuré : Difficile contre Facile 0,87 à 2
joueurs, 0,75 à 5 (200 parties).

## 21. Starboard (plan 13)

Jeu de bluff collectif du livret (`pairs-starboard`, 2–7 joueurs), ajouté
sans changement de plateforme. Source : *Pairs Companion Book*, chapitre
« Starboard ». Règles et points non tranchés : `games/pairs/RULES.md`.
Aucune statistique affichée.

### Modèle

`games/pairs/src/starboard/` sur `deck.rs`, un paquet complet mélangé par
manche (`mix(seed, manche)`). Actions : `Hit`, `Lock`. Chaque siège a un
`Status` (`In`, `Locked`, `Busted`, `Knocked`) ; le dernier encore en jeu se
bloque d'office. Seuls l'ordre de la pioche et les mélanges à venir sont
cachés.

### IA

Politique de simulation : se bloquer quand le risque de paire au prochain
tirage (vu de tous) atteint 0,55, ou 0,7 s'il y a des joueurs bloqués à
éliminer ; sinon tirer. Mesuré : Difficile contre Facile 0,59 à 3 joueurs,
0,57 à 2 (400 parties), 0,65 à 6 (200) : le hasard pèse lourd.

## 22. Sweep (plan 14)

Jeu de collection du livret (`pairs-sweep`, 3–6 joueurs), ajouté sans
changement de plateforme. Source : *Pairs Companion Book*, chapitre
« Sweep ». Règles et points non tranchés : `games/pairs/RULES.md`.

### Modèle

`games/pairs/src/sweep/` sur `deck.rs`, une seule donne. Phases : `Play`
(chacun tire à son tour, face visible), `Capture` (un tireur à égalité
choisit sa cible ; une prise sans choix se fait seule), `Over`. Actions :
`Shoot { card }`, `Capture { card }`. Les prises se résolvent par groupes de
tireurs de même rang, du plus bas au plus haut (`groups`, `tie`). Un tireur
qui tombe au milieu quitte `shooters`. Information cachée : les mains et la
pioche ; `known` garde les cartes ramassées (vues de tous) tant qu'elles
n'ont pas été rejouées, et `determinize` les laisse dans leur main.

### IA

Politique de simulation : tirer sa plus haute carte si elle dépasse toutes
les cibles, sinon sa plus basse ; ramasser la plus haute cible. Mesuré :
Difficile contre Facile 0,70 à 3 joueurs, 0,61 à 5 (200 parties).

## 23. Rocket (plan 15)

Jeu de casino du livret (`pairs-rocket`, 1–6 joueurs contre une banque
automatique), ajouté sans changement de plateforme. Source : *Pairs
Companion Book*, chapitre « Rocket ». Règles et points non tranchés :
`games/pairs/RULES.md`. Aucune statistique affichée ; la vue donne le coût
d'un abandon (`fold_cost`), qui est une règle et non une estimation.

### Modèle

`games/pairs/src/rocket/` sur `deck.rs` ; `Supply::reshuffle` (nouveau)
remélange le paquet quand il en reste moins de 20 %. La banque n'est pas un
siège : le moteur la joue dans `apply`, après le dernier joueur encore en
jeu. Actions : `Hit`, `Fold`. Seul l'ordre de la pioche est caché. À un seul
joueur, le classement n'a pas de sens : seul compte le solde contre la
banque.

### IA

Politique de simulation : se coucher quand le coût attendu d'un tirage
(probabilité de chaque rang qui fait paire × sa valeur, vue de tous)
dépasse le coût de l'abandon. Mesuré : Difficile contre Facile 0,73 à 2
joueurs, 0,70 à 4 (400 parties).

## 24. Monster (plan 16)

Jeu d'argent du livret (`pairs-monster`, 4–8 joueurs), ajouté sans
changement de plateforme. Source : *Pairs Companion Book*, chapitre
« Monster ». Règles et points non tranchés : `games/pairs/RULES.md`.

### Modèle

`games/pairs/src/monster/` sur `deck.rs`, un paquet complet mélangé par
manche (`mix(seed, donne)`). Une seule phase de jeu : chacun agit une fois,
`Discard { cards }` (0 à 3 cartes, triées, ne laissant aucune paire ; la
liste est calculée par le moteur) ou `Fold`. L'abattage, les pénalités, les
sorties, le rejeu et le gain du pot se résolvent seuls. Information cachée :
les mains, les écarts des autres (la main du Monstre), les cartes couchées
et les cartes non distribuées. Le web choisit les cartes à écarter, puis
cherche l'action correspondante dans la liste légale.

### IA

Politique de simulation : écarter le moins de cartes possible en gardant la
plus haute main (le Monstre reçoit moins), se coucher si aucun écart ne
défait les paires. Mesuré : Difficile contre Facile 0,68 à 4 joueurs comme
à 6 (200 parties).

## 25. Troll Guts (plan 17)

Jeu de jetons du livret (`pairs-troll-guts`, 3–6 joueurs), ajouté sans
changement de plateforme. Source : *Pairs Companion Book*, chapitre « Troll
Guts ». Règles et points non tranchés : `games/pairs/RULES.md`.

### Modèle

`games/pairs/src/troll_guts/` sur `deck.rs` (`open_round` pour l'ouverture).
Phases : `Arrange` (chacun range sa pile cachée en secret, en même temps ;
l'action donne l'ordre complet, vérifié comme permutation de la pile),
`Play`, `Over`. Actions : `Arrange { order }`, `Draw`, `Steal { from }`,
`Buy`. Information cachée : les piles des autres (cartes et ordre), la
pioche, les ordres pas encore révélés. Le web range la pile carte par carte,
puis cherche l'ordre dans la liste légale.

### IA

Politique de simulation : ranger la pile en mettant au-dessus les cartes
qui ne font pas paire avec ses cartes visibles ; acheter le dessus de sa
pile s'il est sûr et qu'une carte inconnue risque la paire (plus de 15 %),
sinon piocher ou voler au hasard. La recherche ne délibère que trois
rangements (sûr, croissant, décroissant). Mesuré : Difficile contre Facile
0,61 à 3 joueurs (200 parties), 0,58 à 5 (400) : le jeu est très chaotique.

## 26. Blackstone (plan 18)

Jeu de « chicken » du livret (`pairs-blackstone`, 2–8 joueurs), ajouté sans
changement de plateforme. Source : *Pairs Companion Book*, chapitre
« Blackstone ». Règles et points non tranchés : `games/pairs/RULES.md`.

### Modèle

`games/pairs/src/blackstone/` sur `deck.rs`. Actions : `Take { count }`
(de `min_take` à la taille du paquet) et `Fold`. L'ouverture a son propre
départage : les paires de pierres blanches restent, les 10 en trop
retournent au paquet (`Supply::reshuffle`). Les mises devant les joueurs ne
sont pas dans le pot : jetons + pot + mises devant = 0. Seul l'ordre du
paquet est caché ; la vue donne le coût d'un abandon (une règle).

### IA

Politique de simulation : prendre le minimum sauf si la probabilité
(hypergéométrique, vue de tous) d'une seconde pierre noire dépasse ½, sinon
se coucher. La recherche ne délibère que jusqu'à 3 cartes au-dessus du
minimum. Mesuré : Difficile contre Facile 0,74 à 2 joueurs, 0,67 à 5 (200
parties).

## 27. Horn of Plenty (plan 19)

Jeu de bluff du livret (`pairs-horn-of-plenty`, 2–6 joueurs), ajouté sans
changement de plateforme. Source : *Pairs Companion Book*, chapitre « Horn
of Plenty » (p. 27–28). Règles et points non tranchés :
`games/pairs/RULES.md`. Aucune statistique affichée.

### Règles retenues

- **Donne** (à chaque manche, paquet complet remélangé) : une carte visible
  par joueur, qui ouvre sa main visible ; la plus basse commence. Égalité :
  des cartes de départage aux ex æquo, comme dans Pairs, puis **défaussées**
  (contrairement à `open_round`, qui les garde : étendre `deck.rs` d'un
  départage qui défausse plutôt que recopier). Puis quatre cartes cachées
  chacun, que seul leur propriétaire voit.
- **Tour** : le joueur pose une carte cachée face cachée et annonce un rang
  de 1 à 10, vrai ou faux. En partant de sa gauche, chacun à son tour dit
  « d'accord », « pas d'accord » ou passe ; le premier qui ne passe pas fait
  révéler la carte. S'il a vu juste, il choisit qui la reçoit dans sa main
  visible (lui ou le poseur) ; sinon le poseur choisit (lui ou le
  répondant). Si tout le monde passe, la carte est révélée et le poseur la
  place dans la main visible de **n'importe quel** joueur.
- **Paires** : deux cartes de même rang dans une main visible sont
  défaussées aussitôt, face visible.
- **Fin de manche** quand toutes les cartes cachées sont jouées (quatre
  tours chacun). Classement aux points de cartes (somme des rangs) : le
  premier marque autant de points de partie que de joueurs, puis un de
  moins par place ; les ex æquo prennent la meilleure valeur et la place
  suivante est sautée (décision de l'utilisateur : 5-5-3-2-1 à 5 joueurs).
- **Fin de partie** : dès qu'un joueur a 11 points de partie ou plus en fin
  de manche, le plus haut total gagne ; égalité en tête au-dessus de 11, on
  rejoue une manche entière (avec tous les joueurs).

### Choix du site

- ❓ Le premier joueur de chaque manche se détermine à nouveau par la plus
  basse carte visible (la règle ne prévoit pas de rotation).
- ❓ Garde-fou : après 30 manches, la partie s'arrête et les égalités en tête
  restent des égalités.
- Information cachée : les cartes cachées des autres, la carte posée tant
  qu'elle n'est pas révélée, le paquet. Les mains visibles, la défausse,
  les annonces et les réponses sont publiques.

### IA

Politique de simulation, sans regarder la carte posée : annoncer le vrai
rang quand la carte arrange (ne fait pas paire chez soi ou fait paire chez
le meneur), sinon un rang plausible ; contredire une annonce d'autant plus
souvent que les copies non vues du rang annoncé sont rares ; placer une
carte révélée là où elle rapporte le plus (chez soi si elle ne fait pas
paire, sinon chez l'adversaire le mieux placé à qui elle fait paire). Risque
connu (Captain's Orders, §16) : l'ISMCTS à observateur unique récompense mal
le bluff ; on mesure l'arène et on note le résultat, sans chercher plus loin
dans ce plan. Déterminisation : la carte posée, redistribuée
pour les autres, correspond à son annonce une fois sur deux quand une copie
cachée le permet. La recherche ne délibère, par carte, que la vérité et un
bluff (le rang aux copies non vues les plus nombreuses). Mesuré : Difficile
contre Facile 0,84 à 2 joueurs, 0,82 à 4 (200 parties, ~2 min 30 chacune).

## 28. The Judge (plan 20)

Jeu de prise de risque du livret (`pairs-the-judge`, 2–6 joueurs), règle de
base seulement (décision de l'utilisateur : pas de variante « The Judge
Variant », pas d'option). Source : *Pairs Companion Book*, chapitre « The
Judge » (p. 29–30). Règles et points non tranchés : `games/pairs/RULES.md`.
Aucune statistique affichée.

### Règles retenues

- **Donne** (à chaque manche, paquet complet remélangé) : 7 cartes par
  joueur et une pile cachée de 8 cartes, le Juge.
- **Cuisine** : à son tour, le joueur ajoute une carte de sa main face
  visible à sa recette ou passe ; qui passe a fini pour la manche. Une
  recette ne contient jamais de paire : une carte qui en ferait une n'est
  pas jouable. L'étape finit quand tout le monde a passé.
- **Concours** : on retourne le Juge carte par carte ; toute recette encore
  en lice qui contient ce rang est éliminée. On s'arrête quand le Juge est
  vide ou qu'il ne reste qu'une recette en lice. Les survivantes marquent la
  somme de leurs rangs. Si une carte élimine toutes les recettes restantes,
  personne ne marque.
- **Premier joueur** : tiré au hasard à la première manche ; ensuite celui
  qui a le plus marqué à la manche précédente ; si personne n'a marqué, le
  même joueur.
- **Fin de partie** : cible 50 points à 2–3 joueurs, 40 à 4, 30 à 5–6. Dès
  qu'un joueur l'atteint en fin de manche, le plus haut total gagne.

### Choix du site

- ❓ Égalité de meilleur score de manche : le premier des ex æquo dans
  l'ordre du tour à partir de l'ancien premier joueur commence.
- ❓ Égalité en tête au-dessus de la cible : on rejoue une manche entière,
  comme dans Horn of Plenty.
- ❓ Un joueur dont aucune carte n'est jouable passe d'office (le moteur
  l'enregistre, sans clic).
- ❓ Une recette vide n'entre pas au concours : elle ne marque rien et ne
  compte pas pour « il ne reste qu'une recette ».
- ❓ Garde-fou : après 40 manches, la partie s'arrête au classement courant.
- Information cachée : les mains, le Juge (jusqu'à ses révélations), les
  cartes non distribuées. Les recettes, qui a passé et les cartes du Juge
  retournées sont publiques.

### IA

Politique de simulation : ajouter une carte tant que le risque estimé
qu'une des cartes de la recette soit chez le Juge (à partir des copies non
vues) reste sous un seuil qui monte quand on est en retard au score, et
préférer les rangs dont les copies non vues sont rares ; sinon passer.
Réalisé plus simplement : ajouter la carte qui augmente le plus l'espérance
de la recette (ses points × la chance, hypergéométrique, que le Juge n'en
tienne aucun rang), sinon passer ; un coup sur dix au hasard. Mesuré :
Difficile contre Facile 0,75 à 2 joueurs comme à 4 (200 parties).

## 29. Bread Basket (plan 21)

Jeu de sandwiches de James Ernest (`pairs-bread-basket`, 2–6 joueurs).
Version 3.11 (Crab Fragment Labs, 29/10/2025), qui remplace la fiche
mensuelle *Lunch Rush* (Cheapass 2018). Décision de l'utilisateur :
appliquer la version 2025 en entier, avec sa nouvelle distribution des
cartes, et garder le nom donné par l'auteur. Source hors dépôt :
`~/Téléchargements/Bread+Basket311.pdf`. Règles et points non tranchés :
`games/pairs/RULES.md`. Aucune statistique affichée.

### Matériel

C'est le premier jeu de la crate qui n'utilise pas le deck Pairs. Son
paquet compte 55 cartes : cinq exemplaires de chaque rang de 1 à 10, plus
cinq **Oignons** qui valent −5. L'Oignon est la carte la plus basse. Il
faut un type de carte propre au jeu (Oignon ou rang) plutôt qu'un `Rank`
élargi, parce que `Rank` reste borné à 1–10 pour tous les autres jeux. Le
mélange, la pioche et la défausse viennent de `deck.rs`, qu'on rend
générique sur le type de carte si besoin, au lieu de les recopier.

### Règles retenues

- **Donne** (à chaque manche, avec le paquet complet remélangé) : 10, 9, 8,
  7 ou 6 cartes par joueur pour 2, 3, 4, 5 ou 6 joueurs. Une carte de la
  pioche ouvre la **file** au centre. Le joueur à gauche du donneur
  commence.
- **Tour** : on pose une carte de sa main au bout de la file, ou on pioche
  une carte. La pioche est toujours permise.
- **Sandwich** : la carte posée en complète un quand on trouve, en
  remontant la file, une carte égale avant toute carte supérieure ou égale
  (toutes les cartes entre les deux sont plus basses, et il y en a au moins
  une). Le sandwich est ramassé d'office. Les deux pains vont à la défausse
  et valent 0 ; la garniture reste face visible devant le joueur et compte
  sa valeur, −5 pour un Oignon. Comme les sandwiches sont ramassés
  aussitôt, la carte égale trouvée est unique.
- **Sandwich d'air** : la carte posée est égale à la précédente (deux
  Oignons compris). Les deux cartes vont à la défausse et le joueur
  **doit** piocher une carte.
- **File vide** : la pioche y remet une carte.
- **Fin de main** :
  - Pioche vide : la main s'arrête tout de suite, que ce soit en piochant,
    à la pioche forcée d'un sandwich d'air ou en relançant la file.
  - Main vide : quand un joueur n'a plus de cartes, chacun des autres joue
    encore un tour, puis la main s'arrête.
- **Score de la main** : la garniture ramassée, moins la valeur des cartes
  qui restent en main (un Oignon en main compte −5, c'est-à-dire qu'il ôte
  5 points). Les scores peuvent devenir négatifs.
- **Partie** : la donne passe à gauche à chaque manche. Option `rounds` :
  `six` (6 manches, par défaut) ou `per-player` (une donne par joueur). Le
  plus haut total gagne.

### Choix du site

- ❓ Le premier donneur est tiré au hasard.
- ❓ À la fin d'un sandwich d'air, on pioche avant de relancer la file. Si
  la pioche est vide à ce moment-là, la main s'arrête sans pioche.
- ❓ Pendant le dernier tour de table qui suit une main vide, un autre
  joueur peut vider sa main à son tour. Cela ne rallonge rien, et la main
  s'arrête quand même après le dernier de ce tour de table.
- ❓ Égalité en tête à la fin de la partie : victoire partagée.
- La permission de « défausser les cartes qui s'annulent » est purement
  pratique sur table. Elle est ignorée : le site fait le compte.
- Information cachée : les mains des autres (seul leur nombre de cartes est
  visible) et la pioche. La file, la défausse, la garniture ramassée par
  chacun et les scores sont publics.

### IA

Politique de simulation :
- compléter le meilleur sandwich possible ;
- sinon, poser une carte qui n'offre pas de bon sandwich au joueur
  suivant, en se débarrassant d'abord des Oignons et des cartes hautes
  isolées ;
- piocher quand toute pose ouvre un sandwich coûteux ;
- un coup sur dix au hasard.

Déterminisation : redistribuer les mains cachées et la pioche parmi les
cartes non vues. Réalisé : chaque pose est notée par la garniture gagnée,
plus la pénalité qu'elle ôte de la main, moins la meilleure garniture que
la file laissée offre au suivant ; piocher coûte la pénalité moyenne d'une
carte. Mesuré : Difficile contre Facile 0,995 à 2 joueurs, 0,931 à 4
(200 parties).

## 30. The Cave (plan 22)

Fiche mensuelle Cheapass 2018 n° 4 (`pairs-the-cave`, 2–6 joueurs, deck
Pairs). Source hors dépôt : `~/Téléchargements/pairs/4CAGTheCave.pdf`.
Règles et points non tranchés : `games/pairs/RULES.md`. Aucune statistique
affichée.

### Règles retenues

- **Donne** (à chaque manche, paquet complet remélangé) : 7, 6, 5, 5 ou 4
  cartes pour 2 à 6 joueurs, puis 5 cartes face visible : la **grotte**. Le
  joueur à gauche du donneur commence ; on joue dans le sens horaire.
- **Tour** : on pose une carte de sa main. S'il y a dans la grotte des
  cartes plus basses, elle capture toutes les cartes du rang le plus proche
  en dessous ; sinon, toutes les cartes du rang le plus haut (même s'il est
  égal au sien). La carte posée entre ensuite dans la grotte, que l'on
  complète à 5 cartes depuis la pioche ; pioche vide : la grotte n'est plus
  complétée et le jeu continue.
- **Fin de manche** quand toutes les mains sont vides. Pour chaque rang, le
  ou les joueurs qui en ont capturé le plus marquent 1 point par carte de ce
  rang capturée ; les autres ne marquent rien pour ce rang.
- **Partie** : une manche par joueur, la donne passe à gauche ; le plus
  haut total gagne.

### Choix du site

- ❓ Le premier donneur est tiré au sort.
- ❓ Grotte vide : la carte posée y entre sans rien capturer. Cas
  théorique, la carte posée restant toujours dans la grotte.
- ❓ Égalité en tête : victoire partagée.
- Information cachée : les mains et la pioche. La grotte, les captures de
  chacun et les scores sont publics.

### IA

Politique de simulation : poser la carte qui capture le plus de points
probables (les cartes d'un rang où l'on mène ou peut mener la majorité),
en évitant de laisser une grotte qui offre beaucoup au suivant ; un coup
sur dix au hasard. Déterminisation : mains des autres et pioche
redistribuées parmi les cartes non vues. Réalisé : chaque pose est notée
par le gain de points de majorité, moins la moitié de la plus grosse
capture que la grotte laissée offre au suivant. Mesuré : Difficile contre
Facile 0,818 à 2 joueurs, 0,692 à 4 (200 parties).

## 31. Pickup (plan 22)

Fiche mensuelle Cheapass 2018 n° 11 (`pairs-pickup`, 2–6 joueurs, deck
Pairs). Source hors dépôt : `~/Téléchargements/pairs/11CAGPickup.pdf`.
Aucune statistique affichée.

### Règles retenues

- **Donne** : 7, 6 ou 5 cartes pour 2, 3–4 ou 5–6 joueurs.
- **Tour** : 5 cartes de la pioche forment la **réserve**, face visible.
  Chacun choisit en secret une carte de sa main ; on révèle. Les cartes
  agissent de la plus basse à la plus haute ; à égalité, le joueur le plus
  proche à gauche de celui qui a agi en dernier (au premier coup d'un tour,
  à gauche du dernier à avoir agi au tour précédent ; au tout premier, à
  gauche du donneur).
- **Agir** : le joueur **choisit** une carte de la réserve de rang
  inférieur ou égal à la sienne et la met dans sa pile de score ; la carte
  jouée est défaussée. Toutes les cartes de ce rang dans les piles des
  **autres** joueurs et dans la réserve sont défaussées (« knock out »).
  Rien de prenable (ou réserve vide) : la carte jouée va dans sa propre
  pile (« stacking »), sans rien éliminer.
- **Fin du tour** : la réserve restante est défaussée et une nouvelle est
  donnée. La partie s'arrête quand les mains sont vides ; chaque carte de
  sa pile vaut sa valeur.
- **Option `length`** (décision de l'utilisateur) : `OneDeal` (une donne,
  par défaut) ou `Target` (on redonne jusqu'à ce qu'un joueur atteigne
  150, 100 ou 75 points à 2, 3–4 ou 5–6 joueurs ; la donne passe à gauche).

### Choix du site

- ❓ Le choix de la carte prise est un coup du joueur, une fois les cartes
  révélées et à son rang d'action. Quand une seule carte est prenable (ou
  plusieurs du même rang), le moteur la prend sans demander.
- ❓ Pioche trop courte pour 5 cartes : la réserve prend ce qui reste.
- ❓ En partie longue, la manche se termine au bout de sa donne ; la partie
  s'arrête à la fin de la donne où quelqu'un atteint la cible, le plus haut
  total gagne (égalité : victoire partagée). Garde-fou : 30 donnes. Le
  départage d'une nouvelle donne part du dernier à avoir agi ; seul le
  premier coup de la partie part de la gauche du donneur.
- Information cachée : les mains, les cartes choisies avant la révélation
  (on ne sait pas non plus qui a déjà choisi), la pioche.

### IA

Choix secret : la carte qui rapporte le plus en estimant ce que les plus
basses prendront avant elle ; prise : la plus forte carte prenable, en
comptant les cartes qu'elle élimine chez les autres. Déterminisation :
les cartes choisies par les autres et non révélées sont oubliées, mains et
pioche redistribuées. Réalisé : chaque carte est notée par sa meilleure prise (sa
valeur plus celle des cartes qu'elle élimine chez les autres), ou sa propre
valeur si elle s'empile, moins 0,6 × sa valeur (la dépenser coûte) ; la
prise suit la même note. Mesuré : Difficile contre Facile 0,667 à 2
joueurs, 0,633 à 4 (200 parties ; 0,700 et 0,555 avec un coût de 0,3).

## 32. Market Day (plan 22)

Fiche mensuelle Cheapass 2018 n° 6 (`pairs-market-day`, 2–8 joueurs, deck
Pairs). Source hors dépôt : `~/Téléchargements/pairs/6CAGMarketDay.pdf`.
Aucune statistique affichée.

### Règles retenues

- **Donne** : 6 cartes chacun ; à 2–3 joueurs, 10 cartes et chacun pose
  deux cartes par tour (deux tours de table).
- **Tour** : chacun pose une carte face visible, à tour de rôle. Premier
  tour de la main : le joueur à gauche du donneur commence ; ensuite celui
  qui a le plus marqué au tour précédent (égalité : le même meneur).
- **Valeur** : chaque carte vaut son rang. Rang posé plusieurs fois : les
  premiers sont gâchés, seul le dernier marque, sauf exceptions :
  - **1 Bricoleur** : en fin de tour, son joueur peut l'échanger avec une
    autre carte.
  - **2 Chevalier** : avant le compte, tue la carte qui marque le plus ; il
    ne tue pas les rangs 3 et moins. Un second Chevalier ne gâche pas le
    premier, chacun tue une carte.
  - **3 Royal** : jamais échangé, tué ni gâché ; il marque toujours.
  - **7 Pluie** : si une seule Pluie est posée, elle gâche tout sauf les
    Royaux (1 et 2 compris) et marque elle-même ; plusieurs Pluies sont des
    7 ordinaires.
  - **9 Bandit** : c'est le premier Bandit qui marque, les suivants sont
    gâchés.
- **Ordre de résolution** : retirer les cartes gâchées, puis Royaux, Pluie,
  Chevaliers, Bricoleur.
- **Partie** : la main finit quand toutes les cartes sont posées ; la partie
  compte max(6, nombre de joueurs) mains ; le plus haut total gagne.

### Choix du site

- ❓ Bricoleur (lecture validée par l'utilisateur) : son joueur peut
  échanger son 1 avec une autre carte qui marque encore, sauf un Royal ;
  chacun marque alors la carte reçue. C'est un coup du joueur, refus
  compris (« garder »). Un Bricoleur gâché (doublon, Pluie) n'agit pas.
- ❓ Chevaliers : chacun tue, l'un après l'autre, la carte de plus haut rang
  parmi celles qui marquent encore, rang 4 au moins ; s'il n'y en a pas, il
  ne tue rien.
- ❓ À 2–3 joueurs, le meneur du tour joue sa première carte, puis tout le
  tour de table, puis sa seconde, etc. ; « le plus marqué » compte les deux
  cartes du joueur.
- ❓ Premier donneur tiré au sort, puis la donne passe à gauche (la fiche
  ne le dit pas) ; égalité en tête : victoire partagée.
- Information cachée : les mains et les cartes non distribuées.

### IA

Politique de simulation : jouer la carte qui marque le plus compte tenu des
cartes déjà posées dans le tour (doublons, Pluie seule, Chevaliers), en
gardant les Royaux et la Pluie pour les tours où ils rapportent ;
Bricoleur : échanger contre la plus forte carte permise. Déterminisation :
mains des autres et cartes non distribuées redistribuées. Réalisé : chaque
coup (pose ou échange) est noté par ses points dans le tour tel qu'il est,
moins la moitié de ceux des autres. Mesuré : Difficile contre Facile 0,935
à 2 joueurs, 0,718 à 4 (200 parties).
