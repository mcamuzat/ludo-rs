# ludo-rs — Cryptide (spec 9)

Date : 2026-10-20
Statut : proposé
Précédent : `2026-10-20-black-vienna-design.md`

## 1. Objectif

Ajouter **Cryptide** (Hal Duncan et Ruth Veevers, Osprey Games 2018, édition
française Origames), jeu de déduction pour 3 à 5 joueurs, jouable seul contre
l'IA ou en ligne, des IA complétant la table s'il manque des joueurs [U]. Une
carte de 108 cases hexagonales ; chaque joueur détient un indice secret sur
l'habitat de la créature ; ensemble, les indices désignent une seule case. On
interroge les autres joueurs, qui répondent en posant un disque (« peut-être
ici ») ou un cube (« pas ici ») ; le premier qui fouille la bonne case gagne.

L'utilisateur avait commencé le jeu en React (`~/prog/cryptid`, hors dépôt) :
il y a écrit la carte et un **générateur d'énigmes** dont il est fier, sans
finir le jeu ni l'IA. Il veut surtout **une IA du niveau de Cryptid-Bot**
(github.com/CJ1067/Cryptid-Bot, Christopher Lehman, Python) [U]. Le moteur
reprend son générateur, porté en Rust, et l'IA suit la méthode du bot.

### Hors périmètre

- **Feuille de déduction** à l'écran : « ça se joue sans » [U]. En revanche,
  une **aide de jeu** liste les indices possibles (section 5) [U].
- **Règle à deux joueurs** [R p. 14] : reportée.
- **Cartes Énigme** et livrets d'indices du jeu : remplacés par le générateur.
- **Indications** (« Il n'y a pas d'indice… », [R p. 11–12]) : liées aux cartes
  Énigme, sans objet ici.
- **Correction d'erreur** [R p. 8] : le moteur pose les réponses lui-même, il
  ne se trompe pas.
- Animations, sons, illustrations du jeu.

## 2. Sources

- **[R]** Livret de règles Origames (`~/Téléchargements/crypo_rg.pdf`, mis en
  ligne par l'Escale à jeux) : matériel p. 4, mise en place p. 5–6, partie
  p. 6–8, indices p. 9–10, résumé p. 13, deux joueurs p. 14.
- **[J]** Code de l'utilisateur, `~/prog/cryptid/src/logic/Cryptid.js` : les
  6 sections (terrains, ours, pumas), la liste des 48 indices, les voisinages
  à 1, 2 et 3 cases, le générateur (`generateRandomMap`, `generateClues`,
  `checkSolution`).
- **[B]** Cryptid-Bot, `runner.py` et `cluechecker.py` : la méthode de l'IA
  (section 6) ; `gameboard.py` décrit aussi les 6 sections, ce qui recoupe [J]
  (la section 1 concorde, terrains et ours).
- **[U]** Décisions de l'utilisateur (2026-10-20), citées là où elles servent.

Les PDF et le code de l'utilisateur restent hors du dépôt. `games/cryptide/
RULES.md` cite [R], [J], [B] ou [U] pour chaque règle et marque ❓ ce qui
n'est pas tranché.

## 3. Matériel

- **Carte** : 6 sections de 6 × 3 cases [R p. 4], assemblées en 2 colonnes de
  3 sections, soit 12 colonnes × 9 lignes. Chaque section est placée à une
  position et dans une orientation (retournée ou non) tirées au hasard [J].
  Chaque case a un **terrain** (Lac, Montagne, Forêt, Marécage, Désert) et
  peut faire partie du **territoire** d'un ours ou d'un puma [R p. 4]. Les
  données des sections viennent de [J] ; le constructeur les recoupe case par
  case avec [B], et vérifie l'assemblage, l'orientation et le voisinage sur
  l'exemple de carte de [R p. 5] (même méthode que le plateau de Bantu : une
  surimpression que l'utilisateur valide).
- **Structures** [R p. 4–5] : pierres dressées et cabanes abandonnées, en
  blanc, vert, bleu et noir. Mode normal : 6 structures (3 couleurs, sans le
  noir) ; mode avancé : les 8. Placées au hasard, une par case au plus [J] ❓1.
- **Distance** : nombre de pas d'une case à une autre sur la grille. « Dans, ou
  à N cases » inclut la case de l'élément [R p. 8].
- **Jetons** : une couleur par joueur, cubes et disques en nombre illimité
  [R p. 4].

### Indices [R p. 9–10]

- Dans un terrain parmi deux (10 paires).
- Dans, ou à 1 case d'un terrain (5).
- Dans, ou à 1 case du territoire d'un animal, ours ou puma (1).
- Dans, ou jusqu'à 2 cases d'un type de structure : pierre ou cabane (2).
- Dans, ou jusqu'à 2 cases du territoire d'un ours, d'un puma (2).
- Dans, ou jusqu'à 3 cases d'une couleur de structure : blanc, vert, bleu
  (3), plus le noir en mode avancé (1).

Mode normal : 23 indices. Mode avancé : ces 24 et leurs 24 négations
(« L'habitat ne se trouve pas… »), soit 48 [R p. 5]. Chaque indice se résout
en un ensemble de cases, calculé une fois par carte.

## 4. Règles

### Options

- **Mode** : normal (par défaut) ou avancé [R p. 5].
- 3, 4 ou 5 joueurs.

### Mise en place

- Carte et structures tirées au hasard (section 3).
- **Énigme** : un indice par joueur, tous différents [R p. 8], dont
  l'intersection est **exactement une case**, l'habitat. Chaque indice doit
  être **nécessaire** : sans lui, les autres désignent plus d'une case [J,
  `checkSolution`, appliqué par [J] à 4 et 5 joueurs ; ici à tous les nombres
  de joueurs, choix du site]. Le générateur reprend la démarche de [J]
  (premier indice au hasard, puis indices qui réduisent le plus les cases
  restantes, avec au plus deux indices « dans un terrain ») ; le constructeur
  corrige ses défauts, en garde l'esprit, et tire une nouvelle carte quand une
  carte n'a pas d'énigme. Le générateur est déterministe pour une graine.
- Un premier joueur au hasard.
- **Partage initial** [R p. 6] : à tour de rôle à partir du premier joueur,
  chacun pose un cube sur une case qui ne peut pas être l'habitat selon son
  indice, jusqu'à ce que chacun en ait posé deux. Une case à structure est
  permise, une case qui porte déjà un cube ne l'est pas.

### Tour [R p. 6–7]

À son tour, le joueur **interroge** ou **fouille**, puis la main passe à
gauche. Principes [R p. 7] :

- On n'agit jamais sur une case qui porte un cube : ni question, ni fouille,
  ni jeton.
- On ne pose pas de jeton sur une case qui porte déjà l'un des siens.
- Les jetons ne sont jamais retirés.
- Le moteur pose seul les réponses d'un joueur interrogé : elles découlent de
  son indice, ce n'est pas un choix.

**Interroger** : le joueur choisit une case et un autre joueur ; celui-ci y
pose un disque si la case peut être l'habitat selon son indice, sinon un cube.
Une case où l'interrogé a déjà un jeton est interdite (il ne pourrait pas y
répondre) ❓2. N'importe quelle autre case est permise, même une que
l'interrogateur sait impossible [R p. 7].

**Fouiller** : le joueur choisit une case qui peut être l'habitat selon son
indice et y pose un disque ; s'il y a déjà un disque, il en pose un sur une
autre case possible selon son indice, sans disque à lui [R p. 7]. Puis, dans
l'ordre du tour, chaque autre joueur répond par un disque ou un cube ; celui
qui a déjà un disque sur la case ne répond pas. Au premier cube, la fouille
s'arrête. Si personne ne pose de cube, le joueur **gagne**.

**Cube de pénalité** : si le joueur a fait poser un cube à un autre joueur
(question ou fouille), il pose lui-même un cube sur une case qui ne peut pas
être l'habitat selon son indice [R p. 7]. C'est son choix, une décision à
part (le seul coup où le moteur attend un joueur entre deux tours). S'il n'a
aucune case permise, il n'en pose pas ❓3.

### Fin

La partie se termine à la fouille réussie : le fouilleur gagne, les autres
perdent. Tous les indices et l'habitat sont alors montrés [R p. 7, « comparer
vos indices »]. Chaque tour pose au moins un jeton et les cases sont en
nombre fini : la partie finit toujours, sans limite de durée.

### Information

- Visible de tous : la carte, les structures, tous les jetons et leur
  couleur, l'historique des actions (qui a interrogé qui, où ; les fouilles).
- Chacun voit son propre indice, et ne voit ni ceux des autres ni l'habitat
  avant la fin.

## 5. Web

- **Carte** en SVG : cases colorées par terrain (couleurs de [R p. 4]),
  territoires en pourtour (ours en tirets noirs, puma en trait rouge,
  [R p. 4]), structures en octogone (pierre) et triangle (cabane) de leur
  couleur, jetons empilés par case dans la couleur de chaque joueur, pion
  Recherche sur la case en jeu.
- **Son indice**, en toutes lettres, toujours à l'écran.
- **Coup** : choisir une case, puis « Demander à <joueur> » ou « Fouiller » ;
  pour le partage initial et le cube de pénalité, choisir une case. Les cases
  jouables pour l'action en cours sont celles que le moteur déclare légales.
- **Aide de jeu** [U] : un panneau repliable qui liste les indices possibles
  du mode joué, par famille, avec la règle d'inclusion (« dans, ou à N cases
  inclut la case de l'élément »). Elle ne montre pas quels indices sont
  éliminés pour un joueur, ni aucun compte : pas de feuille de déduction [U].
- Le journal des coups récents, et en fin de partie les indices de chacun et
  l'habitat.
- Un rappel de règles (`rules.ts`) d'accord avec `RULES.md`.

## 6. IA

Pas de recherche ISMCTS : une politique par niveau
(`Heuristics::policy_action`, comme Black Vienna). Elle ne lit que la vue de
son siège.

### Ce que l'IA sait [B]

Pour chaque adversaire, les **indices encore possibles** : ceux qui
s'accordent avec tous ses jetons sur la carte (disque = la case est dans
l'indice, cube = elle n'y est pas), moins son propre indice (indices tous
différents). Tout se lit sur la carte : rien à mémoriser entre deux coups.
Elle tient la même liste pour elle-même, telle que les autres la voient.

Elle énumère ensuite les **combinaisons** d'indices possibles des adversaires
qui, avec le sien, désignent une seule case (et dont chaque indice est
nécessaire, comme le générateur) : cela resserre les listes et donne les
**cases candidates**, pondérées par le nombre de combinaisons. [B] ne le fait
que sous 2 500 combinaisons ; en Rust, avec un ensemble de cases par indice
(masque de 108 bits), le constructeur fixe le plafond selon le temps mesuré en
wasm.

### Décider [B]

- **Fouiller** s'il ne reste qu'une case candidate, ou au plus 4 et qu'une
  d'elles a plus de 49 % de chances de recevoir un disque du voisin de
  gauche ; parmi celles-là, la plus probable.
- Sinon **interroger** l'adversaire qui a le plus d'indices possibles (au
  hasard en cas d'égalité), sur la case où la part de ses indices qui y
  répondent « cube » est la plus proche d'une cible : 35 % à 3 joueurs,
  30 % à 4, 25 % à 5. Un cube lui en apprend beaucoup, un disque ne lui coûte
  rien.
- **Cube de pénalité** et **partage initial** : la case permise qui élimine
  le moins de ses propres indices possibles, vus des autres (elle se dévoile
  le moins).

### Niveaux

- **Moyen** : la méthode de [B] telle quelle, avec le partage initial que [B]
  ne gère pas.
- **Facile** : listes par adversaire sans les combinaisons ; fouille seulement
  quand l'intersection des listes ne laisse qu'une case ; question sur une
  case au hasard parmi les permises ; cubes au hasard parmi les permis.
- **Difficile** : Moyen, plus ce que le constructeur peut gagner à l'arène,
  par exemple choisir la question qui sépare le mieux les combinaisons
  restantes, ou tenir compte, pour fouiller, du risque de laisser gagner le
  joueur suivant.

Arène : Difficile ≥ 0.55 contre Facile, à 3 et à 5 joueurs, en mode normal et
avancé ; la mesure va dans les commandes du `CLAUDE.md`.

### Déterminisation

`determinize` tire, pour chaque adversaire, un indice parmi les combinaisons
compatibles avec la carte, et l'habitat qui en découle (même énumération que
l'IA) ; la conformance l'exige même si la politique ne s'en sert pas.

## 7. Structure

Comme « Ajouter un jeu » du `CLAUDE.md` du dépôt : crate `games/cryptide`
(`map.rs` pour sections, assemblage et distances ; `clues.rs` pour indices et
générateur ; règles, `view.rs`, `determinize.rs`, `heuristics.rs`,
`testing.rs`), `RULES.md`, tests de règles et conformance à 3, 4 et 5 joueurs
dans les deux modes. Id `cryptide`, famille web `modern` (étagère des jeux
d'auteur, avec Black Vienna). Option de mode par `Game::options` et
`check_config`, libellés dans `web/src/games/options.ts`. Mélanges en
`ChaCha8Rng`, tirages en `u32` (règles du dépôt).

Tests attendus en plus des habituels : la carte d'exemple de [R p. 5]
reconstruite à l'identique ; voisinages et distances sur les bords ; chaque
indice de [R p. 9–10] sur un cas ; toute énigme générée a une seule case et
des indices tous nécessaires, sur de nombreuses graines ; une graine fige la
donne.

## 8. Choix laissés au constructeur

- Rendu exact de la carte (taille des cases, palette fine, disposition du
  panneau), dans le thème « Salle de jeux ».
- Les détails du générateur, tant que les propriétés de la section 4 tiennent.
- Le plafond d'énumération et les améliorations du niveau Difficile, mesurées
  à l'arène.

## 9. Points ouverts

- ❓1 Placement des structures : au hasard sur toute la carte [J], alors que
  les cartes Énigme du jeu les placent à la main. À voir en jouant si
  certaines cartes sont trop pauvres.
- ❓2 Interroger un joueur sur une case où il a déjà un jeton : le livret ne
  le dit pas ; interdit ici, la réponse étant déjà connue.
- ❓3 Cube de pénalité sans case permise : le livret ne le prévoit pas ; le
  joueur n'en pose pas.
