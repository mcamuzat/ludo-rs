# ludo-rs — Das glorreiche Manöver

Date : 2026-10-22
Statut : à valider

## 1. Objectif

Ajouter **Das glorreiche Manöver** (Roland Siegers, Schmidt Spiel +
Freizeit, 1992 ; en français « Les Glorieuses Manœuvres ») : une course de
84 cases où chaque soldat avance d'autant plus loin qu'il est **en retard**
et **haut perché**. Le premier de la course n'avance que d'une case, le
cinquième de cinq. Monter sur une pile ou sur une pièce bonus multiplie le
déplacement. Le premier soldat rentré à la caserne marque 12 points, le
suivant 11, et ainsi de suite.

Le jeu est **introuvable** [U] : le site sert à le rejouer. L'utilisateur ne
le connaît que par la règle jeuxsoc. Il se joue de 2 à 4 joueurs, contre
l'IA (Facile, Moyen, Difficile) ou en ligne avec des IA pour compléter la
table, comme les autres jeux du site.

### Hors périmètre

- les **deux variantes** de la règle (troupe disciplinée, sanctions pour les
  indisciplinés) : l'utilisateur les fera ajouter si le jeu de base lui
  plaît [U]. Les numéros des soldats sont déjà affichés pour les préparer ;
- la mise en place libre des pièces bonus de la règle d'origine ;
- les animations.

## 2. Sources

- **[R]** Traduction française de François Haffner pour jeuxsoc.fr, mise en
  ligne par l'Escale à jeux (`~/Téléchargements/glorr_rg.pdf`). Page 1 :
  publicité ; pages 2 et 3 : la règle ; page 4 : les jetons à imprimer.
- **[B]** Le plateau, pages 5 et 6 du même PDF : les numéros des cases sont
  du **texte positionné** (96 mots : 84 cases et 12 places de caserne en
  chiffres romains) sur une image de fond (texture verte, cercles).
- **[K]** Critique allemande de Claudia Schlee et Andreas Keirat
  (Spielphase), qui paraphrase la règle Schmidt :
  <http://ftp.informatik.rwth-aachen.de/keirat/txt/D/Dasglorr.html>. La règle
  Schmidt elle-même n'a pas été trouvée en ligne.
- **[U]** Décisions de l'utilisateur, le 2026-10-22.

Le PDF n'est **pas** copié dans le dépôt (droits d'auteur).
`games/manoever/RULES.md` cite [R], [K] ou [U] pour chaque règle, et marque
❓ ce qui est déduit.

## 3. Règles (`manoever`, 2 à 4 joueurs, jeu de base)

### Matériel [R]

- Un parcours de **84 cases**, de la caserne à la caserne. Les cases 1 à 6 et
  79 à 84 sont blanches (jaunes sur [B]), les autres roses (rouges sur [B]).
- La **caserne** : 12 places numérotées I à XII.
- **12 soldats** en tout, numérotés : 2 joueurs, 6 chacun (rouge, jaune) ;
  3 joueurs, 4 chacun (+ bleu) ; 4 joueurs, 3 chacun (+ vert). Le siège 0
  est rouge, puis jaune, bleu, vert. ❓ À 3 et 4 joueurs, rouge et jaune
  n'utilisent que les soldats 1 à 4 ou 1 à 3.
- **Pièces bonus**, aux cases fixes de la suggestion de [R] [U] :
  - noires (×3) sur 12, 20, 47 et 65 ;
  - blanches (×2) sur 8, 24, 31, 36, 60 et 74 ;
  - grises (gratification) sur 25 (1 point), 40 (2) et 55 (3).

  [K] dit que la règle d'origine laisse placer ces pièces librement (hors
  des 6 premières cases, et hors des 6 dernières et des cases occupées pour
  les noires et blanches) ; l'utilisateur a choisi les cases fixes.

### Mise en place [R][K]

À tour de rôle, en commençant par le siège 0, chaque joueur lance le dé et
pose un de ses soldats encore à la caserne sur la case 1 à 6 du résultat.
Si cette case porte déjà une pile de 3 soldats, on relance. Quand les 12
soldats sont sur le parcours, le dé ne sert plus. ❓ Le soldat posé est celui
de plus petit numéro encore à la caserne : son numéro ne compte pas dans le
jeu de base. Il y a toujours une case libre (12 soldats, 6 cases de 3).

### Un tour [R][K]

Après la mise en place, on continue dans le même ordre, à partir du siège 0.

- Un soldat est **libre** s'il est seul sur sa case ou en haut de sa pile.
- À son tour, le joueur déplace **chacun** de ses soldats libres, **une
  fois**, un par un, dans l'ordre qu'il veut. Il ne peut pas terminer son
  tour tant qu'un soldat libre peut encore bouger.
- Un soldat libéré pendant le tour, parce qu'un soldat du même joueur a
  quitté le haut de sa pile, bouge aussi pendant ce tour.
- Un soldat recouvert pendant le tour par un soldat du même joueur attend le
  tour suivant [K]. La note du traducteur dans [R] dit la même chose.
- Un soldat déjà déplacé ce tour ne rebouge pas, même s'il est encore libre.
- Un soldat libre sans aucune destination permise reste où il est ❓. En
  pratique, cela n'arrive pas (voir « Fin »).
- Un joueur sans soldat libre passe son tour [K]. Le moteur le passe tout
  seul.

### Déplacement [R][K]

Le soldat avance d'**au moins 1** case et d'**au plus** son déplacement
maximum, au choix du joueur. On compte le maximum **au moment où ce soldat
bouge** ❓, puisque chaque déplacement change les positions :

maximum = position × hauteur × bonus

- **Position** : on compte les cases occupées par des soldats, depuis la
  tête de la course. Toutes les cases qui portent au moins un soldat
  comptent, quelle que soit leur couleur ou leur nombre. Les soldats d'une
  même case ont la même position. Les cases vides, les pièces noires,
  blanches et grises ne comptent pas. Les soldats rentrés à la caserne ne
  comptent plus [K].
- **Hauteur** : l'étage du soldat dans sa pile, 1 directement sur le
  plateau. Une pièce noire ou blanche compte comme un étage [R] : un soldat
  posé sur une pièce noire est à l'étage 2.
- **Bonus** : ×2 pour un soldat posé **directement** sur une pièce blanche,
  ×3 directement sur une pièce noire, ×1 sinon [R] (« s'il se trouve sur une
  pièce blanche »). Un soldat à l'étage 3, posé sur un autre soldat qui est
  lui-même sur la pièce, n'a pas le bonus.

Exemples : un soldat en 4e position posé sur une pièce noire (donc à
l'étage 2) avance de 4 × 2 × 3 = 24 cases au plus [R]. Un second soldat
posé sur lui, à la même position, est à l'étage 3 sans bonus : 4 × 3 = 12.
Comme la pièce compte dans la limite de 3 étages, une pièce bonus porte au
plus 2 soldats.

**Divergence entre les sources.** [K] compte la hauteur en soldats
seulement et donne le bonus à toute la case (un soldat seul sur une pièce
noire en 4e position : 4 × 1 × 3 = 12). L'utilisateur a choisi la lecture
littérale de [R] le 2026-10-22 [U] : c'est la traduction de la règle, et son
exemple chiffré colle à cette lecture.

### Atterrir [R][K]

- On peut terminer sur une case occupée : le soldat se pose en haut de la
  pile, quelle qu'en soit la couleur.
- Une pile ne dépasse **jamais 3 étages**, pièce noire ou blanche comprise.
  On ne peut pas terminer sur une case qui en a déjà 3 ; on peut la
  dépasser [K].
- Un soldat qui termine sur une pièce **grise** la gagne : elle quitte le
  plateau et rapporte sa valeur au joueur en fin de partie. Une pièce grise
  ne compte pas comme un étage.
- On ne recule jamais.

### Arrivée et fin [R][K]

- Un soldat dont le déplacement dépasse la case 84 rentre à la caserne. ❓
  Le compte exact n'est pas nécessaire : il suffit que le maximum le
  permette (au moins 85 − sa case).
- Il prend la plus haute place libre : le premier rentré va sur la XII et
  marque 12 points, le suivant sur la XI, et ainsi de suite jusqu'à la I.
- Quand le 11e soldat rentre (place II), le dernier soldat encore sur le
  parcours est posé aussitôt sur la place I [R] (« dès que la case 2 est
  occupée par l'avant-dernier bidasse »), et la partie finit. Ce dernier
  soldat ne prend pas de pièce grise au passage. Chaque joueur
  additionne les places de ses soldats et ses pièces grises. Le plus grand
  total gagne. ❓ En cas d'égalité, la victoire est partagée.
- **La partie finit toujours** : un soldat en position p a un maximum d'au
  moins p, et il y a au plus p − 1 cases occupées devant lui, donc au moins
  une case libre (ou la caserne) à sa portée. Chaque tour, chaque joueur
  qui a un soldat libre avance donc d'au moins une case. Un soldat recouvert
  finit libéré, car le soldat qui le couvre doit bouger au tour de son
  propriétaire. Aucune limite de durée n'est nécessaire, et la rentrée
  automatique du dernier soldat évite la longue fin où il avancerait seul
  d'une case par tour.

## 4. Le plateau : relevé et vérification

- `scripts/manoever-board.py <règle.pdf> <dossier>` lit les pages 5 et 6
  avec `pdftotext -bbox` et relève le centre de chaque case (1 à 84) et de
  chaque place de caserne (I à XII). Il écrit `games/manoever/src/
  board_data.rs` et une image de contrôle, comme `scripts/bantu-board.py`.
- **Assemblage** [B] : la page 6 se pose **au-dessus** de la page 5. La
  case 42 sort par le haut de la page 5, la 43 entre par le bas de la
  page 6. La caserne se reconstitue de même, I à VI (page 6) au-dessus de
  VII à XII (page 5), en colonne à droite. Le plateau assemblé est en
  hauteur, à peu près carré. On l'affiche **tel qu'assemblé**, sans le
  tourner [U].
- Le moteur porte les coordonnées, et c'est la seule source. Le site les
  lit dans un JSON statique, produit par un test qui échoue s'il n'est plus
  à jour (même mécanisme que Bantu, `UPDATE_BOARD_JSON=1`).
- Tests de structure : 84 cases et 12 places, deux cases consécutives
  voisines à l'écran (distance bornée), aucune case superposée, et les
  cases bonus du §3 sur des cases roses.
- **Point d'arrêt utilisateur** : le builder montre l'image de contrôle
  (plateau dessiné d'après le relevé, à côté du rendu des pages 5 et 6)
  avant d'écrire les règles.

## 5. Moteur : crate `games/manoever`

Structure de `games/bantu` : `lib.rs`, `board.rs`, `board_data.rs`,
`view.rs`, `heuristics.rs`, `RULES.md`, `tests/`.

- **`Config`** : vide pour l'instant (`#[serde(default)]`), prête pour les
  variantes.
- **Hasard** : seulement les lancers de dé de la mise en place. Ils se
  tirent d'une graine de l'état, avec `ChaCha8Rng` comme Black-Pot (jamais
  `SmallRng`, jamais un `usize`). `hidden_info: true` (réglage fixe de
  `GameMeta`) : les lancers à venir sont inconnus, et `determinize` retire
  la graine. Après la mise en place, l'information est parfaite.
  La vue ne contient jamais la graine.
- **Actions** :
  - `Roll` pendant la mise en place : lance le dé et pose le soldat. Le
    joueur clique sur le dé ; les relances sont automatiques.
  - `Move { soldier, to: Dest }` ensuite, avec `Dest = Square(n) |
    Barracks`. Deux destinations au-delà de 84 sont le même coup
    (`Barracks`).
- **Fin de tour** : automatique quand le joueur n'a plus de soldat à
  déplacer. Pas d'action « passer » : `pending` désigne toujours un joueur
  qui a une action.
- **Vue** : chaque case avec sa pièce bonus et sa pile (soldats de bas en
  haut, couleur et numéro) ; les soldats encore à la caserne de départ ;
  la caserne d'arrivée (qui sur quelle place) ; les pièces grises restantes
  et gagnées ; le joueur au trait ; pour lui, les soldats qu'il doit encore
  déplacer, leur position, hauteur, bonus, maximum et destinations ; le
  dernier lancer ; les derniers déplacements du tour précédent de chaque
  joueur ; les scores en cours ; la fin.
- **Chrono** : `clock: true`. `timeout_action` lance le dé, ou déplace le
  premier soldat à déplacer vers sa destination la plus lointaine.
- **IA** : la recherche générique. Un tour fait plusieurs actions du même
  joueur, comme dans les autres jeux du site. Si les coups sont trop
  nombreux (un maximum de 24 donne 24 destinations), `search_actions`
  restreint la recherche, au choix du builder. La simulation préfère les
  coups qui montent sur une pile ou une pièce bonus, rentrent à la caserne
  ou prennent une pièce grise. `evaluate` coupe les simulations longues
  d'après les points acquis et l'avancement. Arène attendue ≥ 0.55
  Difficile contre Facile, à 2 et à 4 joueurs.

## 6. Web : `web/src/games/manoever/`

`types.ts`, `Board.tsx`, `rules.ts` (d'accord avec `RULES.md`), un test dans
`__tests__/`, une entrée dans `web/src/games/registry.ts`, famille `board`.
Nom affiché : « Das glorreiche Manöver ».

- **Plateau en SVG** d'après les coordonnées du moteur : le chemin en
  serpentin, les cases rondes numérotées (blanches/jaunes aux deux bouts,
  roses ailleurs), la caserne en colonne I à XII. Il reprend l'esprit de
  [B] sans le copier.
- **Pièces** : disques ; soldats de la couleur du joueur avec leur numéro ;
  pièces noires, blanches et grises (avec leur valeur). Une pile se dessine
  en disques décalés pour qu'on lise chaque étage.
- **Jouer** : je clique sur un de mes soldats à déplacer. Le site affiche son
  calcul (« 4e × étage 2 × noire = 24 ») et **surligne** les cases où il peut
  aller [U] ; la caserne est cliquable si elle en fait partie. Les soldats
  qui restent à déplacer ce tour sont marqués.
- Le dé s'affiche pendant la mise en place, avec un bouton pour le lancer.
- Les déplacements du dernier tour sont mis en évidence ; un message signale
  une pièce grise gagnée et une rentrée à la caserne.
- Les scores en cours et la caserne se lisent à tout moment.

## 7. Tests

- `board.rs` : les tests de structure du §4.
- `tests/rules.rs` :
  - mise en place : soldats par joueur, case = dé, relance sur une pile
    de 3, fin de la mise en place ;
  - position (cases vides, plusieurs soldats d'une case, soldats rentrés
    ignorés), hauteur (pièce noire ou blanche comptée comme étage), bonus,
    exemple 4 × 2 × 3 = 24 ;
  - recalcul du maximum après chaque déplacement du tour ;
  - minimum 1, pas de pile de 4, dépassement d'une pile de 3 ;
  - soldat libéré qui bouge, soldat recouvert par un camarade qui attend,
    soldat déjà déplacé qui ne rebouge pas ;
  - tour passé sans soldat libre ; fin de tour automatique ;
  - pièce grise gagnée ; rentrée sans compte exact, places 12 à 1 ;
  - bonus seulement pour le soldat posé directement sur la pièce (étage 3
    sans bonus), au plus 2 soldats sur une pièce bonus ;
  - rentrée automatique du dernier soldat sur la place I quand le 11e
    rentre, fin, scores, égalité partagée ;
  - `apply` refuse un coup sans rien changer.
- `tests/conformance.rs` : 2, 3 et 4 joueurs.
- Web : sélection d'un soldat, destinations, dé.
- Arène : `ludo-cli arena manoever 200 hard easy`, puis à 4 joueurs ;
  ligne mesurée dans « Commandes » du `CLAUDE.md`.

## 8. Livraison

Un plan, avec un point d'arrêt :

1. Relevé du plateau, tests de structure et image de contrôle. **Validation
   par l'utilisateur.**
2. Règles, vue, IA, conformance, registre.
3. Plateau web.

`README.md` (liste des jeux) et `CLAUDE.md` (arène) sont mis à jour à la
fin.

## 9. Choix faits sans l'utilisateur

- Le soldat posé à la mise en place est celui de plus petit numéro (❓).
- Identifiant `manoever`, famille `board`, nom affiché en allemand.
- Le joueur clique pour lancer le dé ; les relances sont automatiques.

Validés par l'utilisateur dans la conversation : jeu de base seul, cases
bonus fixes, plateau tel qu'assemblé, destinations surlignées, hauteur et
bonus comptés selon [R] à la lettre (contre [K]). Proposés par
moi faute de source et acceptés tels quels : le maximum se calcule au moment
où le soldat bouge ; pas de compte exact pour rentrer à la caserne ; un
soldat libre sans destination reste sur place ; égalité = victoire partagée.
