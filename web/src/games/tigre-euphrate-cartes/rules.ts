// Short reminder shown at the table. The full rules and their sources:
// games/tigre-euphrate-cartes/RULES.md (keep both in step).

export const TIGRE_EUPHRATE_CARTES_RULES = [
  "But : des points dans quatre couleurs (noir, rouge, bleu, vert). Seule ta couleur la plus faible compte ; chaque trésor est un joker.",
  "Huit trésors en rang ; sous chacun, une colonne d'au plus 8 cartes. Un royaume est une colonne, avec celles qu'une carte de jonction lui relie.",
  "Deux actions par tour, la même deux fois si tu veux : poser ou déplacer un chef sur une carte libre, poser une carte, jouer ta catastrophe (une par partie).",
  "Une carte posée au bas d'une colonne rapporte un point de sa couleur au chef de cette couleur du royaume, sinon au roi (chef noir). Le point se marque avec une carte de cette couleur de sa main, d'office si on en a une.",
  "Entre deux trésors qui ont chacun au moins 3 cartes, une carte relie leurs royaumes (sans point). Le marchand (chef vert) du nouveau royaume y échange alors un trésor contre une carte rouge de sa main.",
  "Conflit interne : un chef posé là où un chef de sa couleur se trouve déjà. L'attaquant puis le défenseur ajoutent des cartes rouges ; +1 à qui a son roi sur une carte rouge du royaume. Égalité : le défenseur gagne et le vainqueur marque 1 point rouge.",
  "Conflit externe : une jonction réunit deux chefs de même couleur. Chacun compte les cartes de cette couleur de son ancien royaume et en ajoute de sa main. Le vainqueur marque 1 point, plus toutes les cartes de cette couleur du côté du perdant.",
  "Quatre cartes d'une couleur à la suite en bas d'une colonne : on peut bâtir un navire (bleu et cette couleur ; n'importe lequel pour quatre bleues). À la fin de ton tour, chacun de tes chefs d'une couleur du navire, dans son royaume, marque 1 point.",
  "Fin : à la fin d'un tour, quand il ne reste qu'un trésor sur la table ou que la pioche ne suffit plus à refaire les mains de 8.",
];
