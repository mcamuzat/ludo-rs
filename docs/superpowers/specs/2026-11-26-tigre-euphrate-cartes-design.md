# Tigre et Euphrate, le jeu de cartes : design

## Intention

Ajouter à ludo-rs le jeu de cartes *Euphrat & Tigris, le combat des rois* (Reiner
Knizia, 2005), 2 à 4 joueurs, humains ou IA. Il est distinct du jeu de plateau déjà
présent (`games/tigre-euphrate`) : huit trésors en rang, des colonnes de cartes
sous chacun, des royaumes qu'on relie en posant une carte entre deux trésors.

## Sources

- **[R]** : traduction française non officielle de Cylmoryl pour boiteajeux.net,
  octobre 2005 (`euphc_rg.pdf`, hors dépôt). Référence.
- **[L]** : résumé anglais de Chris Conboy (« Liumas »), mars 2009 (hors dépôt).
  Sert à départager les passages ambigus de [R] ; quand les deux s'opposent, [R]
  l'emporte, et c'est marqué ❓.

## Décisions de l'utilisateur (2026-11-26)

- **Marchand** : à la jonction, il prend **un seul** trésor ([R]), pas « tous sauf
  un » ([L]).
- **Points d'office** : [R] laisse le propriétaire d'un chef refuser de poser la
  carte de sa main ; le site la pose d'office dès qu'il en a une. Pas d'attente
  hors tour.
- **Piles de points visibles** : la variante de [R] (quatre colonnes visibles de
  tous). En ligne, de toute façon, chaque carte arrive face visible et se retrouve
  dans l'historique.

## Non-objectifs

- Le mode même écran : les mains sont secrètes.
- Une crate commune avec le jeu de plateau : les règles se ressemblent, mais pas
  la topologie ni le matériel. On recopie `final_points` (dix lignes).
- Retenir qu'un joueur n'avait pas la couleur d'un point qu'il n'a pas marqué
  (déterminisation contrainte) : à reprendre si l'IA en a besoin.

## Règles (à reporter dans `RULES.md`, sourcées)

- **Matériel** : 185 cartes civilisation (40 noires peuple, 40 vertes marché,
  65 rouges temple, 40 bleues agriculture), 8 trésors (rouges), 4 catastrophes
  (une par joueur), 3 navires bicolores : bleu-noir, bleu-rouge, bleu-vert ([R]
  illustrations). Quatre chefs par joueur : roi noir, marchand vert, prêtre rouge,
  agriculteur bleu.
- **Mise en place** : 8 trésors en rang, une place libre entre deux (7 jonctions),
  une colonne d'au plus 8 cartes sous chacun. Mains de 8. À 2 joueurs, 30 cartes
  retirées au hasard, face cachée. Premier joueur au hasard.
- **Royaume** : les cartes reliées par des côtés : une colonne avec son trésor*,
  plus les colonnes voisines reliées par une carte de jonction. Jamais deux chefs
  de même couleur dans un royaume, hors conflit.
- **Tour** : deux actions, la même deux fois si on veut ; on peut s'arrêter avant
  (❓ [R] « peut réaliser deux actions »).
  1. **Chef** : poser un chef de sa réserve ou en déplacer un, sur une carte libre
     (trésor* et jonctions compris, jamais un navire). Pas de retour à la réserve
     ([L]). Chef de même couleur dans le royaume : conflit interne.
  2. **Carte** : au bas d'une colonne (8 cartes au plus sous le trésor*, navires
     compris ❓), ou dans une jonction vide dont les deux colonnes ont au moins 3
     cartes (navires compris ❓).
     - En colonne : 1 point de la couleur au propriétaire du chef de cette couleur
       dans le royaume, sinon au roi ; point marqué d'office si la carte est en
       main (décision ci-dessus). Puis, si les 4 dernières cartes de la colonne sont
       de la même couleur ([L] : suite consécutive) et qu'un navire convient
       (n'importe lequel pour 4 bleues, celui de la couleur sinon), le joueur peut
       le bâtir : les 4 cartes sont défaussées, leurs chefs rentrent, le navire
       prend leur place.
     - En jonction : face cachée pendant les conflits externes (elle ne compte pour
       personne), retournée ensuite ; aucun point. Puis le marchand du nouveau
       royaume, s'il y a au moins deux trésors (cartes trésor d'origine), en
       échange un contre une carte rouge de la main de son propriétaire, d'office
       si possible ; le trésor va sur sa pile, la carte rouge prend sa place (et
       devient un trésor*). Lequel : le plus à gauche ❓.
  3. **Catastrophe** (une par partie) : retirer une carte, jamais un trésor*, un
     navire ni une carte occupée par un chef. Les cartes du dessous remontent.
- **Conflit interne** : renforts rouges, l'attaquant (le joueur actif) puis le
  défenseur, autant qu'ils veulent. +1 à un participant dont le roi est sur une
  carte rouge (temple ou trésor*) du royaume ([R], exemple A ; [L] dit « son chef
  », ❓). Égalité au défenseur. Le perdant reprend son chef ; le vainqueur marque
  un point rouge avec une des cartes jouées, sinon d'office de sa main ; les
  autres cartes jouées vont à la défausse.
- **Conflit externe** : chaque couleur où deux chefs se retrouvent. Plusieurs
  conflits : le joueur actif choisit l'ordre. Attaquant : le joueur actif s'il est
  concerné, sinon le premier concerné après lui en sens horaire. Base : les
  cartes de la couleur de son côté (son ancien royaume, la jonction exclue),
  trésors* compris pour le rouge. Renforts libres de la couleur. Le perdant
  reprend son chef ; le vainqueur marque une carte jouée (sinon de sa main), puis
  prend toutes les cartes de la couleur du côté du perdant (sauf trésors*,
  navires, cartes occupées) sur sa pile ; les cartes remontent. Un royaume qui se
  scinde annule les conflits dont les chefs sont séparés.
- **Fin du tour** : navires d'abord : pour chaque navire, le joueur actif marque 1
  point par chef à lui d'une des deux couleurs du navire dans son royaume (d'office
  si la carte est en main ; le roi n'y est pas joker). Puis chacun complète sa main
  à 8, le joueur actif d'abord.
- **Fin de partie** : à la fin d'un tour, s'il n'y a plus assez de cartes pour
  compléter toutes les mains, ou s'il ne reste qu'une carte trésor sur la table.
  Ajout du site : aussi après 20 tours de table sans pioche.
- **Score** : la couleur la plus faible, trésors en jokers ; égalité départagée
  par la suivante, etc.

## Moteur (`games/tigre-euphrate-cartes`)

- `id` : `tigre-euphrate-cartes`, nom « Tigre et Euphrate : le combat des rois »,
  2–4 joueurs, sans option, `hidden_info`, horloge (coup par défaut : finir le tour,
  ne rien ajouter, ne pas bâtir, premier conflit).
- `State` : têtes de colonne (trésor ou trésor échangé), 7 jonctions, 8 colonnes de
  cases `{carte, chef}`, jonction face cachée en cours, mains en comptes par
  couleur, pioche `Vec<Color>`, cartes retirées à 2, réserves de chefs,
  catastrophes, piles de points `[u8; 4]` + trésors, navires restants, phase,
  actions restantes, événements du tour et du tour précédent.
- `Action` : `Leader { color, to }`, `Extend { color, column }`,
  `Join { color, gap }`, `Catastrophe { at }`, `EndTurn`, `Commit { count }`,
  `Resolve { color }`, `Ship { ship }`, `NoShip`. Une position est
  `Head(i) | Gap(i) | Card(colonne, rang)`.
- `View` : la table, ma main, tailles des mains, réserves, catastrophes, piles,
  navires restants, pioche, phase (conflit avec bases et renforts), événements.
- `Determinize` : mains adverses, pioche et cartes retirées redistribuées entre
  elles, tailles gardées.

## IA

ISMCTS générique. `rollout_action` : une politique simple qui favorise les
points marqués et les renforts utiles ; `evaluate` coupe après quelques tours de
table sur la couleur la plus faible provisoire. Mesure :
`cargo run --release -p ludo-cli -- arena tigre-euphrate-cartes 200 hard easy`,
seuil ≥ 0.55, à 2 et 4 joueurs.

## Web (`web/src/games/tigre-euphrate-cartes/`)

`types.ts`, `Board.tsx` (rang de trésors, jonctions, colonnes ; ma main ; piles en
colonnes), `rules.ts`, un test, une entrée `family: "cards"` dans
`registry.ts`. Couleurs et noms des chefs repris du jeu de plateau.
