# Black-Pot — règles implémentées

Sources :

- **[R]** Règle Cogitas (Marseille), mise en ligne par l'Escale à jeux. Pages 2
  (but, barème) et 3 (cases, déroulement, fin) du PDF.
- **[B]** Le plateau en couleur, page 4 du même PDF. Relevé dans
  `src/board.rs`, validé par l'utilisateur le 2026-10-21 à côté du scan.
- **[U]** Décisions de l'utilisateur, le 2026-10-21.

Un point que les sources ne tranchent pas est marqué ❓.

## Matériel [R][B]

- Un plateau de 109 cases : un carré de 11 × 11 aux coins coupés (rangées de
  7, 9, sept de 11, 9 et 7 cases). Chaque case porte une valeur de 1 à 24 :
  - **citron** : 1 à 6 ;
  - **cerise** : 7 à 12, 15, 16, 18, 20, 24 ;
  - **Black-Pot** : 13, 14, 17, 19, 21, 22, 23.
- 40 pions par camp. Le jeu se joue à 2 joueurs, ou à 4 en deux équipes de
  2 ; chaque équipe partage ses 40 pions.
- 3 dés.

## Un coup [R][U]

- On doit **utiliser les 3 chiffres** tirés [R]. On les répartit en un, deux
  ou trois groupes. Un chiffre seul vaut sa face. Un groupe de 2 ou 3
  chiffres vaut un calcul en + − × ÷, de n'importe quelle forme
  (6 ÷ (4 − 2) comme (6 × 4) ÷ 2).
- ❓ Chaque résultat, y compris intermédiaire, est un entier positif ou nul :
  une division doit tomber juste, et on ne divise pas par 0.
- Chaque groupe de valeur non nulle occupe une case libre de cette valeur.
  **Tous** les groupes doivent en trouver une [U]. Deux groupes occupent
  deux cases différentes.
- Un groupe qui vaut 0 n'occupe rien : c'est ainsi qu'on ne pose aucun pion
  (2 + 4 − 6 = 0) [R]. ❓ Une paire nulle (3 − 3) avec un chiffre seul pose
  donc un pion.
- On ne pose pas plus de pions qu'il n'en reste à son camp.
- Ces règles donnent d'elles-mêmes celles des cases de [R]. Une cerise
  demande 2 ou 3 chiffres. Un Black-Pot demande les 3, car aucune valeur
  Black-Pot n'est la somme, la différence, le produit ou le quotient de deux
  dés.
- Le calcul est annoncé [R] : le site l'affiche à tous, enchaîné comme dans
  la règle (« 6 × 4 = 24 ÷ 2 = 12 »).

## Obligation de jouer [R][U]

- Sur table, un joueur qui ne trouve rien passe, sauf si l'adversaire lui
  montre un coup, ce qui l'oblige à jouer. En ligne [U], on ne passe pas
  volontairement, et le tour d'un joueur sans coup passe tout seul.
- Le coup à 0 pion est un coup comme un autre : on peut le choisir pour
  économiser ses pions [R].
- **« Je ne trouve pas »** [U] : un joueur qui a un coup mais ne le voit pas
  passe sans poser de pion, et le site ne lui montre pas ce qu'il a manqué.
  L'IA trouve toujours ses coups et ne l'emploie pas. Le chrono, quand il
  s'épuise, joue « Je ne trouve pas ».

## Ordre de jeu [R][U]

- À 2 joueurs [U], chacun lance les dés à son tour et joue son tirage.
- À 4 joueurs [R], les équipes alternent autour de la table (sièges 0 et 2
  contre 1 et 3). Le lanceur jette les dés. Les trois suivants jouent ce
  tirage dans l'ordre, puis le lanceur le joue en dernier. Au tour suivant,
  le siège suivant lance.
- Le premier lanceur est tiré au hasard (sur table, le plus grand dé
  commence). Les dés se lancent tout seuls au début de chaque tirage.

## Alignements [R][U]

- Un alignement, c'est 5 pions d'un même camp sur 5 cases consécutives
  d'une ligne, d'une colonne ou d'une diagonale. Dès qu'il est complet, ses
  pions sont retournés et son rapport marqué [R].
- Les croisements sont permis [R]. Un pion compte au plus **une fois par
  direction** [U]. Six pions en ligne ne font donc qu'un alignement dans
  cette direction.
- ❓ Quand un coup crée, dans une direction, une suite de plus de 5 pions qui
  n'y comptent pas encore, on y retient les alignements sans chevauchement
  qui rapportent **le plus** au total. À égalité, ce sont les plus tôt dans
  la suite. Une suite de 10 peut donc ne donner qu'un alignement, s'il
  rapporte plus que deux.

## Barème [R]

| 5 cases alignées | points |
|---|---|
| dont 4 Black-Pot | 400 |
| dont 3 Black-Pot | 250 |
| dont 2 Black-Pot | 100 |
| 5 cerises | 100 |
| 5 citrons | 50 |
| autres | 30 |

Aucun alignement de 5 Black-Pot n'existe sur ce plateau.

## Fin [R]

- La partie s'arrête dès qu'un camp pose son dernier pion, même au milieu
  d'un tirage à 4.
- Chaque camp compte ses alignements, plus 10 points par pion non joué. Le
  plus haut score gagne. Une égalité est partagée, et les équipiers ont le
  même rang.
- **Limite de durée**, ajout du site : la partie s'arrête aussi après 10
  tours de table de suite sans pion posé. Un coup à 0 pion, « Je ne trouve
  pas » et un tour passé ne posent rien. On compte alors de même.
