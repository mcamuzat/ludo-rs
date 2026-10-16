# ludo-rs — Bantu (spec 7)

Date : 2026-10-16
Statut : à valider

## 1. Objectif

Ajouter **Bantu** (Parker Brothers 1955, édité aussi par Summit Games) : un
jeu de parcours **sans hasard**. Chaque pion porte un numéro, et ce numéro
fixe la longueur de son déplacement. Plusieurs pions dans la même rangée
additionnent leurs numéros. Le jeu est rare : l'utilisateur en possède une
version courte japonaise (trois pions, plateau différent), mais pas le
plateau du scan. Les scans sont donc la seule référence.

L'utilisateur veut **« un truc qui marche »** d'abord, à 2, 3 ou 4 joueurs,
contre l'IA ou en ligne comme les autres jeux du site.

### Hors périmètre

- la variante en équipe (2 contre 2, partenaires face à face) : plus tard,
  peut-être ;
- le plateau de l'édition japonaise de l'utilisateur ;
- les animations.

## 2. Sources

- **[R]** Règle française de Summit Games, mise en ligne par François
  Haffner (jeuxsoc.free.fr, septembre 2005).
- **[B]** Plateau scanné en quatre quarts (pages 4 à 7 du même PDF) et en
  une image entière de 420 × 420 px.
- Fichiers locaux de l'utilisateur : `~/Téléchargements/bantu_rg.pdf` (la
  page 1 est une publicité) et `~/Téléchargements/bantu_01.gif`. Ils ne sont
  **pas** copiés dans le dépôt (droits d'auteur).

`games/bantu/RULES.md` cite [R] ou [B] pour chaque règle, et marque ❓ ce
qui est déduit.

## 3. Règles (`bantu`, 2 à 4 joueurs)

### Matériel [R][B]

- 4 couleurs, 4 pions numérotés 1 à 4 par couleur.
- Au centre, les **cases de départ** 1 à 4.
- Quatre **lignes de départ** rouges de 4 cases, en rayons : ligne 1 en bas,
  2 à droite, 3 en haut, 4 à gauche (vues de l'image entière).
- Des **rangées** : plusieurs cases reliées par un fond marron. Des cases
  isolées, et des liaisons bleu clair entre cases.
- L'**arrivée** (« FINISH »), en bas à gauche, reliée à 4 cases marquées
  d'une flèche.

### Parcours [B], déduit ❓

On avance dans le **sens horaire**. Chaque quart relie une ligne de départ à
la suivante : 4 → 3 (en haut à gauche), 3 → 2, 2 → 1, puis 1 → arrivée (en
bas à gauche). Les flèches des lignes de départ et les flèches bleues
confirment ce sens. Un pion sorti sur la ligne N part donc de la ligne N :
le pion 1 n'a qu'un quart à parcourir, le pion 4 presque tout le tour.

Les liaisons bleu clair n'ont pas de sens dessiné. Leur sens se **déduit du
sens horaire** général : la liaison mène vers la ligne de départ suivante,
ou vers l'arrivée. Les flèches bleues marquent les passages à sens unique.

### Déroulement [R]

- **Début** : les pions sont sur les cases de départ de leur numéro. On joue
  à tour de rôle, sens horaire, et le premier joueur est le siège 0.
- **Coup** : à son tour, on déplace **un seul** pion. On peut aussi sortir
  un pion de sa case de départ vers **n'importe laquelle** des 4 cases de la
  ligne de départ de son numéro, si elle est libre. Sortir un pion constitue
  le coup entier. Un pion pris peut être ressorti à tout moment.
- **Obligation** : on doit jouer si c'est possible. Sinon on passe son tour.
- **Longueur du déplacement**, comptée au début du tour :
  - un pion seul dans sa rangée (ou sur une case isolée) avance du numéro
    qu'il porte ;
  - si au moins deux pions sont dans la même rangée ou sur la même ligne de
    départ, quelle que soit leur couleur, chacun d'eux avance de la **somme**
    des numéros de tous les pions de cette rangée ou de cette ligne.
    ❓ « Côte à côte » se lit comme « dans la même rangée », même s'ils ne
    sont pas sur des cases voisines.
- **Chemin** :
  - il faut faire exactement ce nombre de pas, sauf pour entrer dans
    l'arrivée ;
  - on avance dans le sens du parcours, et on peut glisser latéralement le
    long d'une rangée ou d'une ligne de départ (dans les deux sens) ;
  - on ne repasse jamais par une case déjà traversée pendant ce tour ;
  - on ne recule jamais, et on ne passe jamais entre deux cases non
    reliées ;
  - on ne passe **jamais par-dessus** un pion, quelle que soit sa couleur.

  ❓ Quand plusieurs chemins existent, le joueur choisit sa case
  d'arrivée parmi toutes celles qu'un chemin permis atteint.
- **Arrivée** : un pion qui atteint une case fléchée **avant** la fin de son
  déplacement peut entrer dans l'arrivée avec un ou plusieurs pas restants :
  le compte exact n'est pas nécessaire. ❓ Un pion qui s'arrête pile sur une
  case fléchée y reste, et entrera dans l'arrivée à un tour suivant. Un pion
  entré dans l'arrivée n'en ressort plus.
- **Prise** : on ne termine jamais sur un pion de sa couleur. Finir
  exactement sur un pion adverse le renvoie à sa case de départ, sauf si
  ce pion est **protégé** :
  - il est sur une ligne de départ ;
  - ou il partage sa rangée avec au moins un autre pion, de n'importe
    quelle couleur.

  On ne peut pas terminer sur un pion protégé.
- **Fin** : le premier joueur qui a mis tous ses pions dans l'arrivée gagne.
  ❓ Les autres sont classés selon le nombre de pions qu'ils ont dans
  l'arrivée, puis selon leur **avancement** : la somme, sur leurs pions
  restants, du plus court chemin jusqu'à l'arrivée dans le sens du parcours
  (plus elle est petite, mieux c'est ; un pion sur sa case de départ compte
  un pas de plus que la plus proche case de sa ligne). Les ex æquo
  partagent leur rang.
- **Limite de durée** : c'est un **ajout de ludo-rs**, validé par
  l'utilisateur le 2026-10-16, et non une règle de [R]. Comme on peut
  glisser dans les deux sens d'une rangée, un pion peut faire des
  allers-retours sans jamais avancer. La partie s'arrête donc après **30
  tours de table** (30 × nombre de joueurs coups ou tours passés) **sans
  progrès**. Il y a progrès quand un pion est pris, entre dans l'arrivée, ou
  atteint une case plus proche de l'arrivée que toutes celles où il est allé
  depuis sa dernière sortie. *Ajusté à l'implémentation :* la première
  version (« sans prise ni arrivée ») arrêtait des parties normales dès le
  début, quand aucun pion ne peut encore arriver. Elle s'arrête aussi
  si plus aucun joueur ne peut bouger. Dans les deux cas, on classe comme
  ci-dessus, sans vainqueur désigné à l'avance.

### Option du lobby

- `short` (Toggle, faux par défaut) : partie courte [R], seulement avec les
  pions 1, 2 et 3. Les pions 4 ne sont pas utilisés, et les règles ne
  changent pas.

## 4. Le plateau : relevé et vérification

C'est **le principal risque** de ce cycle. Le graphe se relève à la main sur
les quatre quarts de [B] (pages 4 à 7 du PDF, plus nettes que l'image
entière). Pour chaque case, on note :
- un identifiant ;
- ses coordonnées (pour l'affichage) ;
- sa rangée ou sa ligne de départ, ou aucune pour une case isolée ;
- si elle porte une flèche d'arrivée.

On note ensuite chaque liaison, avec son sens.

- Le plateau semble montrer une **symétrie d'ordre 4** autour du centre, mais
  l'arrivée l'interrompt dans le quart en bas à gauche. Le builder vérifie
  cette symétrie quart par quart **avant** de s'en servir pour générer trois
  quarts à partir d'un seul. Toute différence se relève à part.
- Le plateau vit dans le moteur (`games/bantu/src/board.rs`), qui en est la
  **seule source**. Le site le dessine à partir des données du moteur. La
  forme est au choix du builder : un champ de la vue, ou un JSON statique
  produit par un test qui échoue s'il n'est plus à jour.
- Des tests de structure portent sur le graphe : 4 lignes de départ de 4
  cases ; 4 cases fléchées ; depuis chaque case de chaque ligne de départ,
  l'arrivée est atteignable dans le sens du parcours ; aucune case
  inaccessible ; pas de cycle orienté hors des glissements dans une rangée.
- **Point d'arrêt utilisateur** : une fois le relevé fait, le builder
  produit une image du graphe superposée au scan, ou mise côte à côte avec
  lui. Il la montre à l'utilisateur **avant** d'écrire les règles. On ne
  continue qu'après son accord.

## 5. Moteur : crate `games/bantu`

Elle suit la structure de `games/yavalath` : `lib.rs`, `board.rs`, `view.rs`,
`heuristics.rs`, plus `RULES.md` et `tests/`. Information parfaite :
`hidden_info: false`, et `Determinize` est l'identité.

- **`Config`** : `short: bool`.
- **Action** : `Move { piece: u8, to: Dest }`, avec `Dest = Cell(id) |
  Finish`. Sortir un pion est un `Move` vers une case de sa ligne de
  départ. Deux chemins qui mènent à la même case donnent le même coup.
- **Passer** : le moteur passe tout seul le tour d'un joueur qui n'a aucun
  coup. Aucune action « passer » n'existe, et `pending` désigne toujours un
  joueur qui a un coup.
- **Vue** : la position de chaque pion (case de départ, case, arrivée) ; le
  tour ; la longueur de déplacement de chaque pion du joueur au trait ; les
  pions protégés ; le dernier coup (pion, départ, arrivée, prise
  éventuelle) ; les coups légaux ; le **compteur de la limite de durée**
  (tours de table restants avant l'arrêt). La raison de la fin (victoire,
  limite, blocage) est aussi dans la vue.
- **Chrono** : `clock: true`. `timeout_action` joue le premier coup légal.
- **IA** : la recherche générique, en UCT puisque l'information est
  parfaite. La simulation préfère prendre, entrer dans l'arrivée ou avancer
  le plus loin, au choix du builder. `evaluate` coupe les simulations
  longues d'après l'avancement de chacun. Il faut **garantir qu'elles
  s'arrêtent**, car les prises répétées peuvent faire tourner une partie
  longtemps.

## 6. Web : `web/src/games/bantu/`

`types.ts`, `Board.tsx`, `rules.ts` (d'accord avec `RULES.md`), un test dans
`__tests__/`, une entrée dans `web/src/games/registry.ts` (famille `board`,
comme Yavalath) et le libellé de l'option dans `options.ts`.

- **Plateau en SVG** dessiné d'après le graphe : cases rondes, rangées sur
  fond marron, lignes de départ rouges, liaisons bleu clair, flèches,
  arrivée, centre avec les cases de départ. Il reprend l'esprit du scan sans
  le copier pixel pour pixel.
- **Pions** : des disques de la couleur du joueur, avec le numéro.
- **Jouer** : je clique sur un de mes pions jouables. Ses cases d'arrivée
  possibles s'allument, et je clique sur l'une d'elles. L'arrivée est une
  zone cliquable quand elle fait partie des destinations.
- Le dernier coup est mis en évidence, et un message signale une prise.

## 7. Tests

- `board.rs` : les tests de structure du §4.
- `tests/rules.rs` : sortie sur sa ligne ; longueur seule et en rangée
  (somme, toutes couleurs) ; glissement latéral ; pas de retour en arrière
  ni de case repassée ; blocage par un pion sur le chemin ; prise ; pions
  protégés en rangée et sur une ligne de départ ; entrée dans l'arrivée sans
  compte exact ; arrêt pile sur une case fléchée ; tour passé sans coup ;
  victoire et classement (pions arrivés, puis avancement) ; limite de
  durée (un aller-retour répété arrête la partie au 30e tour de table sans
  progrès, et une prise ou une arrivée remet le compteur à zéro) ; arrêt
  quand personne ne peut bouger ; partie courte ; `apply` refuse un coup
  sans rien changer.
- `tests/conformance.rs` : 2, 3 et 4 joueurs, normale et courte.
- Web : sélection d'un pion, destinations, dernier coup.
- Arène : `ludo-cli arena bantu 200 hard easy`, attendu ≥ 0.55, puis à 4
  joueurs. Ajouter la ligne mesurée dans « Commandes » du `CLAUDE.md`.

## 8. Livraison

Un plan, avec un point d'arrêt :

1. Relevé du plateau, tests de structure et image de contrôle. **Validation
   par l'utilisateur.**
2. Règles, vue, IA, conformance, registre.
3. Plateau web.

`README.md` (liste des jeux) et `CLAUDE.md` (arène) sont mis à jour à la
fin.

## 9. Choix faits sans l'utilisateur

- Le sens des liaisons bleu clair se déduit du sens horaire (❓).
- « Côte à côte » se lit comme « dans la même rangée » (❓).
- Le joueur choisit sa case d'arrivée quand plusieurs chemins existent
  (❓).
- Un pion arrêté pile sur une case fléchée attend le tour suivant pour
  entrer (❓).
- Les perdants sont classés par nombre de pions arrivés, puis par
  avancement (❓). La partie s'arrête si plus personne ne peut bouger (❓).
- Limite de durée : principe validé par l'utilisateur ; les 30 tours de
  table et la mesure de l'avancement sont mon choix.
- Le siège 0 commence ; les tours sans coup passent tout seuls.
- La famille web est `board`, comme Yavalath.
