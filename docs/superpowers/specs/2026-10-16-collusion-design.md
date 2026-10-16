# ludo-rs — Collusion (spec 6)

Date : 2026-10-16
Statut : à valider
Précédent : `2026-10-15-president-design.md` (famille « Cartes traditionnelles »)

## 1. Objectif

Ajouter **Collusion** (David Parlett, aussi appelé « Collude », « Cahoots »,
« Dangerous Liaisons ») à la famille « Cartes traditionnelles ». Collusion est
un jeu de plis à 4 où l'on gagne en finissant avec **exactement autant de plis
qu'un autre joueur**. On y négocie à voix haute : « je vise quatre », « qui
part sur trois ? ».

L'utilisateur veut pouvoir parler à table avec des **phrases toutes faites**,
au minimum annoncer le nombre de plis qu'on vise. Il veut d'abord voir comment
le jeu tourne avec une IA **simple**, qui ne lit pas les annonces des autres.
Les alliances calculées viendront dans une seconde étape, si le jeu le mérite.

### Hors périmètre

- demander une couleur ou une hauteur d'entame (« mène-moi un cœur bas ») ;
- une IA qui lit les annonces des autres pour s'allier (seconde étape, comme
  pour Captain's Orders) ;
- le chat libre ;
- Dumbo, l'équivalent à 3 joueurs ;
- les animations et les sons.

## 2. Source

- **[C]** David Parlett, « Collusion », `parlettgames.uk/oricards/collude.html`,
  consultée le 2026-10-16. C'est la seule source.

`games/cards/RULES.md` gagne une section Collusion qui cite [C] pour chaque
règle. Les points que [C] ne tranche pas sont marqués ❓.

## 3. Règles (`cards-collusion`, 4 joueurs exactement)

- **Matériel** [C] : paquet de 52 cartes ; dans chaque couleur, A R D V 10 9 8
  7 6 5 4 3 2.
- **Donne** [C] : 13 cartes à chacun, une à une.
- **Donneur** ❓ : le siège 0 donne la première donne, puis la donne passe à
  gauche à chaque donne (usage courant, non dit par [C]).
- **Jeu** [C] : le joueur à gauche du donneur entame le premier pli, le
  gagnant de chaque pli entame le suivant. Il faut fournir à la couleur si on
  peut, sinon on joue n'importe quelle carte. Il n'y a pas d'atout : la plus
  forte carte de la couleur demandée gagne.
- **Score d'une donne** [C] : un point par pli, plus au plus un bonus :
  - si exactement deux joueurs ont le même nombre de plis, chacun d'eux
    gagne 10 ;
  - si aucun joueur n'a le même nombre de plis qu'un autre, celui qui en a le
    moins gagne 20 ;
  - si trois joueurs ont le même nombre de plis, le quatrième gagne 30.

  Comme 13 est impair, il ne peut y avoir ni deux paires ni quatre joueurs à
  égalité. Ces trois cas couvrent donc toutes les donnes, et le plus petit
  total est toujours unique quand tous diffèrent.
- **Cible** [C] : 100 points, ou 50 en option (« la plupart des critiques
  préfèrent 50 »). On ne peut atteindre la cible qu'avec un bonus. Un joueur
  sans bonus dont `total + plis` atteindrait la cible **retranche** ses plis de
  son total au lieu de les ajouter. [C] le dit pour 100 (« into the
  hundreds ») ; on l'applique à la cible choisie.
- **Fin de partie** ❓ : après chaque donne, si au moins un joueur a atteint la
  cible, la partie s'arrête. Le classement se fait par total, et les ex æquo
  partagent leur rang. [C] ne dit pas qui gagne si plusieurs joueurs
  franchissent la cible ensemble : on prend le plus haut total.

### Parler

[C] permet de parler à tout moment, sans rien révéler de sa main. Les accords
ne lient personne. Sur le site, on parle avec des phrases toutes faites, et
**seul le joueur dont c'est le tour parle** (le moteur ne fait agir que les
sièges en attente). La règle devient donc : à ton tour, avant de poser ta
carte, tu peux dire **une phrase**, puis tu joues.

Les phrases :

- « Je vise N plis » : N va du nombre de plis déjà pris à ce nombre plus les
  plis qui restent ;
- « Je ne sais pas encore » ;
- « Qui vise N ? » : N va de 0 à 13.

Une annonce ne lie personne : on peut viser 4 et jouer tout autre chose, et le
bluff fait partie du jeu. Les phrases d'une donne s'effacent à la donne
suivante.

## 4. Moteur : `games/cards/src/collusion/`

Le module suit la procédure « Ajouter un jeu de cartes classique » du
`CLAUDE.md` : `mod.rs`, `view.rs`, `determinize.rs`, `heuristics.rs`,
`testing.rs`. Le paquet et la donne viennent de `deck.rs` (`pack_52`,
`deal_evenly`).

- **`Config`** : `target: u32`, `Number` de 50 à 100 et 100 par défaut. ❓ On
  pourrait aussi proposer un choix entre deux valeurs seulement (`Choice`
  50 / 100). Le builder choisit la forme la plus simple qui passe par
  `Game::options`.
- **Actions** :
  - `Play { card }` ;
  - `Say(Phrase)`, avec `Phrase = Aim(u8) | Unsure | WhoAims(u8)`. Cette
    action est légale une seule fois par tour, avant `Play`, et **seulement si
    le joueur a au moins deux cartes jouables**. Le joueur reste en attente
    après avoir parlé. Avec une seule carte jouable, le site la joue donc
    tout seul (`Table.autoPlay` ne joue que lorsqu'il n'existe qu'un coup
    légal), au 13e pli en particulier. Dans ce cas, il n'y a rien à
    négocier pour ce coup.
  - Chrono : dire une phrase relance le temps du tour, comme toute action. Le
    tour peut ainsi durer jusqu'à deux fois le temps de base, puisqu'on ne
    parle qu'une fois. On l'accepte, pour ne pas toucher au code générique
    du chrono.
- **Vue** : ma main triée par couleur puis par valeur ; le pli en cours, avec
  qui a posé quoi ; le dernier pli ramassé ; le nombre de plis de chacun ; les
  totaux, la cible, le donneur, le numéro de donne ; la dernière phrase de
  chacun et le fil des phrases de la donne ; le détail du score de la donne
  précédente (plis, bonus, retranché ou non) ; les coups légaux.
  **Aucune statistique** : ni compte des cartes tombées, ni probabilité. C'est
  la règle de l'utilisateur pour tous les jeux de cartes.
- **Déterminisation** : les cartes non vues par l'observateur sont
  redistribuées entre les trois autres mains, tailles conservées, **en
  respectant les coupes**. Un joueur qui n'a pas fourni à une couleur n'en
  reçoit plus. Le `redeal` actuel de `deck.rs` ne sait pas tenir de
  contrainte : l'étendre dans `deck.rs` (un `redeal` sous contraintes,
  réutilisable par les prochains jeux de plis) plutôt que de l'écrire dans
  le module.
- **Fin** : `Outcome::Ranked` d'après les totaux.
- **Chrono** : `clock: true`. `timeout_action` joue la plus petite carte
  légale, sans rien dire.

### IA (version simple)

- La recherche (ISMCTS générique) ne choisit que la carte : `search_actions`
  et `rollout_action` ne proposent jamais `Say`. Sinon la recherche tournerait
  en rond sur des paroles sans effet sur le score.
- La simulation joue une carte légale plausible, au choix du builder (par
  exemple la plus petite carte de la couleur, ou la plus forte quand on est
  dernier et qu'on veut le pli). `evaluate` arrête la simulation à la fin de
  la donne, comme au Président : la donne suivante relève du hasard.
- **Annonces de l'IA** : à son tour, avant de chercher sa carte, l'IA dit
  « Je vise N » quand sa cible estimée a changé depuis sa dernière annonce.
  Elle ne parle pas sinon : pas de phrase à chaque tour, et pas de « Qui
  vise ? ». Elle est **sincère**, puisqu'elle annonce ce qu'elle croit faire,
  mais **libre** : elle change d'avis quand la donne tourne. Elle ne tient pas
  compte des annonces des autres.

  L'estimation est au choix du builder. Un compte simple des levées probables
  (plis pris plus cartes maîtresses ou presque) suffit pour cette étape.
- **Accroche dans `ludo-core`** : le moteur d'IA ne joue aujourd'hui que ce
  que la recherche choisit. Il faut une accroche générique dans
  `Heuristics`, par exemple `fn scripted_action(s, seat) -> Option<Action>`
  (le nom est au choix du builder), que l'IA joue sans chercher quand elle
  renvoie `Some`. Par défaut elle renvoie `None`, ce qui ne change aucun autre
  jeu. Collusion y renvoie la phrase à dire. Comme `Say` laisse le joueur en
  attente, l'IA est rappelée et cherche alors sa carte. Dans `ai::choose`,
  l'accroche passe **avant** le coup au hasard de Facile.
- **Coup au hasard de Facile** : aujourd'hui, `ai::choose` tire 15 % du
  temps un coup parmi tous les coups légaux. À Collusion, ce tirage
  tomberait surtout sur des phrases, qui sont jusqu'à 29 contre 13 cartes au
  plus. Une seconde accroche de `Heuristics`, par exemple
  `fn random_actions(s, seat) -> Vec<Action>`, fixe les coups parmi lesquels
  ce tirage se fait. Par défaut elle renvoie `legal_actions`, ce qui ne
  change ni les autres jeux ni leurs arènes. Collusion y renvoie ses seules
  cartes jouables.

### Enregistrement

`crates/ludo-registry` : un `static` et une entrée dans `GAMES`.

## 5. Web : `web/src/games/cards/collusion/`

`types.ts`, `Board.tsx` (cartes : `../PlayingCard`), `rules.ts` (d'accord
avec `RULES.md`), un test dans `__tests__/`, une entrée `family: "cards"` dans
`web/src/games/registry.ts`, et le libellé de l'option dans
`web/src/games/options.ts`. Le nom affiché est « Collusion », le nom actuel
choisi par Parlett.

- **Table** : quatre sièges autour du tapis, le pli en cours au centre, ma
  main en bas. Sous chaque siège : son nombre de plis, son total et le badge
  donneur.
- **Bulles** : la dernière phrase de chaque joueur s'affiche en bulle près de
  son siège, et reste là jusqu'à sa phrase suivante ou jusqu'à la fin de la
  donne. Un fil replié montre toutes les phrases de la donne.
- **Parler** : à mon tour, un bouton « Parler » ouvre les trois phrases. Pour
  « Je vise » et « Qui vise ? », je choisis N dans les valeurs que le moteur
  propose. Une fois que j'ai parlé, le bouton disparaît jusqu'à mon tour
  suivant.
- **Jouer** : un clic sur une carte légale la joue. Les cartes illégales sont
  grisées.
- **Fin de donne** : un récapitulatif montre les plis, le bonus et les plis
  retranchés de chacun, puis les nouveaux totaux.

## 6. Tests

- `tests/collusion_rules.rs` : obligation de fournir ; gagnant du pli sans
  atout ; entame du gagnant ; rotation du donneur ; les trois bonus ;
  retranchement sans bonus à l'approche de la cible, pour 100 et pour 50 ;
  arrivée sur la cible avec un bonus ; fin de partie et ex æquo ; `Say` une
  fois par tour, avant `Play`, avec les bornes de N ; phrases effacées à la
  donne suivante.
- `deck.rs` : le `redeal` sous contraintes respecte les coupes et les tailles.
- `ludo-core` : l'accroche d'IA par défaut ne change rien ; une IA qui la
  remplit joue l'action donnée, puis cherche.
- `tests/conformance.rs` : un cas à 4 joueurs, cible 100 et cible 50.
- Web : plateau (bulles, bouton « Parler », cartes grisées).
- Arène : `ludo-cli arena cards-collusion 200 hard easy 4`, avec un
  attendu ≥ 0.55. Ajouter la ligne mesurée dans « Commandes » du `CLAUDE.md`.

## 7. Livraison

Un seul plan, fusionné sur `main` quand il est vert : accroche d'IA dans
`ludo-core`, `redeal` sous contraintes, module `collusion`, `RULES.md`,
registre, plateau, `README.md` (liste des jeux) et `CLAUDE.md` (arène).

## 8. Choix faits sans l'utilisateur

- Le donneur passe à gauche à chaque donne (❓).
- Plusieurs joueurs sur la cible : le plus haut total gagne (❓).
- Le retranchement s'applique à la cible choisie, 50 comme 100.
- On ne parle qu'à son tour, une phrase par tour.
- L'IA n'annonce que « Je vise N », et seulement quand sa cible change.
- On ne peut pas parler quand on n'a qu'une carte jouable. Le site joue
  alors cette carte tout seul.
- Dire une phrase relance le chrono du tour, une fois au plus.
- Deux accroches génériques d'IA dans `ludo-core` (phrase scriptée, coups du
  tirage au hasard de Facile), sans effet sur les autres jeux.
