# ludo-rs — David & Goliath

Date : 2026-10-30
Statut : décisions déléguées par l'utilisateur (« fais tout automatiquement »)

## 1. Objectif

Ajouter **David & Goliath** (Reinhard Staupe), un jeu de plis de 3 à 6
joueurs. À chaque pli, la carte la plus haute ramasse toutes les autres, et
la plus basse la prend. Une couleur dont on a une ou deux cartes rapporte
leur valeur ; au-delà, un point par carte. Le jeu est **introuvable** [U] :
le site sert à y rejouer, contre l'IA ou en ligne avec des IA pour compléter.

L'utilisateur a laissé toutes les décisions au builder. Celles qui ne
viennent pas des règles sont marquées *(choix du site)* ici et ❓ dans
`RULES.md`.

### Hors périmètre

- Les illustrations des cartes : elles portent seulement leur couleur et leur
  valeur.
- Les options du lobby : aucune.

## 2. Sources

- **[R]** Deux traductions anglaises de la règle, fournies par l'utilisateur.
  Copies locales : `~/Téléchargements/David&Goliath.pdf` et
  `~/Téléchargements/David_&_Goliath_Rules_(English).pdf`. Les deux disent la
  même chose et ne sont pas copiées dans le dépôt.
- **[U]** L'utilisateur, le 2026-10-30.

## 3. Règles (`cards-david-goliath`, 3 à 6 joueurs)

- **Matériel** [R] : 90 cartes, de 1 à 18 dans cinq couleurs : rouge, jaune,
  vert, bleu, violet. L'ordre est celui du décompte de [R].
- **Paquet selon les joueurs** [R] : à 3, les valeurs 1 à 9 (45 cartes) ; à
  4, de 1 à 12 ; à 5, de 1 à 15 ; à 6, tout le paquet. Cela fait toujours
  15 cartes par joueur, soit des valeurs de 1 à 3 × le nombre de joueurs.
- **Donne** [R] : le joueur à gauche du donneur commence. ❓ Le siège 0
  donne la première manche, et la donne passe à gauche à chaque manche
  *(choix du site, comme Hattrick)*.
- **Pli** [R] : le premier joueur pose la carte de son choix. Les autres
  jouent ensuite à tour de rôle, dans le sens horaire. Il faut **fournir**
  la couleur de la première carte quand on le peut. Sinon, on joue
  n'importe quelle carte.
- **Plus haute et plus basse** [R] : la plus haute carte est celle de plus
  grande valeur, quelle que soit sa couleur. Entre deux valeurs égales,
  c'est la **dernière jouée** qui l'emporte. Il en va de même pour la plus
  basse.
- **Prise** [R] : le joueur de la plus basse carte prend la plus haute
  carte. Le joueur de la plus haute carte prend toutes les autres, y
  compris la plus basse. Il commence le pli suivant.
  ❓ Si toutes les cartes ont la même valeur, la dernière jouée est à la
  fois la plus haute et la plus basse : son joueur prend tout.
- **Cartes gagnées** [R] : elles restent visibles devant leur joueur,
  rangées par couleur.
- **Fin de manche** [R] : la manche dure 15 plis, et toutes les cartes sont
  jouées.
- **Points d'une manche** [R] : on compte chaque couleur à part. Avec une ou
  deux cartes d'une couleur, on marque leur valeur. Avec trois ou plus, on
  marque 1 point par carte. Exemple de [R] : rouge 4, 10, 11 donne 3 ;
  jaune 8 donne 8 ; vert 5, 9 donne 14 ; bleu 2, 3, 6, 10, 12 donne 5 ;
  violet 10, 12 donne 22. Le total est de 52.
- **Partie** [R] : il y a autant de manches que de joueurs. On additionne
  les points, et le plus haut total gagne. Les ex æquo partagent leur rang.

## 4. Moteur : module `games/cards/src/david_goliath/`

Ce module suit la forme de `hattrick/` : `mod.rs`, `view.rs`,
`determinize.rs`, `heuristics.rs` et `testing.rs`. Il a son propre paquet
`Card { color: Color, value: u8 }`, et ses donnes viennent de
`deck::deal_evenly`, avec un flux de graine par manche, comme Hattrick.

- **Action** : `Play { card }`.
- **Information cachée** : seules les mains sont cachées. Un joueur qui n'a
  pas fourni la couleur demandée n'en a plus pour toute la manche. Le moteur
  le note, et `determinize` redistribue les mains des autres joueurs avec
  `deck::redeal_where`, en respectant ces couleurs manquantes. La graine des
  manches suivantes est aussi tirée à nouveau.
- **Vue** :
  - ma main, triée ;
  - le nombre de cartes en main de chacun ;
  - le pli en cours ;
  - le dernier pli, avec qui avait la plus haute carte et qui avait la plus
    basse ;
  - les cartes gagnées de chacun, par couleur ;
  - les points que ces cartes valent déjà, calculés par le moteur ;
  - les totaux ;
  - le numéro de la manche, le nombre de manches et le donneur ;
  - le joueur au trait ;
  - le décompte de la manche précédente ;
  - mes coups légaux.
- **Temps écoulé** : on joue la plus basse carte légale *(choix du site)*.
- **IA** : l'ISMCTS générique, et la simulation s'arrête à la fin de la
  manche, comme pour Hattrick. Si la recherche ne suffit pas, le builder peut
  améliorer la simulation.

## 5. Web : `web/src/games/cards/david-goliath/`

On y met `types.ts`, `Board.tsx`, `rules.ts` et un test dans
`web/src/games/cards/__tests__/`. On ajoute aussi une entrée
`family: "modern"` dans le registre, comme Hattrick.

- Les cartes sont de la couleur de leur famille et portent leur valeur. Les
  cinq couleurs restent lisibles dans les deux thèmes.
- Pour chaque joueur, on affiche les cartes gagnées en rangées par couleur,
  avec les points de la manche en cours et le total.
- La main et le pli en cours sont au centre. Une fois le pli fini, on voit
  qui a pris la plus haute carte et qui a pris le reste.

## 6. Tests

Tests de règles :
- le paquet selon le nombre de joueurs, et 15 cartes par main ;
- l'obligation de fournir, et le jeu libre quand on n'a pas la couleur ;
- l'exemple de [R] : vert 4, vert 9, rouge 2, vert 7 → le joueur du 2 prend
  le 9 vert, et celui du 9 prend le 4, le 2 et le 7, puis commence ;
- les égalités, où la dernière carte jouée l'emporte, en haut comme en bas ;
- toutes les valeurs égales ;
- le décompte de l'exemple de [R] (52 points) ;
- le nombre de manches et la rotation de la donne ;
- `apply` refuse un coup sans rien changer ;
- les couleurs manquantes sont respectées par `determinize`.

Ensuite la conformance de 3 à 6 joueurs, un test web, et l'arène
`ludo-cli arena cards-david-goliath 200 hard easy 4`, qui doit donner au
moins 0.55.
