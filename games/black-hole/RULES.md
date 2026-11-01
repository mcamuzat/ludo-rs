# Black Hole — règles implémentées

Source :

- **[B]** Wal Joris, *Black Hole*, règle anglaise de Néstor Romeral Andrés
  (nestorgames, 2013), une page (copie de l'utilisateur,
  `BLACKHOLE_EN.pdf`, hors dépôt), consultée le 2026-11-01.

Spec : `docs/superpowers/specs/2026-11-01-black-hole-design.md`. Un point que
la source ne tranche pas est marqué ❓.

## Matériel

- [B] Plateau triangulaire de 21 cases : 6 rangées de 1 à 6 cases.
- [B] 10 tuiles rouges et 10 bleues, de 1 à 10 ; 6 et 9 sont soulignés.
- Une case touche jusqu'à 6 cases : deux sur sa rangée, deux dans la rangée
  du dessus, deux dans celle du dessous (billes empilées en triangle, comme
  sur la figure de [B]).

## Partie

- [B] Rouge commence ; le siège 0 est Rouge. On pose à tour de rôle une
  tuile de sa couleur, face visible, sur une case vide.
- [B] Les tuiles se posent dans l'ordre croissant : le 1, puis le 2, etc.
- [B] Quand les 20 tuiles sont posées, la case vide est le trou noir.
  Chacun additionne ses tuiles voisines du trou ; le plus petit total
  gagne. ❓ À égalité, la partie est nulle.

## Option `order`

- `Ascending` (défaut) : la règle ci-dessus.
- `Free`, variante 1 de [B] : les tuiles se posent dans n'importe quel
  ordre.
- `Random`, variante 2 de [B] : chacun tire ses tuiles au hasard dans sa
  réserve face cachée. ❓ La tuile tirée est visible des deux joueurs pendant
  le tour où elle se pose ; l'ordre du reste n'est connu de personne.
