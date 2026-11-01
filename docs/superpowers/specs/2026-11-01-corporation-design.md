# ludo-rs — Corporation

Date : 2026-11-01
Statut : demande sans consigne (la règle seule, « pas d'autre doc »), comme Black Hole ; décisions prises par le builder, marquées *(choix du site)* ici et ❓ dans `RULES.md`.

## 1. Objectif

Ajouter **Corporation** (Eric Solomon, nestorgames 2009), 2 à 4 joueurs. Des employés de quatre couleurs montent les étages d'un immeuble, de l'accueil à la salle du conseil. Ils ne montent que par des **promotions** : six paires de couleurs posées sur la table des directeurs, que les joueurs réarrangent. Le premier joueur dont les trois employés atteignent la salle du conseil gagne, quel que soit celui qui les y a menés.

## 2. Source

- **[C]** Règle anglaise de Néstor Romeral Andrés (nestorgames, 2009), deux pages. Copie locale de l'utilisateur, `CORPORATION_EN.pdf`, non copiée dans le dépôt.

## 3. Plateau (relevé sur les figures de [C])

- **Accueil** : 4 cases, rouge, bleu, vert, jaune.
- **Bureaux** : 8 étages, du bas vers le haut rouge, bleu, vert, jaune, rouge, bleu, vert, jaune.
- **Salle du conseil** au sommet.
- Chaque case de l'accueil mène au 1er étage ; chaque étage mène au suivant ; le 8e mène à la salle du conseil.
- Contrôle : dans l'exemple de [C], les positions A à E sont les étages 3 à 7 (vert, jaune, rouge, bleu, vert). Les deux déroulements du texte s'y retrouvent exactement ; un test les rejoue.

## 4. Règles (`corporation`, 2 à 4 joueurs)

- **Matériel** [C] : 3 employés et 3 jetons par couleur. *(Choix du site)* : le siège 0 est rouge, 1 bleu, 2 vert, 3 jaune ; à 2 ou 3 joueurs, les couleurs restantes sont neutres.
- **Mise en place** [C] : chacun à son tour pose un jeton de la couleur de son choix sur un cercle libre de la table, jusqu'à 12. ❓ Le siège 0 commence, et la partie continue dans l'ordre (12 se divise par 2, 3 et 4 : le siège 0 rejoue). Option `table` : `Draft` (règle) ou `Random` (table tirée au hasard, comme le solo de [C]).
- **Tour** [C] :
  1. échanger deux jetons de la table (facultatif) ;
  2. faire entrer un employé de n'importe quelle couleur sur une case de l'accueil, au sommet de la pile (facultatif) ;
  3. appliquer les promotions jusqu'à ce qu'aucune ne s'applique, dans l'ordre de son choix.
- **Promotion** [C] : une paire (gauche, droite) promeut l'employé au sommet d'une pile, de la couleur de gauche, quand la couleur « sous lui » est celle de droite. Il monte d'une case et se pose au sommet de la pile d'arrivée. ❓ La couleur sous lui est celle de l'employé juste dessous s'il est sur une pile, sinon celle de sa case (lecture de « either … or … (if he's in a stack) » ; les exemples de [C] sont compatibles avec les deux lectures).
- *(Choix du site)* : quand une seule promotion est possible, le moteur l'applique ; le joueur ne choisit que lorsqu'il y en a plusieurs. Une étape facultative sans choix possible (personne dehors) est sautée.
- **Fin** [C] : le premier joueur dont les trois employés sont dans la salle du conseil gagne. ❓ La partie s'arrête dès cet instant, même si des promotions restent ; l'ordre des promotions choisi par le joueur actif décide donc qui arrive « premier ». Les autres sont classés par nombre d'employés dans la salle du conseil.
- *(Choix du site)* : limite de durée, 20 tours de table sans entrée ni promotion ⇒ partie nulle.
- Le solo de [C] (minimiser le nombre de tours) n'est pas repris.

## 5. Moteur

- Crate `games/corporation` : `lib.rs` (règles, `Game`), `view.rs`, `heuristics.rs` ; `RULES.md` ; tests de règles et de conformance (2, 3, 4 joueurs, deux tables).
- Actions : `Place { circle, color }` (mise en place), `Swap { a, b }`, `Enter { color, desk }`, `Skip` (étape facultative), `Promote { from }`.
- Information complète : `determinize` est l'identité, même écran proposé.
- La vue donne les piles, la couleur de chaque case, la table, les employés dehors et dans la salle du conseil, les promotions possibles, le déroulé du tour en cours et du précédent.
- IA : l'ISMCTS générique ; `timeout_action` = passer, ou la première promotion.

## 6. Web

- `web/src/games/corporation/` : `types.ts`, `Board.tsx`, `rules.ts`, un test ; famille `board`.
- L'immeuble en SVG, étages teintés de leur couleur, piles en pions décalés ; la table des promotions à côté, numérotée 1 à 6 du bas vers le haut comme dans [C]. On échange en cliquant deux jetons, on fait entrer en choisissant une couleur puis une case de l'accueil, on promeut en cliquant une pile surlignée.

## 7. Vérification

- `ludo-cli arena corporation 200 hard easy` : au moins 0.55.
