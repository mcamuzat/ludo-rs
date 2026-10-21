# ludo-rs — Black-Pot

Date : 2026-10-21
Statut : livré (2026-10-21)

## 1. Objectif

Ajouter **Black-Pot** (Cogitas, Marseille) : un jeu d'alignement et de calcul
mental. On lance 3 dés, on combine les chiffres avec + − × ÷, et on pose ses
pions sur les cases qui portent les valeurs obtenues. Le but est d'aligner 5
pions, de préférence sur des cases Black-Pot. Il se joue à 2 joueurs, ou à 4
en deux équipes de 2, contre l'IA ou en ligne comme les autres jeux du site.

Le **calcul mental est le cœur du jeu** [U] : le joueur écrit lui-même son
calcul, et le site ne signale jamais quelles cases il peut atteindre.

### Hors périmètre

- une aide qui surligne les cases atteignables ;
- les animations des dés.

## 2. Sources

- **[R]** Règle Cogitas, mise en ligne par l'Escale à jeux
  (`~/Téléchargements/blapo_rg.pdf`). La page 1 est une publicité, la page 2
  le but et le barème, la page 3 les cases, le déroulement et la fin.
- **[B]** Le plateau en couleur, page 4 du même PDF.
- **[U]** Décisions de l'utilisateur, le 2026-10-21.

Le PDF n'est **pas** copié dans le dépôt (droits d'auteur). La grille des
valeurs du §4 est un relevé.

`games/black-pot/RULES.md` cite [R], [B] ou [U] pour chaque règle, et marque
❓ ce qui est déduit.

## 3. Règles (`black-pot`, 2 ou 4 joueurs)

### Matériel [R][B]

- Un plateau de **109 cases** numérotées de 1 à 25 (en fait de 1 à 24, voir
  §4), de trois sortes :
  - **citron** : valeurs 1 à 6 ;
  - **cerise** : 7, 8, 9, 10, 11, 12, 15, 16, 18, 20, 24 ;
  - **Black-Pot** : 13, 14, 17, 19, 21, 22, 23.
- 40 pions par camp (ivoire et marron). À 4 joueurs, chaque équipe partage
  ses 40 pions.
- 3 dés.

### Un coup [R][U]

Le joueur dispose des 3 chiffres tirés. Il doit **utiliser les 3** [R]. Il
les répartit en un, deux ou trois **groupes** :

- un chiffre seul vaut sa face ;
- un groupe de 2 ou 3 chiffres vaut un calcul en + − × ÷ qui utilise
  chacun de ses chiffres une fois. Toutes les formes sont permises : à 3
  chiffres, on combine deux chiffres, puis on combine le résultat avec le
  troisième, dans un sens ou dans l'autre (6 ÷ (4 − 2) comme (6 × 4) ÷ 2).
  ❓ Chaque résultat intermédiaire est un entier positif ou nul : une
  division doit tomber juste, et on ne divise pas par 0.

Chaque groupe de valeur non nulle occupe **une case libre de cette valeur**.
Tous les groupes doivent en trouver une [U] : avec 4, 2 et 6 joués
séparément, il faut une case 4, une case 2 et une case 6 libres. Deux groupes
de même valeur occupent deux cases différentes. Un groupe de valeur 0 n'occupe
rien : c'est ainsi qu'on ne pose aucun pion (2 + 4 − 6 = 0) [R]. ❓ Une paire
qui vaut 0 (3 − 3) avec un chiffre seul pose donc un seul pion.

On ne pose pas plus de pions qu'il n'en reste à son camp.

Ces contraintes donnent d'elles-mêmes les règles des cases de [R] : un
citron s'obtient d'un chiffre ou d'un calcul ; une cerise demande 2 ou 3
chiffres ; un Black-Pot demande les 3 (aucune valeur Black-Pot n'est la somme
ou le produit de deux dés). Le moteur n'a pas de règle à part pour elles.

Le calcul choisi est **annoncé** [R] : le site l'affiche à tous
(« 6 × 4 = 24 ÷ 2 = 12 »).

### Obligation de jouer [R][U]

Sur table, un joueur passe quand il ne trouve rien, sauf si l'adversaire lui
montre un coup, ce qui l'oblige à jouer. En ligne [U] : **on ne passe pas
volontairement**, et le moteur passe tout seul le tour d'un joueur qui n'a
aucun coup. Un coup à 0 pion (les 3 chiffres font 0) est un coup comme un
autre : on peut toujours le choisir quand les dés le permettent, pour
économiser ses pions [R].

**« Je ne trouve pas »** [U] : un joueur qui a un coup mais ne le trouve pas
peut le déclarer. Il passe sans poser de pion, comme sur table, et le site
ne lui montre pas ce qu'il a manqué. C'est une action `NotFound`, légale
dès qu'un coup existe. L'IA ne la choisit jamais, car elle trouve toujours
ses coups. La règle de bonne foi (« je ne passe pas exprès ») ne se vérifie
pas : on fait confiance au joueur, comme sur table.

### Ordre de jeu [R][U]

- **À 2 joueurs** [U] : chacun lance les dés à son tour et joue son propre
  tirage.
- **À 4 joueurs** [R] : deux équipes, A1 et A2 contre B1 et B2, assises en
  alternance (sièges 0 = A1, 1 = B1, 2 = A2, 3 = B2). A1 lance les dés ; B1
  joue ce tirage le premier, puis A2, B2 et A1 jouent le **même** tirage. Au
  tour suivant, B1 lance, A2 joue le premier, et ainsi de suite : le lanceur
  joue toujours en dernier. Les équipiers ne communiquent pas.
- Le premier lanceur est tiré au hasard d'après la graine (sur table, le plus
  grand dé commence).
- Les dés se lancent tout seuls au début de chaque tirage (mon choix) ; le
  tirage est visible de tous.

### Alignements [R][U]

- Un alignement, c'est **5 pions d'un même camp** sur 5 cases consécutives
  d'une ligne, d'une colonne ou d'une diagonale.
- Dès qu'un alignement est complet, ses pions sont **retournés** et son
  rapport est aussitôt marqué [R].
- Les croisements sont possibles [R] : un pion peut servir à des
  alignements de directions différentes, mais **une seule fois par
  direction** [U]. Six pions en ligne ne font donc qu'un alignement dans
  cette direction.
- ❓ Quand un coup crée dans une direction une suite de plus de 5 pions
  libres (pas encore comptés dans cette direction), le moteur y retient les
  alignements qui ne se chevauchent pas et rapportent **le plus** au joueur
  (une suite de 10 en donne deux). À égalité, le choix est fixe et
  déterministe.

### Barème [R]

| 5 cases alignées | points |
|---|---|
| dont 4 Black-Pot | 400 |
| dont 3 Black-Pot | 250 |
| dont 2 Black-Pot | 100 |
| 5 cerises | 100 |
| 5 citrons | 50 |
| autres (« diverses ») | 30 |

Aucun alignement de 5 Black-Pot n'existe sur ce plateau (vérifié sur le
relevé : 216 alignements possibles, dont 20 à 4 Black-Pot). Un alignement à
un seul Black-Pot est « divers ».

### Fin [R]

- La partie s'arrête **dès qu'un camp pose son dernier pion**.
- Chaque camp compte ses alignements, plus **10 points par pion non joué**.
  Le camp qui a le plus de points gagne ; une égalité est partagée.
- **Limite de durée**, ajout du site (mon choix) : en fin de partie, les
  cases libres se raréfient et les tirages utiles aussi. La partie s'arrête
  après **10 tours de table de suite sans pion posé** (un coup à 0 pion ou
  un tour passé ne compte pas comme pion posé), et on compte comme ci-dessus.

## 4. Le plateau [B]

Un carré de 11 × 11 aux coins coupés : des rangées de 7, 9, puis sept de 11,
puis 9 et 7 cases, centrées (109 cases). Valeurs relevées de haut en bas,
de gauche à droite, sur la page 4 :

```
         19  8  3  7  5  7 17
      1  12 17  9  2  8 13 11  1
  19 11  13 10  3 20  4  9 14 12 17
   2 14  16  2  4 23  3  1 15 21  2
   5 15   4  6 21 18 21  5  3 16  6
   1  2  10 22 24 23 24 22 10  2  1
   6 16   3  5 21 18 21  6  4 15  5
   2 21  15  1  3 23  4  2 16 14  2
  17 12  14  9  4 20  3 10 13 11 19
      1  11 13  8  2  9 17 12  1
         17  7  5  7  3  8 19
```

La sorte de chaque case se déduit de sa valeur (§3), et le dessin le
confirme case par case. Le plateau est symétrique par rotation d'un demi-tour,
ce qui sert de contrôle au relevé (un test le vérifie). Comptes : 42 citrons,
38 cerises, 29 Black-Pot.

Le 6 et le 9 se distinguent par la sorte : 6 est un citron, 9 une cerise.

- Le plateau vit dans le moteur (`games/black-pot/src/board.rs`), seule
  source ; le site le dessine d'après la vue ou un JSON produit par un test,
  au choix du builder (Bantu fait le second).
- **Point d'arrêt utilisateur** : le builder dessine le plateau du site et
  le montre à côté de la page 4 du scan **avant** d'écrire les règles. On ne
  continue qu'après son accord.

## 5. Moteur : crate `games/black-pot`

Elle suit la structure de `games/bantu` : `lib.rs`, `board.rs`, `view.rs`,
`heuristics.rs`, `testing.rs`, `RULES.md`, `tests/`.

- **`Config`** : aucune option (`#[serde(default)]` quand même).
- **Action** : `Play { groups: Vec<Group> }`, avec `Group { expr: Expr,
  cell: Option<CellId> }` et `Expr = Die(index) | Op(op, Box<Expr>,
  Box<Expr>)`. `cell` vaut `None` si et seulement si le groupe vaut 0.
  `apply` vérifie tout : chaque dé du tirage employé exactement une fois,
  calculs justes (§3), cases libres, distinctes et de la bonne valeur,
  pions suffisants. Il renvoie `Illegal` avec un message clair (« 6 × 4 ÷ 2
  donne 12, pas 14 ») sans rien modifier.
- **`legal_actions`** : un représentant par résultat distinct, c'est-à-dire
  par ensemble de cases occupées (plus le coup à 0 pion s'il existe), avec
  un calcul canonique. `apply` accepte n'importe quel calcul juste, pas
  seulement le canonique.
- **Hasard** : les tirages viennent d'un `ChaCha8Rng` dont la graine ne va
  jamais dans une vue (règle du dépôt : jamais de `SmallRng`, tirer des
  `u32`). Le tirage courant est public ; les suivants sont cachés.
  `determinize` remplace la graine des tirages futurs. `hidden_info` est
  donc vrai.
- **Vue** : chaque case (valeur, sorte, camp qui l'occupe, directions où
  son pion est déjà compté) ; le tirage courant et qui l'a lancé ; qui joue ;
  les pions restants et le score de chaque camp ; les alignements marqués
  (cases, rapport) ; le dernier coup de chaque joueur avec son calcul en
  texte annoncé, ses cases et les alignements qu'il a faits ; les tours
  passés ; le compteur de la limite de durée ; la raison de la fin. **Pas
  de liste des coups légaux** dans la vue : ce serait une aide.
- **Chrono** : `clock: true`. `timeout_action` joue `NotFound` : le temps
  écoulé ne révèle aucun coup.
- **Coups cachés** [U] : l'interdit porte sur l'**écran seulement**. La
  session commune envoie toujours `legalActions` au navigateur, et on ne la
  modifie pas. Comme `NotFound` est légal dès qu'un coup existe, un joueur
  qui a un coup en a au moins deux. Le coup unique joué automatiquement par
  `web/src/session/table.ts` ne se déclenche donc jamais pour révéler un
  coup. Le Board n'utilise pas `legalActions`, sauf pour savoir que c'est à
  lui de jouer.
- **IA** : ISMCTS générique. Le nombre de coups peut dépasser le millier :
  `Heuristics::search_actions` restreint la recherche aux coups sensés
  (au choix du builder : ceux qui prolongent ou bloquent des suites, les
  Black-Pot, le coup à 0 pion). La simulation et `evaluate` s'appuient sur
  le potentiel des lignes encore ouvertes. L'IA joue toujours des calculs
  justes ; elle « trouve » tous les coups, quel que soit son niveau, et le
  niveau joue sur la qualité du choix.

## 6. Web : `web/src/games/black-pot/`

`types.ts`, `Board.tsx`, `rules.ts` (d'accord avec `RULES.md`), un test dans
`__tests__/`, une entrée dans `web/src/games/registry.ts` (famille `board`,
comme Yavalath et Bantu).

- **Plateau en SVG** d'après le plateau du moteur : carrés citron (jaune
  pâle), cerise (orange) et Black-Pot (brun, étoile verte), avec la valeur.
  Il reprend l'esprit du scan sans le copier.
- **Pions** : ivoire et marron, face unie ; un pion compté dans un
  alignement est retourné (marqué). Les alignements marqués sont tracés.
- **Dés** : les 3 dés du tirage, bien visibles.
- **Saisie du calcul** [U] : je compose un groupe en cliquant des dés et des
  opérations (+ − × ÷), avec les deux formes à 3 chiffres ; puis je clique
  la case visée. Je recommence pour les autres groupes jusqu'à avoir employé
  les 3 dés, ou je déclare un groupe « = 0 ». Un bouton annule le groupe en
  cours, un autre valide le coup. L'écran **n'affiche pas le résultat** du
  calcul en cours et ne surligne aucune case : c'est le moteur qui accepte
  ou refuse, avec son message. Le frontend ne calcule rien.
- Un bouton **« Je ne trouve pas »** envoie `NotFound`.
- Le calcul annoncé du dernier coup de chacun s'affiche, avec les points
  marqués.

## 7. Tests

- `board.rs` : 109 cases ; rangées de 7, 9, 11…; sortes d'après les
  valeurs ; symétrie d'un demi-tour ; 216 fenêtres d'alignement, aucune à
  5 Black-Pot.
- `tests/rules.rs` : chiffres séparés ; paire et triple, toutes formes ;
  division inexacte et par 0 refusées ; dé employé deux fois ou oublié
  refusé ; case occupée ou de mauvaise valeur refusée ; groupe sans case
  refusé ; coup à 0 pion ; paire nulle plus chiffre seul ; obligation de
  jouer et tour passé tout seul ; `NotFound` légal seulement si un coup
  existe, et absent des coups de l'IA ; pions insuffisants ; chaque ligne du
  barème ; croisement permis, même direction refusée ; suite de 6 et de
  10 ; ordre de jeu à 2 et à 4 (lanceur en dernier, même tirage pour les
  4) ; fin au dernier pion ; points des pions non joués ; égalité ; limite
  de durée ; `apply` refuse sans rien changer.
- `tests/conformance.rs` : 2 et 4 joueurs.
- Web : composition d'un groupe, envoi d'un coup, affichage d'un refus.
- Arène : `ludo-cli arena black-pot 200 hard easy`, attendu ≥ 0.55, puis à
  4 joueurs. Ajouter la ligne mesurée dans « Commandes » du `CLAUDE.md`.

## 8. Livraison

1. Plateau, tests de structure et dessin de contrôle. **Validation par
   l'utilisateur.**
2. Règles, vue, IA, conformance, registre.
3. Web.

`README.md` (liste des jeux) et `CLAUDE.md` (arène) sont mis à jour à la
fin.

## 9. Choix faits sans l'utilisateur

- Résultats intermédiaires entiers et positifs ou nuls, division exacte
  (❓).
- Un groupe qui vaut 0 n'occupe rien, y compris une paire nulle (❓).
- Une suite de plus de 5 pions libres donne les alignements qui rapportent
  le plus, sans chevauchement (❓).
- Les dés se lancent tout seuls ; le premier lanceur est tiré au hasard.
- Limite de durée de 10 tours de table sans pion posé.
- L'IA trouve tous les coups à tous les niveaux.
- Famille web `board`.
