# Wu Hsing multijoueur : design

## Intention

Porter sur ludo-rs le Wu Hsing de Sid Sackson (« Domino Bead Game »), aujourd'hui
jeu HTML/JS autonome dans `~/prog/magic/domino`, en jeu multijoueur de la plateforme :
2 à 4 joueurs, humains (en ligne) ou IA. Cela fait partie du regroupement en Rust
des jeux de l'utilisateur dispersés dans plusieurs dépôts ; le puzzle Wu Hsing
viendra ensuite, dans un chantier séparé.

Le jeu est simple pour un ordinateur et dur pour un humain : **trouver un seul coup
légal est déjà difficile**. L'écran de jeu doit donc aider à trouver où poser,
c'est ce qui distingue ce portage d'une simple traduction du JS.

## Objectifs

- Règles de base complètes, 2, 3 et 4 joueurs, fidèles à la règle française
  (François Haffner, 2004, `~/prog/magic/domino/wuhsi_rg.pdf` pages 2–3, hors dépôt).
- Une aide à la pose réglable par chaque joueur, qui montre par défaut les
  emplacements légaux du domino choisi et leurs points.
- Une IA à trois niveaux, Difficile ≥ 0.55 contre Facile en `ludo arena`.

## Non-objectifs

- Les variantes de la règle (duel « stratégique » à 15 dominos identiques, mains
  visibles) : l'utilisateur ne veut que la base.
- Le puzzle Wu Hsing et Mountains and Valleys (autres chantiers).
- Une crate « dominos » commune avec Eurominoes / Packs Kingdoms : « c'est un jeu de
  domino mais pas vraiment, on verra plus tard ».
- Le mode même écran : les mains sont secrètes (`shared_screen` reste faux).

## Anti-objectifs

- Un plateau dont le bord empêche un coup que la règle autorise.
- Une aide qui révèle quoi que ce soit de la main d'un autre siège.
- De la logique de règle dans `web/` : l'UI n'affiche que ce que la `View` contient.

## Approches envisagées

1. **Crate `games/wu-hsing`, plateau sans bord** (choisie) : cases occupées indexées
   par coordonnées signées, le moteur énumère les poses légales et les met dans la
   vue du joueur au trait. Même schéma que les autres jeux (`legal: Vec<Action>`
   existe déjà dans Black Hole, Kurote, Bantu…), l'aide « places » n'est qu'un
   affichage de cette liste.
2. Recopier le JS sur sa grille fixe 20×14 : plus rapide, mais à 2 joueurs (30
   dominos posés au total) un bord peut couper un groupe. Écartée.
3. Crate commune « dominos » façon `pairs`/`cards` : matériel et règles trop
   différents de ceux d'Eurominoes ; mutualisation prématurée. Écartée.

## Règles (à reporter dans `games/wu-hsing/RULES.md`, sourcées [R])

- **Matériel** : 30 dominos, les 15 paires `a ≤ b` de valeurs 1–5, chacune en double.
  Chaque valeur est une couleur (palette « cinq éléments » du JS, `VALUE_COLORS`).
- **Donne** : 2 j. → 14 chacun ; 3 j. → 9 chacun, une des 3 restantes écartée face
  cachée ; 4 j. → 7 chacun. Les 2 restantes forment le carré de départ : deux
  dominos horizontaux l'un sous l'autre, à personne. Si ce sont deux doubles de la
  même valeur, maldonne : on redonne (tirage déterministe depuis la graine).
- **Premier joueur** au hasard (graine), puis sens horaire.
- **Groupes** : une suite d'au moins 2 demi-dominos contigus dans une rangée ou une
  colonne. Un groupe valide est un préfixe de la répétition d'un cycle de *p* valeurs
  distinctes, *p* = 1 à 5 (a,a,a… / a,b,a,b… / … / a,b,c,d,e,a…). Une rangée peut
  contenir plusieurs groupes séparés par des vides.
- **Pose** : un domino touche au plus 3 lignes (son axe, plus une perpendiculaire par
  demi-domino). Toute ligne de longueur ≥ 2 après la pose doit être un groupe valide.
  1. Le domino doit **prolonger au moins 2 groupes**. Sa propre ligne de 2 cases ne
     compte que si elle dépasse 2 après la pose (elle touche une tuile existante dans
     son axe). Exemples A, C, E (2 groupes), B, D (3) ; X et Z illégaux.
  2. Interdit de former 4 dominos alignés de même orientation se touchant par leur
     grand côté (exemple Y).
- **Score** : produit des nouvelles longueurs des groupes qui comptent pour la règle
  1, doublé si le domino est un double (exemple B : 4×5×4 = 80, 160 en double).
- **Passer** : seulement si aucune pose n'est légale (la règle papier, où les autres
  désignent un domino jouable, revient au même ; le moteur tranche à leur place).
  Passer rapporte 0.
- **Fin** : (a) un joueur vide sa main ; les joueurs après lui dans le tour jouent
  encore jusqu'à ce que tous aient eu autant de tours, comptés depuis le premier
  joueur (la partie s'arrête à la fin du tour de table, pas en revenant à celui qui
  a fini : le JS, `advanceTurnAfterAction`, se trompe ici quand le finisseur n'est pas
  le premier joueur ; un test Rust couvre ce cas) ; (b) tous les joueurs ont passé
  successivement.
- **Score final** : −5 par double et −25 par autre domino restant en main. Le plus
  haut score final gagne ; égalité = victoire partagée (`Outcome` habituel).

La référence d'implémentation est `~/prog/magic/domino/js/rules.js`
(`evaluatePlacement`, `scanLine`, `isValidLineSequence`, `violatesFourInLine`,
`dealDeck`, `computeStandings`) et ses tests `test/rules.test.js`, dont la
régression « la ligne propre du domino ne compte pas d'office ». Les cas de ces
tests sont portés en tests Rust.

## Moteur (`games/wu-hsing`)

- `id` : `wu-hsing`, 2–4 joueurs, aucune option.
- `State` : graine/générateur `ChaCha8Rng` (jamais `SmallRng`), mains, carte du
  plateau `(ligne, colonne) → (valeur, domino)`, scores, compteur de passes
  consécutives, état de fin de tour après vidage de main, dernier coup.
- `Action` : `Place { domino, cellule d'ancrage, sens (horizontal/vertical), retourné }`
  et `Pass`. La forme exacte est au choix du constructeur, mais deux actions qui
  produisent le même plateau ne doivent pas toutes deux être listées (un double
  retourné = le même coup ; deux dominos de mêmes valeurs dans la main = le même coup
  à l'id près : n'en lister qu'un).
- `legal_actions` : toutes les poses légales de la main ; `Pass` seulement si la
  liste est vide. Recherche limitée aux cases vides voisines du plateau existant.
- `apply` renvoie `Illegal` (message français) sans toucher l'état.
- `View` : plateau, scores, nombre de dominos de chaque main, ma main, siège au
  trait, dernier coup, fin de partie avec détail des déductions, et pour le joueur
  au trait seulement `legal: Vec<(Action, points)>`. Jamais la graine, ni les mains
  adverses, ni le domino écarté à 3 joueurs.
- `Determinize` : redistribue les dominos que l'observateur ne voit pas (mains
  adverses aux tailles inchangées, domino écarté) ; la conformance le vérifie.

## IA

- Facile : un coup au hasard parmi les poses légales, pondéré vers les gros points
  (à régler pour rester battable).
- Moyen : le coup qui rapporte le plus (l'IA du JS).
- Difficile : ISMCTS générique ; `search_actions` garde les K meilleures poses par
  points (K ≈ 12, à mesurer), `rollout_action` glouton ou quasi glouton.
- Mesure : `cargo run --release -p ludo-cli -- arena wu-hsing 200 hard easy`, à 2 et
  4 joueurs ; seuil ≥ 0.55 ; la ligne correspondante va dans « Commandes » de
  `CLAUDE.md`.

## Web (`web/src/games/wu-hsing/`)

- `types.ts`, `Board.tsx`, `rules.ts` (rappel de règles d'accord avec `RULES.md`), un
  test dans `__tests__/`, une entrée dans `web/src/games/registry.ts`. Textes en fr et
  en (`web/src/i18n/`).
- Plateau : boîte englobante des dominos posés plus une marge d'une case (deux en
  cas de besoin pour un domino), qui grandit avec la partie ; adapté au mobile
  (zoom/défilement au choix du constructeur).
- Dominos : points et couleur par valeur comme dans le JS.
- **Aide à la pose**, réglage par navigateur (préférence locale, try/catch autour de
  `localStorage`), trois niveaux :
  - *places* (défaut) : en choisissant un domino de la main, ses emplacements légaux
    s'allument sur le plateau avec leurs points ; un clic sur l'un pose le domino
    (s'il y a plusieurs poses sur la même case — sens, retournement —, l'UI permet
    de choisir entre elles).
  - *dominos jouables* : les dominos de la main qui ont au moins une pose sont
    marqués ; placement manuel.
  - *aucune* : placement manuel seul.
  En placement manuel : rotation (`R`) et retournement (`F`), aperçu au survol qui
  dit légal + points ou illégal, d'après la liste `legal` de la vue.
- Bouton « Passer » toujours visible ; s'il existe un coup, le moteur refuse et l'UI
  le dit (« un coup est possible ») sans montrer lequel, sauf aide *places* /
  *dominos jouables* qui le montrent déjà.
- Fin de partie : scores, déductions de main, gagnant.

## Enregistrement

- `crates/ludo-registry` : `static` + entrée dans `GAMES`, dépendance dans son
  `Cargo.toml` et le workspace.
- `tests/conformance.rs` : `check::<WuHsing>()` à 2, 3 et 4 joueurs.

## Laissé au constructeur

Forme exacte des types d'action et de vue, représentation interne du plateau,
réglage de Facile et de K pour Difficile, détails visuels du plateau et du choix
entre plusieurs poses sur une même case, emplacement du sélecteur d'aide.
