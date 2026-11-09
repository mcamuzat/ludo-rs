// Short reminder shown at the table. The full rules and their source:
// games/kurote/RULES.md (keep both in step).

export const KUROTE_RULES = [
  "Chacun a 7 sorciers : 5 sur le plateau, 2 en réserve. Vert commence.",
  "À son tour, on téléporte un de ses sorciers non épuisés sur une case vide voisine (même en diagonale) d'un autre de ses sorciers.",
  "Interdit d'arriver sur une case entourée de 3 sorciers adverses ou plus.",
  "Un sorcier qui a bougé est épuisé (couché). Quand tous les sorciers d'un joueur sont épuisés, ils se relèvent tous.",
  "Un sorcier entouré de 3 sorciers adverses ou plus, épuisés ou non, est capturé.",
  "Avec moins de 5 sorciers en jeu, on peut à la place poser un sorcier de réserve sur sa première rangée ; il arrive épuisé.",
  "Le premier qui capture 3 sorciers gagne. Nulle après 80 tours sans capture.",
];
