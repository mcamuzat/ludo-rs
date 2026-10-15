# ludo-rs — Cartes traditionnelles : le Président et les options de partie (spec 4)

Date : 2026-10-15
Statut : plan 2 livré
Précédent : `2026-10-13-pairs-design.md` (cycle 3)

## 1. Objectif

Ouvrir sur l'accueil une famille **« Cartes traditionnelles »** (jeux au paquet
de 52, 32 ou 54 cartes), avec un premier jeu, le **Président**, jouable contre
l'IA, sur le même écran ou en ligne. Les variantes du Président se règlent dans
le lobby : ce cycle livre donc aussi le **formulaire d'options de partie**,
générique, reporté depuis la spec plateforme (`ConfigForm`, §7 et §8 de
`2026-09-26-plateforme-yavalath-design.md`).

Le matériel commun (`games/cards/src/deck.rs`) doit permettre d'ajouter ensuite
un autre jeu classique (Belote, Huit américain…) en n'écrivant que ses règles,
son IA et son plateau.

### Hors périmètre

Variantes putsch, passage de tour, « 2 utilisé comme atout », « … ou rien »,
changement de places entre les manches, échanges négociés ; partie à 3 joueurs
(paquet réduit) et à plus de 7 joueurs (double paquet) ; traduction du site
(l'i18n fera l'objet d'un cycle séparé : les libellés nouveaux sont déjà rangés
par clé, voir §5) ; animations et sons.

## 2. Sources des règles

- **[W]** Wikipédia, article « Trou du cul (jeu) » (la page « Président (Jeu de
  cartes) » y redirige), consulté le 2026-10-15. Source principale.
- **[P]** Pagat.com, « President » (`pagat.com/climbing/president.html`),
  consulté le 2026-10-15, pour ce que [W] ne précise pas.

`games/cards/RULES.md` cite [W] ou [P] pour chaque règle. Les points que les
sources ne tranchent pas sont marqués ❓ ; ceux de la §3 ont été validés par
l'utilisateur le 2026-10-15.

## 3. Règles du Président (`cards-president`, 4 à 7 joueurs)

### Base

- **Matériel** [W] : paquet de 52 cartes. Les couleurs ne comptent pas.
- **Ordre** [W] : 3 < 4 < 5 < 6 < 7 < 8 < 9 < 10 < Valet < Dame < Roi < As < 2.
- **Donne** [W] : chacun reçoit le même nombre de cartes ; les cartes en trop
  sont montrées à tous et écartées pour la manche.
- **Jeu** [W] : sens horaire. L'entame pose une à quatre cartes de même valeur.
  Le suivant passe, ou « monte » : même nombre de cartes, valeur strictement
  supérieure.
- **Passer** [P] : passer n'empêche pas de jouer à son tour suivant dans le même
  pli.
- **Fin du pli** [W] : quand tous les autres joueurs encore en jeu ont passé à la
  suite, le dernier à avoir joué entame le pli suivant ; s'il est sorti, la main
  passe au joueur à sa gauche.
- **Sortie et titres** [W] : premier sorti Président, deuxième Vice-Président,
  avant-dernier Secrétaire (Vice-Trou du cul), dernier Trou du cul ; les autres
  sont Neutres (aucun à 4 joueurs). Le dernier joueur qui a encore des cartes est
  Trou du cul sans jouer.
- **Places fixes** : les parties « statiques » de [W]. Les sièges ne changent
  pas d'une manche à l'autre.
- **Première manche** [W] : le joueur à gauche du donneur entame (donneur : siège
  0).
- **Manches suivantes** [W] : le Trou du cul donne, le joueur à sa gauche entame.
  Après la donne et avant l'entame :
  - le Trou du cul donne au Président ses deux meilleures cartes, le Président
    lui rend deux cartes de son choix ;
  - le Secrétaire donne au Vice-Président sa meilleure carte, le Vice-Président
    lui rend une carte de son choix ;
  - les Neutres n'échangent rien.
- **Points** [W] : Président n−1, puis un de moins par rang, Trou du cul 0.
  ❓ La première manche rapporte des points comme les autres (pour [W], elle ne
  sert qu'à fixer les titres).
- **Fin de partie** : après N manches (option, 5 par défaut) ; classement par
  total, les ex-aequo partagent leur rang.

### Variantes (options du lobby, toutes désactivées par défaut)

- **Révolution** [W] : poser quatre cartes de même valeur inverse l'ordre des
  valeurs (le 3 devient la plus forte, le 2 la plus faible, les jokers forts
  restent au-dessus de tout) jusqu'à la fin de la manche. ❓ Un nouveau carré
  inverse de nouveau l'ordre. L'ordre normal revient à chaque nouvelle manche.
- **Interdiction de finir sur un 2** [W] : un joueur qui pose sa dernière carte
  avec un ou plusieurs 2, ou un ou plusieurs jokers, est Trou du cul.
  ❓ Pendant une révolution, la carte interdite est le 3 (extension permise par
  [W]) ; les jokers restent interdits.
  ❓ Plusieurs fautifs se classent derrière tous les autres joueurs, le premier
  fautif tout en bas.
- **Jokers** : `Sans` (52 cartes), `Forts` ou `Wild` (54 cartes).
  - **Forts** [W][P] : le joker est au-dessus du 2 ; il se pose seul ou avec
    l'autre joker (paire de jokers), jamais avec une autre valeur.
  - **Wild** [P] : le joker prend la valeur des cartes qu'il accompagne (un 7 et
    un joker forment une paire de 7). Seuls, ou à deux, les jokers sont la
    combinaison la plus forte de leur taille. ❓ Un joker vaut une carte
    naturelle de même valeur (égalité, donc il faut monter au-dessus).
    ❓ Un carré complété par des jokers déclenche la révolution.
  - Pour les échanges, le joker est la meilleure carte.

## 4. Moteur : crate `games/cards`

Structure calquée sur `games/pairs` : une crate, un module par jeu, le matériel
commun dans `deck.rs`.

### `deck.rs`

- `Suit` (pique, cœur, carreau, trèfle), `Rank` (2 à As), `Card` (carte
  naturelle ou `Joker`), constructeurs des paquets 52, 32 et 54 cartes.
- Mélange `ChaCha8Rng` ; tirages en `u32` convertis, jamais de `usize`
  (`CLAUDE.md`, principe Pairs). Test `the_deal_of_a_seed_is_pinned`.
- Donne en parts égales avec les cartes en trop mises de côté.
- `redeal_hidden` : redistribue des cartes cachées entre des mains de tailles
  fixées, en laissant en place les cartes connues de l'observateur.
- Pas d'ordre des valeurs : il appartient à chaque jeu.

### `president/`

`mod.rs` (règles, `Game`), `view.rs`, `determinize.rs`, `heuristics.rs`,
`testing.rs`.

- **`Config`** : `rounds: u8` (1 à 10, défaut 5), `revolution: bool`,
  `no_finish_on_two: bool`, `jokers: Jokers` (`Off | High | Wild`).
- **Actions**, en valeurs et non en cartes puisque les couleurs ne comptent pas :
  - `Play { rank, naturals, jokers }` : `rank` est la valeur jouée (`None` pour
    des jokers seuls), `naturals` le nombre de cartes naturelles, `jokers` le
    nombre de jokers ;
  - `Pass` ;
  - `Give(Vec<Rank>)` : les cartes que rend le Président ou le Vice-Président.
  Le moteur choisit les couleurs à retirer de la main (ordre fixe des
  couleurs), ce qui évite des coups équivalents pour l'IA.
- **Phases** : `Exchange` (le Président et le Vice-Président rendent leurs
  cartes en même temps : `pending` contient les deux sièges ; les cartes du Trou
  du cul et du Secrétaire partent automatiquement) puis `Play`, jusqu'à la fin de
  la manche, puis manche suivante ou fin de partie.
- **Vue** : ma main, triée selon l'ordre courant ; le nombre de cartes de chaque
  joueur ; la combinaison sur la table et son auteur ; qui a passé depuis ;
  titres, scores, numéro de manche ; révolution active ; cartes écartées à la
  donne ; cartes reçues et données à l'échange de la manche, pour les deux
  joueurs concernés ; la config. **Aucune statistique** (ni compte des cartes
  jouées, ni estimation) : règle de l'utilisateur pour tous les jeux de cartes.
- **Déterminisation** : les cartes non vues par l'observateur sont redistribuées
  entre les autres mains, tailles conservées ; celles qu'il a données à
  l'échange restent chez leur destinataire tant qu'elles n'ont pas été jouées.
- **Fin** : `Outcome::Ranked` d'après les totaux.
- **Chrono** : `clock: true` ; `timeout_action` passe, ou rend les cartes les plus
  faibles à l'échange.
- **IA** : ISMCTS générique ; la simulation pose la plus petite combinaison
  légale et passe parfois plutôt que de monter avec un 2 ou un joker.
  `search_actions` reste à l'identique tant que l'arène ne montre pas de
  problème de largeur.

### Enregistrement

`crates/ludo-registry` : un `static` et une entrée dans `GAMES`, la dépendance
dans son `Cargo.toml` et dans le workspace.

## 5. Options de partie (générique)

### Moteur (`ludo-core`)

- Le trait `Game` gagne `fn options() -> Vec<GameOption>`, vide par défaut.
  `GameOption { key, kind }`, avec
  `kind: Toggle { default } | Choice { values, default } | Number { min, max, default }`.
  `key` est le nom du champ du `Config` ; les valeurs de `Choice` sont les noms
  sérialisés des variantes de l'énumération.
- Test générique (conformance) : les valeurs par défaut de `options()` forment
  bien `Config::default()` une fois sérialisées.
- `fn check_config(config, players) -> Result<(), Illegal>`, `Ok` par défaut ;
  `AnyGame::new_match` l'appelle et refuse une config hors bornes.
- `GameMeta` (et donc `listGames()`) inclut les options.
- `ludo arena` accepte `--config '<json>'`.

### Premier utilisateur existant

Ghost Town expose `hands` (`Number`, 1 à `MAX_HANDS`, défaut 5) : le formulaire
est livré et testé avant le Président. Son champ `hands: Option<u8>` devient
`hands: u8` (défaut 5), pour que la valeur par défaut annoncée et
`Config::default()` coïncident ; la désérialisation lit encore `null` (journaux
existants) comme 5.

### Site

- Libellés côté site, dans `web/src/games/options.ts`, indexés par
  `<id du jeu>.<clé>` (et `<id>.<clé>.<valeur>` pour les choix). Le cycle i18n
  traduira ce dictionnaire.
- `web/src/shell/GameOptions.tsx` : formulaire construit depuis les options du
  jeu (case à cocher, liste, champ numérique) ; absent si le jeu n'en a pas.
- Local : réglé avant « Jouer ». En ligne : seul l'hôte règle ; le message
  `lobby` porte la config et les invités voient un résumé en lecture seule.
  `PROTOCOL_VERSION` incrémenté. La config part dans `table.start(seed, config)`.
- La table affiche le résumé des options actives.

## 6. Web du Président

- Famille `"cards"` dans `GameFamily`, étagère « Cartes traditionnelles » après
  « Deck Pairs » ; `Emblem` dessine une carte à jouer portant l'initiale du jeu.
- `web/src/games/cards/PlayingCard.tsx` : une carte (valeur, couleur, joker),
  commune aux futurs jeux classiques.
- `web/src/games/cards/president/` : `types.ts`, `Board.tsx`, rappel de règles
  (`rules.ts`, d'accord avec `RULES.md`, variantes actives incluses).
- Plateau : adversaires avec nombre de cartes, titre et score ; combinaison au
  centre ; bandeau « Révolution » ; ma main en bas. Je sélectionne des cartes ;
  « Jouer » ne s'active que si la sélection correspond à un coup légal fourni
  par le moteur ; si plusieurs coups correspondent (joker wild), je choisis dans
  la liste. « Passer » toujours visible quand il est légal.
- Échange : écran « Rends 2 cartes au Trou du cul » (ou 1 au Secrétaire), qui
  montre les cartes reçues.

## 7. Tests

- `games/cards` : tests de `deck.rs` (paquets, donne figée, cartes écartées,
  `redeal_hidden`) ; `tests/president_rules.rs` : un test par règle et par
  variante de la §3 ; `tests/conformance.rs` de 4 à 7 joueurs, sans jokers et
  avec jokers wild.
- `ludo-core` : options par défaut égales à `Config::default()` ;
  `check_config` refuse une config hors bornes.
- Web : `GameOptions` (rendu des trois types, lecture seule pour l'invité),
  protocole (`lobby` avec config), plateau du Président (`__tests__/`).
- Arène : `ludo arena cards-president 400 hard easy`, puis avec
  `--config '{"revolution":true,"jokers":"Wild"}'`. Attendu : le niveau
  difficile bat nettement le facile.

## 8. Livraison

Deux plans, chacun fusionné sur `main` quand il est vert :

1. **Options de partie** : `ludo-core`, `ludo-wasm`, `ludo arena --config`,
   Ghost Town, `GameOptions`, protocole.
2. **Cartes traditionnelles et Président** : `games/cards` (`deck.rs`,
   `president/`, `RULES.md`), registre, famille et étagère, plateau. Mise à
   jour du `README.md` (liste des jeux) et du `CLAUDE.md` (procédure « Ajouter un
   jeu de cartes classique », ligne d'arène dans « Commandes »).

## 9. Ajustements faits au plan 2

- ❓4 (nouveau, à valider) : la carte interdite se juge selon l'ordre en vigueur quand elle est posée, avant la révolution que ce coup déclenche.
- Le plateau sélectionne des cartes et n'active « Jouer » que si la sélection forme un coup permis ; comme une action désigne une valeur, des naturels et des jokers, une sélection correspond toujours à un seul coup : la liste de choix prévue au §6 pour les jokers wild n'est pas nécessaire.
- L'IA arrête ses simulations à la fin de la manche en cours (`evaluate`) : la donne suivante relève du hasard.
- Mesures de l'arène (200 parties à 4 joueurs) : Difficile 0.87 contre Facile ; 0.85 avec révolution et jokers wild.
- Les modes de jokers (forts, wild) sont livrés avec les règles de base (`combos.rs`), et non avec les variantes : la génération des coups les contient d'emblée.
