# Wu Hsing — règles implémentées

Source :

- **[R]** Sid Sackson, *Wu Hsing – Domino Bead Game*, dans *A Gamut of Games*
  (Dover) ; adaptation française de François Haffner, janvier 2004, pages 2–3
  (copie de l'utilisateur, `wuhsi_rg.pdf`, hors dépôt).

Un point que la source ne tranche pas est marqué ❓. Seule la règle de base est
implémentée (pas la « variante stratégique » ni les mains visibles).

## Matériel

- [R] 30 dominos : les 15 dominos sans 0 ni 6 (valeurs 1 à 5), chacun en
  double. Chaque valeur est une couleur : 1 Bois (vert), 2 Feu (rouge),
  3 Terre (ocre), 4 Métal (gris), 5 Eau (bleu).
- [R] Grille réelle ou imaginaire, carreaux de la taille d'un demi-domino : le
  plateau n'a pas de bord.

## Préparation

- [R] À 2 joueurs, 14 dominos chacun ; à 3, 9 chacun et un des 3 restants
  écarté face cachée ; à 4, 7 chacun.
- [R] Les 2 dominos restants sont retournés et placés côte à côte en carré au
  centre ; ils n'appartiennent à personne. Ici, deux dominos horizontaux l'un
  sous l'autre.
- [R] Deux doubles de la même couleur au départ : maldonne, on redistribue.
- [R] Le premier joueur est tiré au hasard, puis on joue dans le sens horaire.

## Pose

- [R] Un groupe est une suite d'au moins 2 demi-dominos contigus dans une
  rangée ou une colonne. Il forme un cycle de 1 à 5 couleurs : a,a,a… ;
  a,b,a,b… ; a,b,c,a,b,c… ; a,b,c,d,a… ; a,b,c,d,e,a…. Dès qu'il se répète ou
  contient les cinq couleurs, son modèle est fixé. Une rangée peut contenir
  plusieurs groupes séparés par des vides.
- [R] Règle 1 : chaque domino doit prolonger correctement **deux groupes ou
  plus**. Les deux demi-dominos d'un domino ne forment pas à eux seuls un
  groupe prolongé : sa propre ligne ne compte que si elle touche un domino déjà
  posé dans son axe (exemples A, C, E de [R] : deux groupes ; B, D : trois ;
  X, Z illégaux). Toute ligne touchée doit rester un cycle valide.
- [R] Règle 2 : interdit de placer un quatrième domino dans l'alignement de
  trois dominos qui se touchent déjà par leur grand côté (exemple Y). Comme
  l'adaptation JS : on compte les dominos de même orientation exactement
  alignés.
- [R] Un joueur qui pense ne pas pouvoir jouer expose sa main ; les autres
  peuvent désigner un domino jouable, qu'il doit alors jouer. Ici le moteur
  tranche : on ne passe que si aucune pose n'est possible. Passer rapporte 0.

## Score et fin

- [R] À chaque pose, on marque le produit des nouvelles longueurs des groupes
  prolongés, doublé si le domino est un double (exemple B : 4 × 5 × 4 = 80,
  160 avec un double).
- [R] La partie s'arrête quand un joueur a posé tous ses dominos : les joueurs
  qui ont commencé après lui continuent de sorte que chacun ait eu le même
  nombre de tours. Elle s'arrête donc quand le tour revient au premier joueur.
  (L'ancienne version JS rendait la main au finisseur, ce qui donnait un tour
  de trop à certains joueurs quand le finisseur n'était pas le premier.)
- [R] Elle s'arrête aussi quand tous les joueurs ont passé successivement.
- [R] Chacun retire de son score 5 points par double et 25 par autre domino
  encore en main. Le plus haut score l'emporte. ❓ Égalité : rang partagé.
