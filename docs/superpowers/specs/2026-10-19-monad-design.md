# ludo-rs — Monad (spec 6)

Date : 2026-10-19
Statut : validée
Précédent : `2026-10-16-pennywise-design.md`

## 1. Objectif

Ajouter **Monad** (Sid Sackson, 3M 1970), 2 à 4 joueurs, jouable contre l'IA,
sur le même écran ou en ligne, comme les autres jeux du site. On part des
Unités pour monter, par échanges, achats et accaparements, jusqu'aux Monades ;
le premier qui en réunit le nombre requis gagne.

### Hors périmètre

Règle en équipe (2 contre 2, première équipe à 5 Monades) : reportée, à la
demande de l'utilisateur. Animations et sons.

## 2. Sources

- **[M]** Règle officielle 3M (1970), en anglais (`Monad_Rules.pdf`), et sa
  réédition Gryphon Games (`Monad_Rulesbis.pdf`, même texte, « purple » au lieu
  de « navy »). Règle de référence.
- **[F]** Traduction française de Paul Lequesne (7 décembre 2008, `1mill_rg.pdf`,
  mise en ligne par l'Escale à jeux). Donne le vocabulaire du site : Unité, Bi,
  Tri, Quad, Quint, Monade, Échanger, Acheter, Accaparer, Piocher, Retourner,
  Passer, Joker, carte Identité, carte Bonus. Ses planches à imprimer donnent le
  dessin des cartes.
- **[T]** *The First Million*, réécriture de Nathan Morse (`The_First_Million_game_summary.pdf`),
  la même mécanique sur un autre thème. Ne sert qu'à trancher un point flou de [M].
- **[U]** Précisions de l'utilisateur (2026-10-19) : [M] fait référence, [T]
  tranche seulement, l'équipe peut attendre.

Les PDF restent hors du dépôt. `games/monad/RULES.md` cite [M], [F], [T] ou [U]
pour chaque règle et marque ❓ ce qui n'est pas tranché. Écarts relevés entre
les textes : [F] retire 3 Bonus à 2 joueurs, [M] 2 (on suit [M] : 4 Bonus,
2 joueurs) ; [T] fait commencer un joueur désigné par couleur, [M] le voisin du
donneur (on tire au hasard).

## 3. Matériel

- **Couleurs** [M] : chaudes rouge, orange, jaune ; froides bleu marine, bleu
  ciel, vert. Combinaisons Bonus : rouge/marine, orange/ciel, jaune/vert.
  L'ordre de référence des couleurs (affichage, tri) est rouge, orange, jaune,
  marine, ciel, vert.
- **Rangs et valeurs** [M] : Unité 1, Bi 3, Tri 7, Quad 16, Quint 36, Monade 80.
- **Cartes** [M] : 24 Unités (4 par couleur) ; 6 Bi, 6 Tri, 6 Quad, 6 Quint (une
  par couleur) ; 10 Monades (identiques, sans couleur) ; 6 cartes Identité (une
  par couleur) ; cartes Bonus (une par joueur, toutes identiques : le site n'en
  garde qu'un indicateur « bonus pris ce tour »).
- **Mise en place** [M] selon le nombre de joueurs :
  - 2 joueurs : 12 Unités (2 par couleur) ; 2 Identités tirées parmi les 6.
  - 3 joueurs : 18 Unités (3 par couleur) ; Identités marine, ciel, vert.
  - 4 joueurs : 24 Unités ; Identités rouge, orange, marine, ciel.
  Les Identités sont distribuées au hasard, une par joueur, face visible. Les
  colonnes Bi, Tri, Quad et Quint sont mélangées chacune ; toutes leurs cartes
  sont visibles, la carte du haut est celle qu'on prend. Toutes les Unités sont
  distribuées, 6 par joueur ; ni pioche ni défausse au départ.
- **Premier joueur** : tiré de la graine ([M] : voisin du donneur ; le site n'a
  pas de donneur).

## 4. Règles

Un tour est **soit** une suite de transactions (échanges, achats,
accaparements, autant qu'on veut) terminée par « Finir le tour », **soit** une
seule action Piocher, Retourner ou Passer, qui fait tout le tour [M].

- **Échanger** [M] : deux cartes de la main d'un même rang, l'une chaude,
  l'autre froide, contre la carte du haut du rang suivant (deux Quint donnent
  une Monade). Impossible si ce rang est vide sur la table. Les Unités cédées
  vont sur la défausse, face visible ; les autres cartes vont au bas de leur
  colonne.
- **Bonus** [M] : si les deux cartes échangées forment une combinaison Bonus,
  sont d'un rang au moins Bi, sans Joker, et qu'aucun bonus n'a été pris ce
  tour, le joueur peut prendre le haut de chaque rang inférieur aux cartes
  échangées, dont une Unité du haut de la pioche (jamais de la défausse). Un
  rang vide ne donne rien et le reste du bonus est quand même pris ([M] : « you
  lose that part of your bonus » ; [T] arrête le bonus au premier rang vide, on
  suit [M]). Le choix se fait **juste après l'échange** : prendre ou renoncer ;
  renoncer garde le droit à un bonus plus tard dans le tour. Si aucune part du
  bonus n'est disponible, le site ne propose rien (mon choix).
- **Joker** [M] : les Bi, Tri, Quad et Quint de la couleur de sa carte Identité
  peuvent remplacer, dans un échange, une carte d'un autre rang (les Unités ne
  sont jamais Jokers). Un seul Joker par échange ; la règle chaude/froide
  s'applique avec la couleur réelle du Joker ; un échange avec Joker ne donne
  jamais de bonus. Un échange avec Joker joue au rang de l'autre carte, Unité
  comprise (un Joker Quint et une Unité donnent un Bi : permis, quoique peu
  utile). Deux cartes de même rang dont l'une est de la couleur d'Identité
  forment un échange ordinaire (pas de Joker utilisé, bonus possible).
- **Acheter** [M] : la carte du haut d'une colonne (Bi à Quint), ou une Monade,
  contre deux cartes ou plus de la main, toutes de rang strictement inférieur à
  la carte achetée, dont la valeur totale atteint la sienne. Pas de monnaie
  rendue. **Mon choix** : le site n'accepte que les achats **sans carte
  superflue** (retirer n'importe quelle carte ferait passer sous la valeur).
  Payer en plus n'apporte rien et multiplierait les coups possibles.
- **Accaparer** [M] : 4, 5 ou 6 Unités de couleurs toutes différentes contre le
  haut des Tri, des Quad ou des Quint. [M][U] : « you must **first** "prepare"
  by making at least one other play (trading or buying) in that turn » —
  l'accaparement n'est permis qu'après au moins un échange ou un achat fait
  dans le même tour. ([T] impose l'inverse, un investissement ou un achat
  après ; l'utilisateur a choisi [M] à la lettre le 2026-10-19.)
- **Ordre de rangement** (mon choix) : quand plusieurs cartes vont au bas d'une
  colonne ou sur la défausse, elles y vont dans l'ordre de référence des
  couleurs. [M] ne dit rien, [T] laisse choisir ; le choix n'a pas d'effet sur
  le jeu, sauf sur l'ordre de la future pioche.
- **Piocher** [M] : s'il y a une pioche, prendre la carte du haut. Tout le tour.
- **Retourner** [M] : s'il n'y a pas de pioche mais une défausse, la retourner
  telle quelle (sans mélange) pour en faire la pioche : la première Unité
  défaussée devient la première piochée. Tout le tour.
- **Passer** [M] : permis seulement si le joueur ne peut ni échanger, ni
  piocher, ni retourner, ni acheter une carte ou une Monade avec deux cartes du
  rang immédiatement inférieur et assez d'autres cartes pour atteindre la
  valeur. Passer est alors permis même si un autre achat ou un accaparement
  serait possible. Le site vérifie la condition (il n'y a pas de main à montrer).
- **Fin** [M] : le premier à réunir 5 Monades à 2 joueurs, 4 à 3, 3 à 4 gagne,
  aussitôt, même au milieu de son tour. Les 10 Monades suffisent toujours (à
  4 joueurs, 4 × 2 = 8 < 10 ; à 3, 3 × 3 = 9 < 10).
- **Classement** (mon choix) : le vainqueur premier, les autres par nombre de
  Monades, égalités en rangs égaux.
- **Limite de durée** (ajout du site, comme pour Bantu) : la partie s'arrête si
  un tour de table complet n'est fait que de « Passer » (rien ne peut plus
  changer), ou après 40 tours de table sans qu'aucune Monade soit prise. Le
  classement se fait alors par Monades seulement, sans vainqueur désigné.

## 5. Information cachée

- **Public** : les colonnes entières, le stock de Monades, la défausse entière
  (posée face visible devant tout le monde), le nombre de cartes de la pioche,
  les cartes Identité, les Monades de chacun, l'indicateur de bonus.
- **Mains** : chacun voit la sienne. Des autres, la vue donne **le nombre de
  cartes par rang** (mon choix) : chaque Bi et plus est pris ou rendu à la vue
  de tous, et chaque Unité aussi se compte. Les couleurs restent cachées.
- **Pioche** : son ordre est caché (le site ne compte pas sur la mémoire des
  joueurs, même si elle est faite de la défausse retournée).
- **Dernier coup** : la vue décrit la transaction précédente (qui, quoi, pour
  quoi) ; une Unité piochée ou prise en bonus n'y est montrée qu'à son preneur.
- **Determinize** : pour chaque rang, les cartes que l'observateur ne voit pas
  (mains des autres, et pour les Unités la pioche) sont redistribuées au hasard
  en gardant les nombres par rang et par main et la taille de la pioche. Les
  tirages utilisent `ChaCha8Rng` et des `u32`, comme ailleurs.

## 6. Moteur (`games/monad`, crate à part)

Le jeu suit « Ajouter un jeu » du `CLAUDE.md`. Identifiant `monad`, nom
affiché « Monad », `hidden_info: true`, `clock: true`, aucune option de lobby.

- Types : `Color` (6 variantes), `Rank` (Unité à Quint), `Card { rank, color }`.
  Une main est un multiensemble de cartes ; les colonnes sont des files ; la
  défausse et la pioche des piles d'Unités.
- Actions : `Trade { a, b }`, `Buy { target, cards }` (`target` : un rang ou la
  Monade), `Leap { colors }`, `TakeBonus`, `DeclineBonus`, `EndTurn`, `Draw`,
  `Flip`, `Pass`. Les cartes d'une action sont données dans l'ordre de
  référence, pour qu'un coup ait une seule écriture.
- `legal_actions` énumère : les échanges distincts (Joker compris), les achats
  sans carte superflue, les accaparements, et selon la phase du tour
  `EndTurn` / `Draw` / `Flip` / `Pass`. Après un échange qui ouvre un bonus,
  seuls `TakeBonus` et `DeclineBonus` sont légaux. `apply` vérifie que l'action
  est dans `legal_actions`, sans toucher à l'état sinon.
- `timeout_action` : `DeclineBonus` pendant un choix de bonus ; sinon
  `EndTurn` si permis ; sinon `Draw`, `Flip` ou `Pass` ; sinon la première
  action légale.
- La `View` porte tout ce que le plateau affiche ; le web ne calcule rien
  (sauf apparier une sélection de cartes avec une action de `legalActions`).

## 7. IA

L'ISMCTS générique. `search_actions` peut écarter des coups absurdes (un Joker
Quint pour échanger des Unités, des achats qui paient avec des cartes de haut
rang quand un paiement plus bas existe) si la mesure l'exige. `evaluate` coupe
les parties simulées sur une estimation de progression (Monades, puis valeur
des mains). Mesure : `ludo-cli arena monad 200 hard easy`, Difficile ≥ 0.55
contre Facile, à 2 et à 4 joueurs ; les valeurs vont dans « Commandes » du
`CLAUDE.md`.

## 8. Web

- `web/src/games/monad/` : `types.ts`, `Board.tsx`, `rules.ts`, un test dans
  `__tests__/`, une entrée dans `web/src/games/registry.ts`.
- Cartes au dessin de [F] : un disque de la couleur, coupé en 1 (Unité), 2 (Bi),
  3 (Tri), 4 (Quad) ou 5 (Quint) parts, la valeur au centre ; la Monade en
  yin-yang noir et blanc marqué « 80 ». Les couleurs reprennent celles des
  planches (rouge vif, orange, jaune, bleu marine, bleu ciel, vert).
- Table : pioche et défausse à gauche, les quatre colonnes au centre (cartes
  qui se chevauchent, toutes visibles), le stock de Monades à droite. Devant
  chaque joueur : sa carte Identité, sa carte Bonus (retournée quand le bonus
  a été pris ce tour), ses Monades, et pour les adversaires le nombre de cartes
  par rang.
- Jouer : on sélectionne des cartes de sa main ; les boutons « Échanger »,
  « Acheter … », « Accaparer » s'allument quand la sélection correspond à une
  action de `legalActions` (pour l'achat, un bouton par cible possible).
  « Prendre le bonus » / « Renoncer », « Finir le tour », « Piocher »,
  « Retourner la défausse », « Passer » sont des boutons simples.
- Accueil (mon choix) : une nouvelle famille `trade` et une étagère « Jeux
  d'échange » ; Monad y est seul pour l'instant.
- Rappel de règles à table, d'accord avec `RULES.md`.

## 9. Livraison

Un plan (`docs/superpowers/plans/2026-10-19-monad.md`), un commit par étape
(moteur, registre et IA, web, docs), puis une relecture par un relecteur Opus.
