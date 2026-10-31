# ludo-rs — Cobras

Date : 2026-10-31
Statut : décisions déléguées par l'utilisateur (« fais tout automatiquement »)

## 1. Objectif

Ajouter **Cobras** (Chris & Suzanne Zinsli, Cardboard Edison, 2017), un jeu
de plis de 2 à 5 joueurs. Les perdants d'un pli se partagent les cobras
dessinés sur les cartes jouées et les mettent dans leur panier. Le gagnant
rend son panier contre des points, selon un barème qui culmine à 7 cobras
(21 points). Le Cobra Royal permet, une fois par manche, de se joindre aux
gagnants ou aux perdants d'un pli.

L'utilisateur avait programmé le **mode solo** en JS
(`~/prog/magic/spiral/cobras.html`). Il veut ici la **règle de 2 à 5
joueurs** [U] : le solo n'est pas repris.

Les décisions qui ne viennent pas de la règle sont marquées *(choix du
site)* ici et ❓ dans `RULES.md`.

### Hors périmètre

- Le mode solo (1 joueur et deux fictifs).
- Les illustrations : une carte montre sa couleur, sa valeur et 1 à 3
  cobras.

## 2. Sources

- **[R]** *Cobras Rules v3* (Cardboard Edison, 2017), pages 1 à 9. Copie
  locale : `~/prog/magic/spiral/Cobras Rules v3.pdf`. Le print-and-play
  (`printnplay-low.pdf`, même dossier) confirme le matériel. Aucun des deux
  n'est copié dans le dépôt.
- **[U]** L'utilisateur, le 2026-10-31.

## 3. Règles (`cards-cobras`, 2 à 5 joueurs)

- **Matériel** [R] : 52 cartes, de 1 à 13 dans quatre couleurs (rouge,
  vert, bleu, jaune). Une carte de 1 à 4 montre 1 cobra, de 5 à 9 en montre
  2, de 10 à 13 en montre 3 (les « triples »). Chaque joueur a un Cobra
  Royal et un panier. ❓ Les jetons de cobra sont illimités : la boîte en
  compte 42, mais [R] ne prévoit pas qu'ils manquent.
- **Barème du panier** [R] : 1 → 1, 2 → 3, 3 → 5, 4 → 8, 5 → 11, 6 → 15,
  7 → 21, 8 → 15, 9 → 8, 10 et plus → 3. Un panier vide ne rapporte rien.
- **Donne** [R] : on mélange les 52 cartes et on les distribue également.
  Une carte reste écartée face cachée à 3 joueurs, deux à 5. À 2 joueurs, on
  donne comme à 3 : la troisième pile est celle du **fictif**.
- **Amorcer le panier** [R] : avant le premier pli, chacun choisit en
  secret une carte de sa main. On les révèle ensemble, et chacun met dans
  son panier les cobras de sa carte. Les triples révélés rejoignent la
  ligne des triples. *(Choix du site, comme Pickup)* : pendant le choix,
  personne ne sait qui a déjà choisi.
- **Premier joueur** [R] : tiré au hasard à la première manche (d'après la
  graine) ; ensuite, le joueur qui a **le moins de points**. ❓ Entre
  ex-aequo, le premier dans le sens horaire après celui qui a commencé la
  manche précédente. À 2 joueurs, le fictif ne commence jamais une manche.
- **Pli** [R] : le premier joueur pose une carte de son choix, puis les
  autres jouent dans le sens horaire. Il faut fournir la couleur demandée
  si on le peut ; sinon on joue n'importe quelle carte. La plus haute carte
  de la couleur demandée gagne le pli, et son joueur commence le suivant.
- **Gagnant** [R] : il rend **tous** les cobras de son panier et marque les
  points du barème.
- **Perdants** [R] : les cobras des cartes du pli, plus ceux restés au
  centre, sont partagés le plus également possible entre les joueurs qui
  n'ont pas gagné le pli. Le reste de la division reste au centre pour le
  pli suivant.
- **Cobra Royal** [R] : au lieu d'une carte, à son tour, une fois par
  manche. On annonce aussitôt si l'on **partage la victoire** (on rend son
  panier comme le gagnant) ou **la défaite** (on reçoit sa part avec les
  perdants). Le Cobra Royal ne compte aucun cobra. S'il ouvre le pli, c'est
  la carte suivante qui fixe la couleur demandée. Il est permis même quand
  on pourrait fournir.
- **Sans gagnant** [R] : si personne ne gagne le pli avec une carte (tous
  ont joué leur Cobra Royal), celui qui a ouvert le pli ouvre le suivant.
  ❓ Ceux qui ont annoncé la victoire rendent quand même leur panier : [R]
  dit que « tous les joueurs peuvent gagner un pli », ce qui n'arrive que
  de cette façon. Si personne n'a perdu, tous les cobras restent au centre.
- **Fin de manche** [R] : dès que la ligne compte 10 triples après un pli.
  Chacun perd 1 point par cobra resté dans son panier ; le centre est vidé,
  les Cobras Royaux reviennent à leurs joueurs. ❓ La manche s'arrête aussi
  si une main est vide au début d'un pli (un joueur qui a joué son Cobra
  Royal garde une carte de plus que les autres : sans cette règle, les
  autres n'auraient plus rien à poser).
- **Fin de partie** [R] : après une manche où un joueur a au moins
  **100 points** (**75** à 5 joueurs). Le plus haut total gagne ; les
  ex-aequo partagent le rang. Pas de limite de durée *(choix du site)*.

### À 2 joueurs [R]

- Le fictif est assis à gauche du donneur. ❓ Le siège 0 donne la première
  manche, puis la donne alterne. Sa pile est mélangée ; il joue la carte du
  dessus à son tour.
- Option **Pile du fictif** : cachée (« haute surprise ») ou retournée après
  l'amorce des paniers (« haute stratégie » : la carte du dessus est
  visible). *(Choix du site)* : retournée par défaut.
- Quand le fictif suit, sa carte est considérée de la couleur demandée,
  quelle que soit sa couleur. À valeur égale avec une vraie carte de la
  couleur, la carte du fictif est plus basse. Quand il ouvre le pli, sa
  carte fixe la couleur demandée.
- Le fictif reçoit sa part des cobras (elle est perdue), peut gagner un pli
  (sans marquer) et ouvre alors le suivant. Il n'amorce pas de panier et n'a
  pas de Cobra Royal.

## 4. Moteur

- `games/cards/src/cobras/` sur le modèle de David & Goliath : `mod.rs`,
  `view.rs`, `determinize.rs`, `heuristics.rs`, `testing.rs` ; paquet
  propre (`Suit`, `Card { suit, value }`), `deck::deal_evenly`.
- Actions : `Seed { card }`, `Play { card }`, `King { share: Win | Loss }`.
- Information publique : cartes jouées et révélées, paniers, centre, ligne
  des triples, Cobras Royaux utilisés et leur annonce, couleurs qu'un joueur
  n'a pas fournies. Cachées : les mains, la carte écartée, la pile du
  fictif (sauf sa carte du dessus si elle est retournée), les cartes
  choisies pour l'amorce avant la révélation.
- `determinize` redistribue les mains des autres, les cartes écartées et la
  pile cachée du fictif, en respectant les couleurs non fournies, et oublie
  les choix d'amorce des autres.
- IA : l'ISMCTS générique. La simulation s'arrête quand la manche est
  comptée, avec le classement aux points.
- `timeout_action` : la plus petite carte permise (jamais le Cobra Royal).

## 5. Web

- `web/src/games/cards/cobras/` : `types.ts`, `Board.tsx`, `rules.ts`, un
  test ; famille `modern` [U].
- Chaque siège montre son total, son panier (nombre de cobras) et si son
  Cobra Royal est encore en main. Le centre montre la ligne des triples
  (x / 10), les cobras au centre, le pli en cours ou le dernier pli avec son
  résultat. La main propose les cartes permises et, s'il est disponible, le
  Cobra Royal avec ses deux annonces.
- À 2 joueurs, le fictif a sa place à table (pile, carte du dessus si elle
  est retournée).
- Le barème du panier est rappelé à l'écran, comme sur la carte Panier.

## 6. Vérification

- Tests de règles : partage et reste, fictif, Cobra Royal (victoire,
  défaite, ouverture, pli sans gagnant), fin de manche et de partie,
  premier joueur.
- Conformance à 2 (les deux options), 3, 4 et 5 joueurs.
- `ludo-cli arena cards-cobras 200 hard easy` : au moins 0.55, mesuré à 2 et
  4 joueurs.
