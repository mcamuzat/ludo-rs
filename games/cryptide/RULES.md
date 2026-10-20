# Cryptide — règles

Jeu de déduction de Hal Duncan et Ruth Veevers (Osprey Games 2018, édition
française Origames), 3 à 5 joueurs.

Sources :
- **[R]** livret de règles Origames (page citée entre crochets) ;
- **[J]** le code React de l'utilisateur, `src/logic/Cryptid.js` : sections,
  indices, générateur d'énigmes ;
- **[B]** Cryptid-Bot de Christopher Lehman (github.com/CJ1067/Cryptid-Bot) :
  sections (`gameboard.py`), voisinage (`dcounter.py`), méthode de l'IA
  (`runner.py`) ;
- **[U]** décisions de l'utilisateur (2026-10-20).

Les sources restent hors du dépôt. Un point non tranché par les sources est
marqué ❓.

## Hors périmètre
Règle à deux joueurs [R p. 14] (reportée), cartes Énigme et livrets
d'indices (remplacés par le générateur), Indications [R p. 11–12],
correction d'erreur [R p. 8] (le moteur répond lui-même, sans erreur),
feuille de déduction [U].

## Carte
- 6 sections de 6 × 3 cases, deux par bande, trois bandes : 12 colonnes ×
  9 lignes [R p. 4–5]. Colonnes paires plus hautes d'une demi-case [J][B].
- Les sections sont placées au hasard, chacune retournée ou non (rotation de
  180°) [J]. Leurs terrains et territoires viennent de [J] ; ils concordent
  case par case avec [B], et les tables « retournées » de [J] sont bien la
  rotation des tables d'origine.
- Terrains : Lac, Montagne, Forêt, Marécage, Désert ; territoires d'ours et
  de puma [R p. 4].
- Structures : pierres dressées et cabanes abandonnées, une de chaque par
  couleur ; blanc, vert, bleu en mode normal, plus le noir en mode avancé
  [R p. 4–5]. Le jeu les place d'après ses cartes Énigme [U] ; ici, au
  hasard sur toute la carte, une par case au plus [J].

## Indices [R p. 9–10]
- dans un terrain parmi deux (10) ;
- dans, ou à 1 case d'un terrain (5) ;
- dans, ou à 1 case du territoire d'un animal (1) ;
- dans, ou jusqu'à 2 cases d'une pierre dressée, d'une cabane (2) ;
- dans, ou jusqu'à 2 cases du territoire d'un ours, d'un puma (2) ;
- dans, ou jusqu'à 3 cases d'une structure blanche, verte, bleue (3), noire
  en mode avancé (1).

« Dans, ou à N cases » comprend la case de l'élément [R p. 8]. Mode normal :
23 indices ; avancé : 24 et leurs négations, 48 [R p. 5].

## Énigme
Un indice par joueur, tous différents [R p. 8], dont l'intersection est une
seule case, l'habitat. Chaque indice est nécessaire (sans lui, les autres
laissent plus d'une case) et deux indices au plus portent sur deux terrains
[J]. [J] ne vérifiait la nécessité qu'à 4 et 5 joueurs : ici à toute table.

Générateur, dans l'esprit de [J] `generateClues` : des indices au hasard
(autant que de joueurs moins deux), puis celui qui laisse le moins de
cases (plus d'une), puis un qui n'en laisse qu'une ; 50 essais par carte,
sinon nouvelle carte. Les indices sont ensuite répartis au hasard entre les
joueurs. Défauts de [J] corrigés :
- `filterTable` et `checkSolution` appelés sans `this` à 4 et 5 joueurs ;
- premier indice tiré par `random() * max + 1` : le premier de la liste
  jamais tiré, et un indice noir possible en mode normal ;
- mode écrit `"nmal"`, si bien que le jeu normal tirait les indices avancés ;
- structures jamais sur la première ligne ni la première colonne.

## Mise en place
- Premier joueur tiré au sort [R p. 6].
- Partage initial [R p. 6] : à tour de rôle, chacun pose un cube sur une case
  qui ne peut pas être l'habitat selon son indice, jusqu'à deux cubes
  chacun. Une case à structure est permise, une case à cube ne l'est pas.

## Tour [R p. 6–7]
On interroge ou on fouille, puis la main passe à gauche.

Principes [R p. 7] : on n'agit jamais sur une case à cube ; on ne pose pas de
jeton là où l'on en a déjà un ; les jetons restent. Le moteur pose les
réponses d'après les indices.

- **Interroger** : une case et un autre joueur ; il pose un disque si
  l'habitat peut y être selon son indice, sinon un cube. N'importe quelle
  case, même une que l'on sait impossible [R p. 7], sauf là où l'interrogé
  a déjà un jeton [U].
- **Fouiller** : une case possible selon son indice, sans cube ; on y pose
  un disque, ou, si l'on en a déjà un, on en pose un sur une autre case
  possible sans jeton à soi [R p. 7] (s'il n'y en a pas, on n'en pose pas ❓).
  Les autres répondent dans l'ordre du tour à partir de la gauche ; ceux qui
  ont déjà un disque sur la case ne répondent pas et comptent pour « oui »
  [R p. 7][U]. Au premier cube, la fouille s'arrête.
- **Cube de pénalité** : si l'on a fait poser un cube à un autre joueur, on
  pose un cube sur une case libre de cube qui ne peut pas être l'habitat
  selon son indice [R p. 7]. Sans case permise, on n'en pose pas (cas
  rarissime [U], non prévu par [R]).

## Fin
Une fouille sans cube : le fouilleur gagne, les autres perdent [R p. 7]. On
montre alors tous les indices et l'habitat. Personne ne pose de cube sur
l'habitat : fouiller l'habitat reste toujours possible, et chaque tour pose
au moins un jeton : la partie finit toujours.
