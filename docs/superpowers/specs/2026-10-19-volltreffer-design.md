# ludo-rs — Volltreffer (spec 7)

Date : 2026-10-19
Statut : livré
Précédent : `2026-10-19-monad-design.md`

## 1. Objectif

Ajouter **Volltreffer** (*Le Labyrinthe souterrain et les 5 tribus*, Günter
Burkhardt), 3 à 5 joueurs, jouable contre l'IA, sur le même écran ou en ligne.
Chaque manche, on engage 8 aventuriers en les payant de ses points, on pose des
groupes, puis on joue 8 plis. Une partie se gagne en finissant une manche dans
le Sanctuaire (61–70), le plus près possible de 66.

### Hors périmètre

La variante « la carte révélée prend la place libérée, puis on échange deux
cartes » (non retenue par l'utilisateur). Animations et sons.

## 2. Sources

- **[R]** Règle traduite en anglais par Jason Leveille (`volltreffer_rules.pdf`).
  Règle de référence.
- **[E]** Traduction espagnole de Fer Murciano (`Voltreffer_-_Reglamento.pdf`),
  faite sur [R]. Sa page 1 garde des bribes d'une autre mise en page de [R]
  (« 4.2.1 Publishing a Party — In clockwise or[der] according to th… »).
- **[A]** Aide de jeu (`volltreffer_player_aid.pdf`) : « Reveal cards (optional) »
  en phase 2.
- **[P]** Plateau : piste 1–80, places d'engagement 0/1/2/4, case « Blessing »,
  départs marqués à 25 (3★ et 4★) et 30 (5★), Sanctuaire 61–70 autour du 66,
  flèche « Oops! TELEPORTER » de 80 vers 41.
- **[U]** Décisions de l'utilisateur (2026-10-19) : mains des adversaires
  **cachées** ; on ne peut pas engager une carte qu'on ne peut pas payer ;
  option « Sanctuaire 63–69 » ; **aucune limite de durée**.

Les PDF restent hors du dépôt. `games/volltreffer/RULES.md` cite [R], [E], [A],
[P] ou [U] pour chaque règle et marque ❓ ce qui n'est pas tranché.

## 3. Matériel

- **Cartes** [R] : 50, valeurs 1 à 10 dans 5 tribus (couleurs). La règle ne
  nomme pas les tribus : le site les appelle par leur couleur (rouge, vert,
  bleu, jaune, violet ; mon choix, le rouge et le vert sont sur la photo de [R]).
- **Piste** [P] : un score de 0 à 80 par joueur (0 n'est pas une case, mais
  on peut y tomber en payant exactement ce qu'on a).
- **Départ** [R] : 25 points à 3 et 4 joueurs, 30 à 5.
- **Carte de négociation** [R] : à un joueur tiré au hasard au début, puis
  passée à gauche (siège suivant) à chaque nouvelle manche.

## 4. Règles

### Manche

- **Bénédiction** [R] : on mélange les 50 cartes et on retourne la première.
  Sa tribu est l'atout de la manche ; cette carte ne sert plus à rien d'autre.
- **Étal** [R] : on retourne 4 cartes, placées sur les coûts 0, 1, 2, 4. Les
  atouts vont sur les coûts les plus hauts, le plus fort sur le 4 ; parmi les
  autres, plus la valeur est haute, plus le coût est haut ; à valeur égale, la
  carte arrivée en dernier va à gauche (moins chère). Les 4 premières sont
  retournées une à une, avec la même règle. Le moteur range l'étal par
  (atout, valeur, ordre d'arrivée inverse).
- **Engager** [R] : à partir du détenteur de la carte de négociation, dans
  l'ordre des sièges, chacun prend une carte de l'étal et recule de son coût,
  puis une nouvelle carte est retournée et l'étal rangé. Huit tours de table :
  chacun finit avec 8 cartes. [U] On ne peut prendre qu'une carte dont le coût
  ne dépasse pas son score ; la carte à 0 est toujours possible. Après le
  dernier engagement, on ne retourne plus de carte (mon choix : sans effet).
  La pioche suffit toujours : 1 + 4 + 39 = 44 cartes au plus à 5 joueurs.
- **Groupes** [R][A] : dans l'ordre des sièges à partir du détenteur de la
  carte de négociation (❓ : [R] ne fixe pas d'ordre, les bribes de [E] disent
  « in clockwise order »), chacun choisit les combinaisons qu'il montre, en une
  seule action. Montrer est facultatif [A]. Combinaisons :
  - groupe de 3 cartes de même valeur : 5 points ; de 4 : 10 ; de 5 : 25. Un
    groupe montre **toutes** les cartes de sa valeur et rapporte selon sa
    taille [R] (« only the largest party for a given rank ») ;
  - 5 tribus (une carte de chaque tribu) : 5 points. ❓ Une carte peut servir
    à la fois dans un groupe et dans les 5 tribus ([R] ne l'interdit pas).
  Les points sont marqués aussitôt. Les cartes montrées restent visibles de
  tous jusqu'au début des plis [R], puis redeviennent cachées.
- **Action préventive** [R] : une fois tous les groupes choisis, le joueur à la
  plus petite somme des valeurs de ses 8 cartes marque 5 points. Égalité : le
  plus de 1, puis le plus de 2, etc. ❓ Si deux mains ont exactement les mêmes
  valeurs, le premier dans l'ordre des sièges à partir du détenteur de la carte
  de négociation (mon choix). Le site fait la vérification que la règle confie
  aux joueurs (montrer sa main sur demande) : rien n'est montré.
- **Plis** [R] : le joueur de l'action préventive entame. Il faut fournir la
  tribu demandée si on le peut ; sinon on joue ce qu'on veut (couper n'est pas
  obligatoire). Le plus fort atout gagne le pli, sinon la plus forte carte de la
  tribu demandée. Le gagnant marque 5 points aussitôt et entame le pli suivant.
- **Téléporteur** [R][P] : un gain qui dépasserait 80 continue depuis 41
  (78 + 5 = 43). Les pertes ne descendent jamais sous 0 ([U], par la règle
  d'engagement).

### Fin

- **Victoire** [R] : à la fin d'une manche (après le 8ᵉ pli), un joueur dans le
  Sanctuaire (61–70, ou 63–69 avec l'option) gagne. S'ils sont plusieurs : le
  plus près de 66, puis le plus haut score, puis victoire partagée.
- **Classement** (mon choix) : les gagnants premiers ; les autres par la même
  règle (distance à 66, puis score), les joueurs du Sanctuaire avant ceux qui
  n'y sont pas.
- Sinon, nouvelle manche : on remélange tout, nouvelle Bénédiction, la carte de
  négociation passe à gauche. Les scores restent, y compris entre 71 et 80 [R].
- **Pas de limite de durée** [U].

## 5. Information cachée

- **Public** : scores, Bénédiction, étal, nombre de cartes en main, détenteur de
  la carte de négociation, groupes montrés (pendant la phase 2 seulement), plis
  en cours et dernier pli, plis gagnés dans la manche, joueur de l'action
  préventive.
- **Mains** [U] : chacun voit la sienne ; des autres, seulement le nombre de
  cartes. Chaque engagement se fait à la vue de tous : pendant les phases 1
  et 2, la vue montre les engagements du dernier tour de table (qui, quelle
  carte, quel coût), comme on les voit passer à table. Les plus anciens, il
  faut s'en souvenir ; l'IA les oublie aussi.
- **Pioche** : cachée.
- **Determinize** : les cartes que l'observateur ne voit pas (mains des autres,
  pioche) sont redistribuées au hasard. Restent en place : les cartes montrées
  pendant la phase 2, celles des engagements que la vue affiche, et nul ne
  reçoit une tribu qu'il n'a pas fournie. Les tailles de main sont gardées.
  `redeal_where` de `games/cards` est rendu générique pour servir ici.

## 6. Moteur (`games/volltreffer`, crate à part)

« Ajouter un jeu » du `CLAUDE.md`. Identifiant `volltreffer`, nom affiché
« Volltreffer », 3 à 5 joueurs, `hidden_info: true`, `clock: true`.

- `Config { narrow_sanctuary: bool }`, option `Toggle` par défaut `false`.
- Types : `Tribe` (5), `Card { tribe, value }`.
- Actions : `Hire { card }`, `Publish { groups: Vec<u8>, tribes: bool }`
  (valeurs triées, une écriture par coup ; `legal_actions` énumère tous les
  sous-ensembles des combinaisons disponibles), `Play { card }`.
- `timeout_action` : l'engagement gratuit, `Publish` vide, la première carte
  jouable.
- La `View` porte tout ce que le plateau affiche, y compris les cartes jouables
  et le coût de chaque place de l'étal.

## 7. IA

ISMCTS générique. `evaluate` coupe une simulation à la fin de la manche où elle
a commencé, avec le classement de fin de partie (le reste est du hasard).
`rollout_action` : engager une carte au hasard parmi les abordables, montrer
ce qui rapporte le plus sans dépasser le haut du Sanctuaire, jouer une carte au
hasard. Mesure : `ludo-cli arena volltreffer 200 hard easy`, Difficile ≥ 0.55
contre Facile, à 3 et à 5 joueurs.

## 8. Web

- `web/src/games/volltreffer/` : `types.ts`, `Board.tsx`, `rules.ts`, un test.
- Table : en haut l'étal (4 cartes sur 0, 1, 2, 4) et la Bénédiction ; au
  centre la piste 1–80 en 8 rangées de 10, le Sanctuaire marqué, un pion de
  couleur par joueur ; autour, les adversaires (cartes en main, plis gagnés,
  groupes montrés, carte de négociation) ; le pli en cours ; en bas, sa main.
- Phase 2 : on coche les combinaisons à montrer (le site les propose), puis
  « Montrer ».
- Accueil (mon choix) : nouvelle famille `modern`, étagère « Jeux de cartes
  d'auteur » ; Volltreffer y est seul pour l'instant.

## 9. Livraison

Pas de plan séparé (jeu court, sur le modèle de Monad) : un commit pour le
moteur, l'IA et le registre, un pour le web, un pour la doc. Relecture par un
relecteur Opus à faire.

Mesures (2026-10-19) : arène Difficile contre Facile 0.917 à 3 joueurs
(200 parties), 0.865 à 5 (100 parties).
