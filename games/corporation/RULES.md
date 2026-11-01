# Corporation — règles implémentées

Source :

- **[C]** Eric Solomon, *Corporation*, règle anglaise de Néstor Romeral
  Andrés (nestorgames, 2009), deux pages (copie de l'utilisateur,
  `CORPORATION_EN.pdf`, hors dépôt), consultée le 2026-11-01.

Spec : `docs/superpowers/specs/2026-11-01-corporation-design.md`. Un point que
la source ne tranche pas est marqué ❓.

## Matériel

- [C] 3 employés et 3 jetons de chacune des 4 couleurs : rouge, vert, bleu,
  jaune. Les employés attendent dehors, au pied de l'immeuble.
- Plateau, relevé sur les figures de [C] :
  - l'accueil, 4 cases : rouge, bleu, vert, jaune ;
  - 8 bureaux, du bas vers le haut : rouge, bleu, vert, jaune, rouge, bleu,
    vert, jaune ;
  - la salle du conseil, au sommet.
  Les positions A à E de l'exemple de [C] sont les étages 3 à 7 (vert,
  jaune, rouge, bleu, vert) : les deux déroulements du texte s'y
  retrouvent (`tests/rules.rs`).
- [C] Chaque case de l'accueil mène au 1er étage, chaque étage au suivant,
  le 8e à la salle du conseil.
- [C] La table des directeurs : 6 paires de cercles, numérotées 1 à 6 du bas
  vers le haut comme dans [C].

## Joueurs

- [C] 2 à 4 joueurs, une couleur chacun. Le siège 0 est rouge, 1 bleu, 2
  vert, 3 jaune ; les couleurs restantes sont neutres.

## Mise en place

- [C] Chacun à son tour pose un jeton de la couleur de son choix sur un
  cercle libre de la table, jusqu'à ce que les 12 soient posés.
- ❓ Le siège 0 commence, et la partie continue dans l'ordre : avec 12
  jetons, c'est de nouveau au siège 0.
- Option `table` : `Draft` (défaut, la règle ci-dessus) ou `Random` (jetons
  tirés au hasard, comme dans le solo de [C]).

## Tour

1. [C] Échanger deux jetons de la table (facultatif). Seuls les échanges de
   deux couleurs différentes sont proposés, les autres ne changent rien.
2. [C] Faire entrer un employé de n'importe quelle couleur, pris dehors,
   sur une case de l'accueil, au sommet de la pile qui s'y trouve
   (facultatif). L'étape est sautée quand plus personne n'est dehors.
3. [C] Appliquer les promotions, dans l'ordre de son choix, jusqu'à ce
   qu'aucune ne s'applique. Une même paire sert autant de fois qu'il le
   faut. Quand une seule promotion s'applique, le moteur la joue ; le
   joueur choisit dès qu'il y en a plusieurs.

## Promotion

- [C] Une paire (gauche, droite) promeut l'employé **au sommet** d'une pile
  quand il est de la couleur de gauche et que la couleur sous lui est celle
  de droite. Il monte d'une case et se pose au sommet de la pile d'arrivée.
- ❓ La couleur sous lui est celle de l'employé juste dessous s'il est sur
  une pile, sinon celle de sa case (lecture de « either … or … (if he's in
  a stack) » ; les exemples de [C] s'accordent avec les deux lectures).
- Personne ne quitte la salle du conseil.

## Fin

- [C] Le premier joueur dont les trois employés sont dans la salle du
  conseil gagne, même si un autre les y a menés. Une couleur neutre n'y
  gagne rien.
- ❓ La partie s'arrête dès cet instant, même si des promotions restent :
  l'ordre choisi par le joueur actif décide qui arrive premier.
- ❓ Les autres sont classés par nombre d'employés dans la salle du conseil.
- Ajout du site : après 20 tours de table sans entrée ni promotion, la
  partie est nulle.
- Le solo de [C] (tout promouvoir en un minimum de tours) n'est pas repris.
