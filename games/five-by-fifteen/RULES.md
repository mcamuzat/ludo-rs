# 5×15

Jeu solo de Friedemann Friese (2F-Spiele, 2020), inspiré de la patience
« Montana ».

Sources :
- [R] Règle anglaise de 2F-Spiele, `2f-spiele_rules_5x15_US.pdf` (hors dépôt).
- [U] Décisions de l'utilisateur (2026-11-22), d'après sa version « Shinkansen
  Express » : les tuiles sont des wagons, les lignes des voies.

## Matériel et mise en place [R]

75 tuiles : 5 couleurs (Émeraude, Cramoisi, Azur, Ambre, Violet [U]), chacune
numérotée de 1 à 15. On les mélange et on les pose face visible sur 5 lignes de
15 colonnes. On retire les cinq « 1 », on les remélange et on les pose en
colonne zéro, devant le plateau : chacun donne la couleur de sa ligne. Cela
laisse cinq trous.

## But [R]

Trier chaque ligne de sa couleur, de 1 à 15, de gauche à droite.

## Actions [R]

- **a) Poser** dans un trou une tuile qui lui va : de la même couleur que sa
  voisine, et exactement 1 de plus que la tuile à gauche du trou ou 1 de moins
  que la tuile à droite. À gauche de la première colonne se trouve le « 1 » de
  la ligne : seul le 2 de cette couleur y va.
- **b) Pousser** d'une case vers la droite, dans le trou juste après le 15,
  une suite de tuiles adjacentes, consécutives et de même couleur qui finit par
  le 15. Toute la suite avance. Longueur minimale selon le parcours :
  - Ligne courte (niveau 1) : 3 tuiles ou plus ;
  - Ligne moyenne (niveau 2) : 4 ou plus ;
  - Ligne longue (niveau 3) : 5 ou plus.

Rien ne sort à droite de la quinzième colonne. Une ligne entièrement triée n'a
plus de trou : sa dernière colonne ne se joue plus (on ne pousse pas une ligne
finie).

Les tuiles 13, 14 et 15 attelées forment la locomotive [U] ; l'écran montre
quand elle peut avancer (action b possible).

## Fin [R]

- Gagné : les cinq lignes sont triées.
- Perdu : plus aucune action possible.
- [R] dit aussi « perdu si l'on tourne en boucle ». [U] Le moteur signale une
  position déjà vue (avertissement) sans arrêter la partie ; le joueur peut
  abandonner à tout moment (perdu).
