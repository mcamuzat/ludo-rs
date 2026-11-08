// Short reminder shown at the table. The full rules and their source:
// games/tigre-euphrate/RULES.md (keep both in step).

export const TIGRE_EUPHRATE_RULES = [
  "But : des points dans quatre couleurs (noir, rouge, bleu, vert). Seule ta couleur la plus faible compte ; chaque trésor est un joker.",
  "Deux actions par tour, au choix : poser ou déplacer un chef, poser une tuile, poser une catastrophe, changer des tuiles.",
  "Un chef se pose à côté d'un temple (tuile rouge), hors des rivières, sans réunir deux royaumes. Un royaume est un groupe de tuiles et de chefs qui contient au moins un chef.",
  "Une tuile posée dans un royaume rapporte un point de sa couleur au chef de cette couleur, sinon au roi (chef noir). Les tuiles bleues vont sur les rivières, les autres jamais.",
  "Conflit interne : un chef posé là où un chef de sa couleur se trouve déjà. On compte les temples voisins de chaque chef, puis l'attaquant, puis le défenseur ajoutent des temples de leur main. Égalité : le défenseur gagne. Le vainqueur marque 1 point rouge.",
  "Conflit externe : une tuile réunit deux royaumes où se trouvent deux chefs de même couleur. Chacun compte les tuiles de cette couleur de son ancien royaume et peut en ajouter. Le perdant retire son chef et ces tuiles ; le vainqueur marque 1 point par chef et par tuile retirés.",
  "Renforts (variante anglaise, par défaut) : l'attaquant n'en ajoute que pour passer devant, le défenseur exactement de quoi égaliser, ou rien.",
  "Quatre tuiles d'une couleur en carré : on peut bâtir un monument de cette couleur. À la fin de ton tour, chacun de tes chefs dans le royaume d'un monument de sa couleur marque 1 point (le roi seulement pour le noir).",
  "Un royaume qui contient deux trésors ou plus : le propriétaire du marchand (chef vert) les prend tous sauf un, ceux des coins d'abord.",
  "Fin : quand il ne reste qu'un ou deux trésors, ou quand le sac ne suffit plus à refaire une main de six tuiles. Les points des autres restent cachés jusque-là.",
];
