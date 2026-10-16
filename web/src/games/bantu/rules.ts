// Short reminder shown at the table. The full rules and their sources:
// games/bantu/RULES.md (keep both in step).

export const BANTU_RULES = [
  "But : amener tous ses pions dans l'arrivée (« FINISH », en bas à gauche) avant les autres. Pas de dé : tout se calcule.",
  "Le pion N sort sur la ligne de départ N, puis tourne dans le sens horaire : le 1 est tout près de l'arrivée, le 4 fait presque le tour.",
  "Un pion seul dans sa rangée avance de son numéro. Plusieurs pions dans la même rangée ou sur la même ligne, toutes couleurs confondues, avancent chacun de la somme de leurs numéros.",
  "Il faut le compte exact, sans repasser par une case ni sauter un pion. On peut glisser dans une rangée. L'arrivée s'atteint depuis une case fléchée, sans compte exact.",
  "Finir sur un pion adverse le renvoie au départ, sauf s'il est sur une ligne de départ ou dans une rangée avec un autre pion.",
  "Ajout du site : la partie s'arrête après 30 tours de table sans progrès (aucun pion pris, arrivé, ni plus près de l'arrivée qu'il ne l'a jamais été) ; on classe alors par pions arrivés, puis par chemin restant.",
];
