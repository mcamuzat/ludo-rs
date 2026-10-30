# ludo-rs — Siberian Dice

Date : 2026-10-30
Statut : à valider

## 1. Objectif

Ajouter **Siberian Dice** (Artem Borovkov et Eugene Panferov, 2015) : un jeu
de dés à deux joueurs sur un plateau hexagonal de 37 cases. On lance un de
ses dés, puis on le pose. Un dé est **parfait** quand son nombre de points
est égal au nombre de dés qui le touchent. Le premier joueur qui a six dés
parfaits gagne.

Le site d'origine (siberiandice.com) et l'application Android ont disparu.
Il ne reste que le PDF print-and-play de BoardGameGeek [U], et ludo-rs sert à
rejouer le jeu. On joue à deux, contre l'IA (Facile, Moyen, Difficile) ou en
ligne, comme les autres jeux du site.

### Hors périmètre

- toutes les **variantes** de la page 2 du PDF : PRIMES, STRATEGIC,
  GROSSMEISTER, PURITY, et la partie à 3 joueurs. L'utilisateur veut la
  règle de base seule [U] ;
- les animations, lancer de dé compris.

## 2. Sources

- **[R]** `siberian-dice.pnp.pdf`, révision 2.0 (BGG). Page 1 : la règle de
  base. Page 3 : le plateau numéroté. Copie locale de l'utilisateur :
  `~/Téléchargements/siberian-dice.pnp.pdf`. Elle n'est **pas** copiée dans
  le dépôt (droits d'auteur).
- **[U]** Décisions de l'utilisateur du 2026-10-30.

`games/siberian-dice/RULES.md` cite [R] ou [U] pour chaque règle, et marque ❓
ce qui est déduit.

## 3. Règles (`siberian-dice`, 2 joueurs)

### Matériel [R]

- Un plateau hexagonal de **37 cases**, en rangées de 4, 5, 6, 7, 6, 5, 4.
  Les cases ont les côtés en pointe vers le haut et le bas, comme sur la
  page 3.
- 12 dés à six faces par joueur : bleus pour le premier (siège 0), rouges
  pour le second.

### Coordonnées [R]

Le moteur reprend la numérotation de la page 3 : la case `ab` a deux
coordonnées `a`, `b` de 1 à 7, avec `|a − b| ≤ 3`. Cela donne bien 37 cases.
La rangée du milieu est 11, 22, … 77 ; la rangée du haut est 41, 52, 63, 74 ;
celle du bas est 14, 25, 36, 47.

Les six **voisines** de `ab` sont `(a−1)(b−1)`, `(a+1)(b+1)`, `a(b−1)`,
`(a+1)b`, `a(b+1)` et `(a−1)b`, quand elles sont sur le plateau. Par exemple,
44 touche 33, 55, 43, 54, 34 et 45, ce qui correspond au dessin.

### Dé parfait [R]

Un dé posé est parfait quand son nombre de points est **égal** au nombre de
dés posés sur ses cases voisines, quelle que soit leur couleur et quelle que
soit leur valeur. Avoir plus ou moins de voisins ne compte pas. Ce statut se
recalcule après chaque coup. Un 6 ne peut être parfait que sur une case qui a
six voisines, donc jamais au bord.

### Déroulement [R]

- **Début** : plateau vide, chaque joueur a ses 12 dés en réserve. Bleu
  commence.
- **Coup** : à son tour, on doit jouer. On choisit **un** de ses dés, soit
  dans sa réserve, soit sur le plateau. On le lance, et on voit le résultat.
  Ensuite seulement, on le pose sur une **case vide** de son choix, face
  obtenue visible. Le choix du dé se fait donc avant de connaître le
  résultat. Le choix de la case se fait après. Une fois le dé choisi, on ne
  revient plus en arrière.
- Un dé repris sur le plateau libère sa case. Il **peut y être reposé**,
  puisque la case est vide quand on pose : c'est la lecture littérale de
  « any vacant hex », confirmée par l'utilisateur [U]. Le seul effet est
  alors la nouvelle valeur.
- Une fois sa réserve vide, on joue forcément un dé déjà posé. Il reste
  toujours au moins 13 cases vides, donc on peut toujours poser.

### Fin de partie [R]

Après chaque coup, on compte les dés parfaits de chaque couleur. Le coup se
termine quand le dé est posé. On ne compte pas pendant qu'un dé lancé attend
d'être posé. *(Choix de conception.)*

- Si une seule couleur en a **6 ou plus**, elle gagne. C'est vrai même si le
  coup a été joué par l'adversaire : son dé a pu rendre parfaits des dés de
  l'autre couleur. [R] le dit : « not necessarily ends by the winner's move ».
- Si les deux couleurs atteignent 6 en même temps, la partie est **nulle**.

### Limite de durée [U]

C'est un **ajout de ludo-rs** et non une règle de [R]. Il est validé par
l'utilisateur le 2026-10-30 (« 30 tours comme Bantu »). Comme on peut
toujours déplacer un dé du plateau, rien ne force une partie à finir.

La partie s'arrête sur un **nul** [U], sans départage par le nombre de dés
parfaits, après **30 tours de table sans progrès**,
soit 60 coups. Il y a **progrès** quand une couleur bat son propre record de
dés parfaits pour la partie, c'est-à-dire quand elle en a plus que jamais
depuis le début. *(Choix de conception, pas une décision de l'utilisateur.)*
Un record ne dépasse jamais 5 sans que la partie se termine, donc la partie
finit toujours.

## 4. Moteur : crate `games/siberian-dice`

Elle suit le guide « Ajouter un jeu » du `CLAUDE.md` du dépôt, avec la même
forme que `games/black-pot`, qui a aussi des dés tirés par le moteur.

- **Pas de `Config`** utile, et pas d'options dans le lobby.
  `check_config` refuse tout autre nombre de joueurs que 2.
- **Actions** : `Pick(Source)`, avec `Source = Reserve | Cell(id)`, puis
  `Place(id)`. Les dés en réserve sont tous pareils, d'où un seul
  `Pick(Reserve)`. Après un `Pick`, le moteur lance le dé et attend le
  `Place` du même joueur. Le dé lancé est dans la vue, et la case qu'il a
  libérée est vide.
- **Hasard** : les lancers viennent d'une graine `ChaCha8Rng` gardée dans
  l'état. Elle ne va jamais dans une vue, et on tire un `u32` qu'on convertit
  (voir les principes du `CLAUDE.md`). Comme seuls les lancers à venir sont
  cachés, `determinize` ne fait que changer la graine. La conformance doit
  passer.
- **Vue** :
  - le contenu de chaque case : vide, ou dé avec sa couleur, sa valeur et
    s'il est parfait. Le moteur calcule ce dernier point ;
  - le nombre de dés en réserve et le nombre de dés parfaits de chaque
    couleur ;
  - le joueur au trait et l'étape où il en est (choisir un dé ou poser le dé
    lancé avec sa valeur) ;
  - le dernier coup : sa case de départ ou la réserve, sa case d'arrivée et
    la valeur obtenue ;
  - les actions légales ;
  - le compteur de la limite (tours de table restants) ;
  - la raison de la fin : victoire, nul simultané ou limite.
- **Chrono** : `clock: true`. `timeout_action` prend `Pick(Reserve)` s'il
  reste des dés en réserve, et sinon le premier dé posé. Pour poser, il
  prend la première case légale.
- **IA** : l'ISMCTS générique, sur l'avenir caché des lancers. Un coup a au
  plus 13 choix de dé, puis au plus 37 cases. Le reste est laissé au
  builder : la simulation, avec par exemple une préférence pour les poses
  qui rendent ses dés parfaits sans aider l'adversaire, et `evaluate`, avec
  par exemple l'écart de dés parfaits. Il doit garantir que les simulations
  s'arrêtent, ce que fait déjà la limite de durée.

## 5. Web : `web/src/games/siberian-dice/`

On y met `types.ts`, `Board.tsx`, `rules.ts` (à garder d'accord avec
`RULES.md`) et un test dans `__tests__/`. On ajoute aussi une entrée dans
`web/src/games/registry.ts`, en famille `board`. *Les points ci-dessous sont
des choix de conception, à corriger à la relecture.*

- **Plateau en SVG** : 37 hexagones en pointe, dans l'esprit de la page 3
  (cases bleu nuit sur fond du tapis), sans numéros.
- **Dés** : des carrés arrondis bleus ou rouges, avec les **points** de leur
  face (pas des chiffres). Un dé parfait est **mis en évidence**, par exemple
  avec un contour laiton.
- À côté du plateau, pour chaque joueur, sa réserve et son compteur de dés
  parfaits sur 6.
- **Jouer** : je clique sur ma réserve ou sur un de mes dés posés. Le dé
  est lancé, et sa valeur s'affiche près de ma réserve. Les cases vides
  s'allument, dont celle que le dé vient de quitter, et je clique sur l'une
  d'elles.
- Le dernier coup est mis en évidence (départ et arrivée), pour qu'on voie
  ce que l'adversaire a joué.

## 6. Tests

- `tests/rules.rs` :
  - la **figure d'exemple de [R]**. Dés rouges 2 et 1, dés bleus 3, 5, 6,
    rouge 4, placés comme sur la figure. Le 5 et le 3 sont parfaits, les
    autres non. Une fois le 1 déplacé en « X », le 5 ne l'est plus, et le 2,
    le 4 et le 1 le deviennent. Il faut choisir des coordonnées qui
    reproduisent la figure ;
  - les voisines d'une case du bord, d'un coin et du centre ;
  - un dé pris dans la réserve, un dé repris sur le plateau, et un dé reposé
    sur sa propre case ;
  - on ne peut plus prendre dans la réserve quand elle est vide ;
  - victoire par un coup de l'**adversaire** ;
  - nul quand les deux couleurs atteignent 6 en même temps ;
  - limite : 60 coups sans nouveau record donnent un nul, et un nouveau
    record remet le compteur à zéro ;
  - `apply` refuse un coup illégal sans rien changer (case occupée, dé
    adverse, `Place` avant `Pick`) ;
  - la séquence de lancers d'une graine est figée.
- `tests/conformance.rs` : à 2 joueurs.
- Web : la sélection d'un dé, les cases allumées et l'affichage d'un dé
  parfait.
- Arène : `ludo-cli arena siberian-dice 200 hard easy`, attendu ≥ 0.55. On
  ajoute la ligne mesurée dans « Commandes » du `CLAUDE.md`.
