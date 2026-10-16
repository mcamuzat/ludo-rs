# Bantu — règles implémentées

Sources :

- **[R]** Règle française de Summit Games (Leeds), mise en ligne par François
  Haffner (jeuxsoc.free.fr), septembre 2005. Jeu © 1955 Parker Brothers.
- **[B]** Le plateau, dessiné en vectoriel sur les pages 4 à 7 du même PDF.
  Le graphe en est extrait par `scripts/bantu-board.py` dans
  `src/board_data.rs` (107 cases, 4 lignes de départ, 27 rangées, 78
  liaisons orientées). Le relevé a été validé par l'utilisateur le
  2026-10-16, superposé au scan.

Un point que les sources ne tranchent pas est marqué ❓.

## Matériel [R][B]

- 2 à 4 joueurs. Chacun a 4 pions numérotés de 1 à 4, ou 3 pions en partie
  courte (option).
- Au centre, les cases de départ. Quatre **lignes de départ** de 4 cases
  partent en rayons : ligne 1 en bas, 2 à droite, 3 en haut, 4 à gauche.
- Des **rangées** : des cases reliées par un fond marron. Il y a aussi des
  cases isolées, et des liaisons bleu clair entre les cases.
- L'**arrivée**, en bas à gauche, s'atteint depuis 4 cases fléchées.

## Parcours [B]

On avance dans le sens horaire : ligne 4, quart en haut à gauche, ligne 3,
quart en haut à droite, ligne 2, quart en bas à droite, ligne 1, puis le
quart en bas à gauche jusqu'à l'arrivée. Le pion 1 sort donc tout près de
l'arrivée, et le pion 4 fait presque tout le tour.

❓ Le dessin n'oriente pas les liaisons bleu clair. Chacune va vers la case
la plus proche de la ligne suivante, ou de l'arrivée, dans son quart. Les 10
flèches bleues du plateau donnent toutes le même sens que cette règle. Une
seule liaison (37–52) a ses deux bouts à égale distance : elle est orientée
comme ses équivalentes des autres quarts.

## Déroulement [R]

- Le siège 0 commence, puis on joue dans le sens horaire.
- À son tour, on déplace **un seul** pion. Sortir un pion de sa case de
  départ, vers n'importe quelle case libre de la ligne de départ de son
  numéro, compte comme le coup entier. Un pion pris peut ressortir à tout
  moment.
- On doit jouer si on le peut. Sinon on passe son tour.

## Longueur du déplacement [R]

- Un pion seul dans sa rangée, ou sur une case isolée, avance d'autant de
  cases que son numéro.
- Si au moins deux pions sont dans la même rangée ou sur la même ligne de
  départ, quelle que soit leur couleur, chacun d'eux avance de la **somme**
  de leurs numéros. La somme se calcule au début du tour.
- ❓ « Côte à côte » se lit comme « dans la même rangée », même quand les
  cases ne se touchent pas.

## Chemin [R]

- Il faut faire exactement ce nombre de pas, sauf pour entrer dans
  l'arrivée.
- On suit les liaisons dans leur sens. Dans une rangée ou sur une ligne de
  départ, on peut glisser latéralement, dans les deux sens.
- On ne repasse jamais par une case déjà traversée pendant le même tour, et
  on ne recule jamais.
- On ne passe jamais par-dessus un pion, de quelque couleur qu'il soit.
- ❓ Quand plusieurs chemins existent, le joueur choisit sa case d'arrivée.

## Arrivée [R]

- Un pion qui atteint une case fléchée avec au moins un pas restant peut
  entrer dans l'arrivée : le compte exact n'est pas nécessaire. Un pion qui
  commence son tour sur une case fléchée peut y entrer aussi.
- ❓ Un pion qui s'arrête pile sur une case fléchée y reste, et entrera à un
  tour suivant.

## Prise [R]

- On ne peut pas terminer sur un pion de sa couleur.
- Finir exactement sur un pion adverse le renvoie sur sa case de départ, sauf
  s'il est protégé. Un pion est protégé s'il est sur une ligne de départ, ou
  s'il partage sa rangée avec au moins un autre pion, de n'importe quelle
  couleur.

## Fin

- [R] Le premier joueur qui a mis tous ses pions dans l'arrivée gagne.
- ❓ Les autres joueurs sont classés selon le nombre de pions qu'ils ont dans
  l'arrivée, puis selon le nombre de pas qui leur restent au plus court. Un
  pion encore sur sa case de départ compte un pas de plus que la case la plus
  proche de sa ligne. Les ex æquo partagent leur rang.
- **Limite de durée** : c'est un ajout de ludo-rs, validé par l'utilisateur
  le 2026-10-16, et non une règle de [R]. On peut glisser dans les deux sens
  d'une rangée, donc un pion peut faire des allers-retours sans fin. La
  partie s'arrête après **30 tours de table sans progrès** (un tour passé
  compte comme un tour joué). Il y a progrès quand un pion est pris, quand il
  entre dans l'arrivée, ou quand il atteint une case plus proche de
  l'arrivée que toutes celles où il est allé depuis sa dernière sortie
  (sortir un pion compte donc aussi). Des allers-retours ne battent jamais
  ce record.
- ❓ La partie s'arrête aussi si plus personne ne peut bouger. Ce cas semble
  impossible sur ce plateau, puisqu'un pion sur une case fléchée peut
  toujours entrer dans l'arrivée.
- Dans ces deux derniers cas, on classe tous les joueurs comme ci-dessus.
