# ludo-rs — Black Vienna (spec 8)

Date : 2026-10-20
Statut : livré
Précédent : `2026-10-19-volltreffer-design.md`

## 1. Objectif

Ajouter **Black Vienna** (Gilbert Obermair), jeu de déduction pour 3 à 6
joueurs, jouable contre l'IA, sur le même écran ou en ligne. Trois personnages
sur 27 sont mis de côté : le trio criminel. On interroge les autres joueurs avec
des cartes Enquête de trois personnages ; ils répondent par un nombre de jetons.
Le premier qui nomme le trio gagne.

Le jeu est introuvable dans le commerce : l'utilisateur veut y jouer sur le
site, avec une **feuille d'enquête à l'écran** qui prend des notes simples
toute seule, et une IA dont la force de déduction suit le niveau choisi, pour
qu'un humain qui joue bien puisse gagner.

### Hors périmètre

Les illustrations des personnages (origine et licence inconnues ; le dépôt est
public) : cartes avec la lettre seule [U]. Accusation hors de son tour [U].
Déductions automatiques au-delà des faits directs [U]. Plusieurs parties par
match. Animations et sons.

## 2. Sources

- **[R]** Règle française de François Haffner, d'après la traduction anglaise
  de Joe Huber (`bvien_rg.pdf`, pages 2 à 5).
- **[C]** Planche des cartes (`bvienna_cards.pdf`) : les 27 personnages et la
  liste des 36 cartes Enquête.
- **[F]** Feuille d'enquête plastifiée (`Black_Vienna_A4_lamination_sheet.pdf`) :
  27 lignes, une colonne par joueur, une ligne « Black Vienna ».
- **[U]** Décisions de l'utilisateur (2026-10-20), citées là où elles servent.

Les PDF restent hors du dépôt. `games/black-vienna/RULES.md` cite [R], [C],
[F] ou [U] pour chaque règle et marque ❓ ce qui n'est pas tranché.

## 3. Matériel

- **27 personnages** [C] : A à Z, puis Ö (dans cet ordre, celui de [F]).
- **36 cartes Enquête** [C], chacune de trois personnages, chaque personnage
  sur exactement 4 cartes : ACL, AGM, AOS, APQ, BCY, BHV, BLM, BQT, CFI, CSX,
  DHR, DJZ, DLS, DVY, EGW, ENQ, EÖR, EUV, FÖY, FRX, FSZ, GKO, GPX, HNÖ, HUZ,
  IÖW, IPR, ITZ, JMO, JQX, JTY, KMU, KNT, KPW, LNV, OUW. Un test vérifie les
  4 apparitions.
- **40 jetons** [R] dans la réserve.

## 4. Règles

### Mise en place [R]

- Un donneur, tiré au sort (❓1). Les 36 cartes Enquête sont mélangées et
  partagées en **3 piles de 12**, face visible : la carte du dessus de chaque
  pile est jouable.
- Les 27 personnages sont mélangés ; les 3 premiers, cachés, forment
  **Black Vienna**. Les 24 autres sont distribués un par un à partir de la
  gauche du donneur : 8 chacun à 3 joueurs, 6 à 4, 4 à 6 ; à 5, le donneur en
  reçoit 4 et les autres 5.

### Tour [R]

- Le donneur joue le premier ; ensuite, **le joueur qui vient d'être
  interrogé** joue.
- Au début de son tour, le joueur peut **accuser** (voir plus bas) ; [U]
  seulement à son tour, jamais pendant celui d'un autre.
- Puis il **interroge** : il prend la carte du dessus d'une pile et la pose
  devant un adversaire de son choix. Celui-ci y met autant de jetons qu'il a
  de ces personnages en main (0 à 3), pris dans la réserve. Le moteur calcule
  la réponse : elle n'est pas un choix du joueur.
- **Enquête spéciale** : quand chaque joueur a été interrogé au moins une fois
  et qu'au moins 6 cartes ont été jouées, l'interrogateur peut ajouter à sa
  carte une carte déjà jouée **restée sans jeton**. L'interrogé répond à
  chacune séparément. La vieille carte quitte sa place pour aller devant le
  nouvel interrogé, avec sa nouvelle réponse (❓2).
- Toutes les cartes jouées, leurs jetons et leur interrogé restent visibles
  de tous.

### Accusation [R]

- L'accusateur nomme trois personnages en secret et regarde le trio.
- **Juste** : la partie s'arrête. Il marque **3 points par jeton** encore dans
  la réserve. Chaque autre joueur qui n'a pas encore accusé nomme à son tour
  un trio, en secret et tous en même temps ; s'il est juste, il marque
  **1 point par jeton**. Les autres marquent 0.
- **Fausse** : il continue de jouer (il interroge et répond), ne peut plus
  accuser et marque 0. Comme dans [R], il a regardé le trio : il le voit
  désormais dans sa vue. Les autres apprennent seulement qu'il s'est trompé
  (❓3 : son trio n'est pas montré).

### Fin sans vainqueur [R]

- Quand tous les joueurs sauf un ont accusé à tort, la partie est
  interrompue aussitôt (l'accusateur fautif ne fait pas son enquête). Le
  dernier joueur **peut** poser une question finale (une enquête spéciale si
  la condition tient), puis **doit** accuser ; juste, il marque 1 point par
  jeton (pas 3), sinon 0. ❓4 : ce sont mes lectures de « peut poser une
  question finale et doit ensuite faire une accusation ».
- S'il reste **3 jetons ou moins** dans la réserve, la partie s'arrête :
  personne ne marque. ❓5 : vérifié juste après chaque réponse ; la question
  finale du dernier joueur y échappe (il accuse quand même).
- ❓6 : une réponse qui demande plus de jetons que la réserve n'en a prend ce
  qui reste ; la réponse affichée reste le vrai nombre de personnages.
- ❓7 : si les trois piles sont vides, le joueur dont c'est le tour ne peut
  plus qu'accuser (s'il le peut encore) ou faire une enquête avec la seule
  vieille carte sans jeton, si la condition de l'enquête spéciale tient ; sans
  rien de tout cela, la partie s'arrête sans vainqueur.

### Résultat

Classement aux points, égalités conservées. Une partie par match.

## 5. Feuille d'enquête (web)

Reprend [F] : 27 lignes (A … Ö), une colonne par joueur, la ligne
« Black Vienna » en bas, et sous la grille l'historique des enquêtes (carte,
interrogé, jetons ; enquête spéciale signalée).

- **Notes automatiques, faits directs seulement** [U]. Le moteur les met dans
  la `View` du joueur, dans une liste séparée de ses notes à lui :
  - ses propres cartes : « + » dans sa colonne, « − » dans les autres colonnes
    et dans la ligne Black Vienna ;
  - une enquête à **0 jeton** : « − » pour ses trois personnages chez
    l'interrogé ;
  - une enquête à **3 jetons** : « + » pour ses trois personnages chez
    l'interrogé.
  Rien d'autre : ni propagation d'un « + » aux autres colonnes, ni
  raisonnement sur 1 ou 2 jetons.
- **Notes du joueur** : un clic fait tourner une case entre vide, « + », « − »
  et « ? ». Une case automatique ne se modifie pas (elle se distingue des
  notes manuelles). Les notes manuelles sont un état d'affichage, pas une
  règle : elles restent dans le navigateur (mémoire de la partie en cours,
  perdues à la fin), jamais dans le moteur ni sur le réseau.
- **Aucune statistique**, comme pour les jeux Pairs.
- Le joueur choisit son accusation et ses trios en cliquant trois lettres.

## 6. IA

La force de **déduction** suit le niveau [U] :

- **Facile** : ne retient que les faits directs (ceux de la feuille
  automatique).
- **Moyen** : ajoute les déductions simples d'un bon joueur, appliquées
  jusqu'à ce que rien ne bouge :
  - un personnage qu'un joueur a, aucun autre ne l'a, ni le trio ;
  - une enquête dont le nombre de jetons égale le nombre de personnages encore
    possibles chez l'interrogé : il les a tous ; égal au nombre de « + » déjà
    connus : il n'a pas les autres ;
  - un joueur dont tous les « + » sont connus (autant que sa main) n'a rien
    d'autre ; un joueur dont tous les « − » sont connus a tout le reste ;
  - un personnage exclu de tous les joueurs est dans le trio.
- **Difficile** : déduction complète, par énumération des répartitions
  compatibles avec tout ce qu'il a vu (taille des mains, réponses).

L'IA **tente sa chance** comme un humain un peu pressé [U] : elle accuse à
son tour quand, selon ses déductions, il ne reste plus que **3 trios
possibles ou moins (Facile), 2 ou moins (Moyen), 1 (Difficile)**, en nommant
l'un d'eux (le plus plausible). Une accusation fausse l'élimine, comme un
humain. Quand la règle l'oblige à accuser (dernier joueur), ou après
l'accusation juste d'un autre, elle nomme le trio le plus plausible.

Le choix de la question (pile, interrogé, enquête spéciale) cherche à
apprendre le plus, mesuré par ses déductions du niveau ; Facile y mêle du
hasard. Le moteur de recherche générique (ISMCTS) n'est pas nécessaire : une
politique directe suffit, plus lisible.

**Plateforme** : `Heuristics` ne reçoit pas le niveau. Il faut un crochet
générique dans `ludo-core` qui le lui passe (par exemple une action de
politique selon le niveau, appelée avant la recherche, `None` par défaut),
sans changer le comportement des autres jeux. `Determinize` reste exigé par la
conformance : il redistribue les mains des autres et le trio d'une façon
compatible avec ce que l'observateur a vu (tailles des mains et réponses
publiques), par recherche avec retour arrière.

Arène : Difficile contre Facile, mesurée à 3 et à 5 joueurs et notée dans le
`CLAUDE.md` du dépôt ; Moyen contre Facile aussi, pour vérifier l'échelle.

Réalisé : Difficile ne cherche pas trio par trio (11 s au pire à 6 joueurs)
mais personnage par personnage : un personnage reste criminel possible si une
donne entière le permet. C'est exact quand il ne reste que trois personnages,
le seul cas où Difficile accuse. Mesuré (200 parties) : Difficile contre
Facile 0,70 à 3 joueurs, 0,64 à 5 ; Moyen contre Facile 0,68 à 3 ;
Difficile contre Moyen 0,54 à 3, 0,52 à 5.

## 7. Structure

Comme « Ajouter un jeu » du `CLAUDE.md` du dépôt : crate `games/black-vienna`
(id `black-vienna`), `RULES.md` sourcé, tests de règles et conformance pour
3 à 6 joueurs, registre, `web/src/games/black-vienna/` (types, plateau,
rappels de règles, test), famille web `modern` (étagère « Jeux de cartes
d'auteur ») [ma lecture], README.

## 8. Choix laissés au constructeur

Les ❓ ci-dessus sont des choix du site, à confirmer en jouant. Le détail des
heuristiques de question et la forme exacte du crochet de niveau sont libres,
dans le respect du §6.
