# ludo-rs — Black Hole

Date : 2026-11-01
Statut : demande sans consigne (la règle seule, envoyée après Cobras) ; décisions prises par le builder, marquées *(choix du site)* ici et ❓ dans `RULES.md`.

## 1. Objectif

Ajouter **Black Hole** (Wal Joris), un jeu de placement pour 2 joueurs. Chacun pose à tour de rôle ses tuiles numérotées de 1 à 10 sur un plateau triangulaire de 21 cases. À la fin, une seule case reste vide : le trou noir. Le joueur dont les tuiles voisines du trou totalisent le moins gagne.

## 2. Source

- **[B]** Règle anglaise de Néstor Romeral Andrés (nestorgames, 2013), une page. Copie locale : `~/Téléchargements/BLACKHOLE_EN.pdf`, non copiée dans le dépôt.

## 3. Règles (`black-hole`, 2 joueurs)

- **Matériel** [B] : plateau triangulaire de 21 cases (6 rangées, de 1 à 6 cases). 10 tuiles rouges et 10 bleues, de 1 à 10.
- **Voisinage** : les cases se touchent comme des billes empilées en triangle. Une case a jusqu'à 6 voisines : deux sur sa rangée, deux au-dessus, deux au-dessous.
- **Tour** [B] : Rouge commence. Chacun pose à son tour une tuile de sa couleur, face visible, sur une case vide. *(Choix du site)* : le siège 0 est Rouge.
- **Ordre des tuiles** [B] : croissant (le 1, puis le 2, …).
- **Fin** [B] : quand les 20 tuiles sont posées, la case vide est le trou noir. Chacun additionne ses tuiles voisines du trou ; le plus petit total gagne. ❓ À égalité, la partie est nulle.
- **Option `order`** :
  - `Ascending` (défaut) : la règle de base ;
  - `Free` (variante 1 de [B]) : on pose ses tuiles dans l'ordre de son choix ;
  - `Random` (variante 2 de [B]) : chacun tire ses tuiles au hasard dans sa réserve cachée. *(Choix du site)* : la tuile tirée est montrée aux deux joueurs pendant le tour où elle se pose ; l'ordre du reste n'est connu de personne.

## 4. Moteur

- Crate `games/black-hole` : `lib.rs` (règles, `Game`), `board.rs` (cases et voisinage), `view.rs`, `heuristics.rs` ; `RULES.md` ; tests de règles et de conformance pour les trois options.
- Action : `Place { cell, tile }`.
- La vue montre le plateau, les tuiles restantes de chacun (triées), la tuile à poser (sauf en ordre libre) et, à la fin, le trou noir et les deux totaux.
- `determinize` mélange l'ordre des réserves en ordre aléatoire, sauf la tuile déjà tirée.
- IA : l'ISMCTS générique ; `timeout_action` = la première case libre.

## 5. Web

- `web/src/games/black-hole/` : `types.ts`, `Board.tsx`, `rules.ts`, un test ; famille `board`.
- Plateau SVG en triangle de disques, comme sur la règle. Les tuiles rouges et bleues portent leur valeur ; 6 et 9 sont soulignés. En ordre libre, on choisit d'abord la tuile, puis la case. À la fin, le trou noir est marqué et ses voisines ressortent, avec les deux totaux.

## 6. Vérification

- `ludo-cli arena black-hole 200 hard easy` : au moins 0.55.
