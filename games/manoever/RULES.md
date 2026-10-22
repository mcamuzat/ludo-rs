# Das glorreiche Manöver — règles implémentées

Sources :

- **[R]** Traduction française de François Haffner pour jeuxsoc.fr, mise en
  ligne par l'Escale à jeux (« Les Glorieuses Manœuvres »). Jeu © 1992
  Roland Siegers, Schmidt Spiel + Freizeit.
- **[B]** Le plateau, pages 5 et 6 du même PDF. Les numéros des cases et des
  places de la caserne sont du texte positionné ; `scripts/manoever-board.py`
  en relève les coordonnées dans `src/board_data.rs`. Relevé validé par
  l'utilisateur le 2026-10-22, superposé au scan.
- **[K]** Critique allemande de Claudia Schlee et Andreas Keirat
  (Spielphase), qui paraphrase la règle Schmidt :
  <http://ftp.informatik.rwth-aachen.de/keirat/txt/D/Dasglorr.html>.
- **[U]** Décisions de l'utilisateur, le 2026-10-22.

Seul le jeu de base est implémenté [U] : les numéros des soldats ne comptent
pas. Un point que les sources ne tranchent pas est marqué ❓.

## Matériel [R][B]

- Un parcours de 84 cases, de la caserne à la caserne. Les cases 1 à 6 et
  79 à 84 sont blanches, les autres roses.
- La caserne : 12 places, I à XII.
- 12 soldats en tout : 6 chacun à 2 joueurs (rouge, jaune), 4 à 3 joueurs
  (+ bleu), 3 à 4 joueurs (+ vert). Le siège 0 est rouge, puis jaune, bleu,
  vert.
- Pièces bonus aux cases suggérées par [R], choisies par [U] (la règle
  d'origine les laisse placer librement, d'après [K]) :
  - noires (×3) sur 12, 20, 47 et 65 ;
  - blanches (×2) sur 8, 24, 31, 36, 60 et 74 ;
  - grises sur 25 (1 point), 40 (2) et 55 (3).

## Mise en place [R][K]

À tour de rôle, à partir du siège 0, chaque joueur lance le dé et sort un de
ses soldats sur la case 1 à 6 indiquée. Sur une case qui porte déjà 3
soldats, on relance. ❓ Le soldat sorti est celui de plus petit numéro encore
à la caserne. Le dé ne sert plus ensuite.

## Un tour [R][K]

- Le siège 0 ouvre la marche, puis on suit l'ordre des sièges.
- Un soldat est **libre** s'il est seul ou en haut de sa pile.
- Le joueur déplace chacun de ses soldats libres **une fois**, dans l'ordre
  qu'il veut. Son tour finit quand il n'en reste plus à déplacer.
- Un soldat libéré pendant le tour (son camarade du dessus a bougé) bouge
  aussi.
- Un soldat recouvert pendant le tour par un camarade attend le tour
  suivant [K] (note du traducteur dans [R]).
- Un joueur sans soldat libre passe son tour [K].
- ❓ Un soldat libre sans destination reste sur place (n'arrive pas, voir
  « Fin »).

## Déplacement [R][U]

Au moins 1 case, au plus **position × hauteur × bonus**, compté au moment
où le soldat bouge (❓, accepté par [U]) :

- **position** : 1 + le nombre de cases occupées par des soldats devant lui.
  Les cases vides, les pièces bonus et les soldats rentrés ne comptent pas
  [K] ;
- **hauteur** : son étage dans la pile ; une pièce noire ou blanche compte
  comme un étage [R] ;
- **bonus** : ×2 posé directement sur une pièce blanche, ×3 directement sur
  une noire [R][U]. Un soldat posé sur un autre soldat n'a pas le bonus.

Exemple [R] : en 4e position sur une pièce noire (étage 2),
4 × 2 × 3 = 24. [K] compte la hauteur en soldats et donne le bonus à toute
la case ; l'utilisateur a retenu [R].

## Atterrir [R][K]

- On peut se poser sur une pile, de n'importe quelle couleur.
- Jamais plus de 3 étages, pièce noire ou blanche comprise : une pièce
  bonus porte au plus 2 soldats. On peut dépasser une pile pleine [K].
- Se poser sur une pièce grise la gagne ; elle quitte le plateau et ne
  compte pas comme un étage.

## Arrivée et fin [R]

- ❓ Un soldat dont le maximum dépasse la case 84 peut rentrer à la caserne
  sans compte exact (accepté par [U]).
- Il prend la plus haute place libre : XII (12 points), puis XI…
- Quand le 11e soldat rentre, le dernier est posé aussitôt sur la place I
  [R] et la partie finit ; il ne prend pas de pièce grise au passage.
- Score : places des soldats + pièces grises. Le plus grand total gagne ;
  ❓ les égalités partagent la victoire (accepté par [U]).
- La partie finit toujours : un soldat en position p a un maximum d'au
  moins p et au plus p − 1 cases occupées devant lui, donc une case libre à
  portée ; chaque soldat recouvert finit libéré.
