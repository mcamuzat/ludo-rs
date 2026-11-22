# Onglet Solo et 5×15

Date : 2026-11-22.

## Intention

Les jeux solo de l'utilisateur sont dispersés dans des pages HTML isolées
(`~/prog/magic/spiral`, `~/prog/magic/domino`, `~/prog/lepandasamuse`). Il veut
les réunir dans ludo-rs, en Rust, dans un onglet à part qui distingue les jeux
**contre un automate** (« avec IA ») des **puzzles** (« sans IA »). Le premier
jeu est **5×15** (Friedemann Friese, 2F-Spiele 2020), son préféré.

## Ce que l'utilisateur a décidé

- Onglet Solo, deux étagères : « Contre un automate » et « Puzzles ».
- 5×15 d'abord. Base : la version « Shinkansen Express »
  (`~/prog/lepandasamuse/5x15.html`) : voies ferrées, wagons 1 à 15, la
  locomotive 13-14-15 qui s'allume et fume quand elle est attelée.
- Les « 1 » actuels (symboles ●▲■◆★) sont illisibles : à remplacer.
- Approche A : 5×15 est un `Game` ludo-rs ordinaire à un joueur.
- Cryptide : ajouter une légende de carte, **rouge = puma, noir = ours**
  (le dessin actuel est juste, seule la légende manque).
- Suite, chantiers séparés : Wu Hsing multijoueur, puis le puzzle Wu Hsing
  retravaillé (« il manque quelque chose »), Packs Kingdoms et Eurominoes
  multijoueurs. Plus tard : Spiral, calendrier pentominos. Écartés : Skyway,
  Hide and Seek, Merchant's Solitaire, Chasse aux animaux.

## Approches

- **A, retenue** : `games/five-by-fifteen` implémente `Game`
  (`min_players = max_players = 1`, déjà accepté par `ludo-core`), avec vue,
  journal, WASM et conformance comme les autres jeux. Coût : `Determinize` et
  `Heuristics` triviaux pour un jeu sans information cachée.
- B, écartée : un trait « Puzzle » à part dans `ludo-core` ; seconde filière à
  entretenir, inutile aux jeux à automate qui seront des `Game` de toute façon.
- C, écartée : garder le JavaScript ; contraire à « en Rust » et à « le moteur
  possède toute la logique ».

## Le jeu (règle [R] : `~/prog/magic/spiral/2f-spiele_rules_5x15_US.pdf`)

- 75 tuiles, 5 couleurs × 1–15, mélangées (`ChaCha8Rng`, graine) sur 5 × 15 ;
  les cinq « 1 » sortent, remélangés, en colonne zéro : cinq trous.
- Action a : poser dans un trou une tuile de même couleur, +1 sur sa voisine
  de gauche ou −1 sur sa voisine de droite.
- Action b : pousser d'une case vers la droite, dans le trou juste après le
  15, une suite d'au moins 3/4/5 tuiles consécutives de même couleur finissant
  par le 15 (parcours court, moyen, long : option `Game::options`).
- Pas de sortie à droite de la colonne 15. Gagné : les cinq lignes triées.
- Perdu : plus aucun coup. Boucle : **choix de Claude**, comme la version
  actuelle, le moteur signale dans la vue une position déjà vue (avertissement
  qu'on peut ignorer) et le joueur peut abandonner ; noté [U] dans `RULES.md`
  face au « perdu » de [R].

## L'écran

- Accueil : bascule « À plusieurs / Solo » en haut, avec adresse (`#/solo`),
  réutilisable par le futur blog. L'étagère vient d'un champ du registre web
  (`solo: "automaton" | "puzzle"`) ; un jeu solo n'apparaît pas dans l'onglet
  À plusieurs. Les énigmes Cryptide déménagent dans Puzzles.
- Un jeu solo se lance sans salon ni réseau : choix des options (parcours),
  puis partie locale.
- Plateau 5×15 repris de la version Shinkansen (style, glisser-déposer et
  clic, mise en évidence des tuiles qui vont dans un trou, locomotive animée).
- **Choix de Claude** : le « 1 » de chaque voie montre un grand chiffre de la
  couleur de la voie et le nom de la ligne (Émeraude, Cramoisi…).
- **Choix de Claude** : on garde « Annuler » et les statistiques de victoires
  par parcours (stockage local du navigateur, comme aujourd'hui).
- Textes par `web/src/i18n/` (français de référence, anglais typé dessus).

## Hors périmètre

Les autres jeux solo, Wu Hsing, Packs Kingdoms, Eurominoes, le blog et
l'hébergement sur lepandasamuse.fr (le serveur Proxmox sera vu avec le blog).

## Laissé au constructeur

Forme des actions et de la vue ; détection de position répétée ; mise en
œuvre d'« Annuler » (rejeu du journal dans la session locale ou autre) ; coup
suggéré par l'IA générique si c'est gratuit ; seuil `arena` sans objet (jeu à
un joueur : la suite de tests suffit, avec un test qu'une donne connue se
résout).
