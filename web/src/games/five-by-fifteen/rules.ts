// Short reminder shown at the table. The full rules and their source:
// games/five-by-fifteen/RULES.md (keep both in step).

export const FIVE_BY_FIFTEEN_RULES = [
  "75 wagons, 5 lignes de 1 à 15. Le « 1 » de chaque voie, à gauche, donne la couleur de sa ligne ; il ne bouge pas.",
  "But : former sur chaque voie le train de sa couleur, de 1 à 15, de gauche à droite.",
  "Poser : un wagon va dans un trou s'il suit son prédécesseur (1 de plus que le wagon à gauche) ou précède son successeur (1 de moins que le wagon à droite), de la même couleur.",
  "Avancer : une suite de wagons consécutifs de même couleur menée par le 15 avance d'une case dans le trou devant elle, si elle est assez longue (3, 4 ou 5 wagons selon le parcours).",
  "Les wagons 13, 14 et 15 attelés forment la locomotive : elle fume, et s'illumine quand le train peut avancer.",
  "Rien ne sort à droite. Perdu si plus aucun coup n'est possible ; on peut aussi abandonner.",
];
