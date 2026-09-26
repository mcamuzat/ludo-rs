# ludo-rs — Plateforme de jeux + Yavalath (spec 1)

Date : 2026-09-26
Statut : en relecture

## 1. Objectif

Un site statique unique, en Rust compilé en WASM, qui rassemble des jeux
traditionnels jouables contre l'IA, en local ou en ligne (P2P), et auquel on
ajoute un jeu sans toucher à la plateforme.

Le projet reprend l'architecture éprouvée de `mindbug-rs` (réducteur pur,
pont WASM mince, page React, IA ISMCTS, P2P PeerJS où l'hôte fait autorité)
et la généralise à N jeux et à 2–4 joueurs.

Le travail est découpé en trois cycles, chacun avec sa spec et son plan :

1. **Cette spec** : la plateforme, validée par le **Yavalath** (2 ou 3 joueurs).
2. **Mahjong Riichi** (4 joueurs, information cachée, appels hors tour). Spec
   séparée, qui évaluera les crates existantes de shanten et de scoring
   (maintenance, licence, compilation WASM).
3. Les jeux suivants.
4. **Serveur optionnel** (API, comptes, classements, tournois). Hors de cette
   spec, mais préparé par le journal de partie (§3.1) et par l'interface
   `Session` (§6).

La plateforme est conçue dès maintenant pour le cas le plus dur du cycle 2 :
information cachée, 4 joueurs, décisions de plusieurs joueurs en même temps.

### Critères de réussite

- On joue au Yavalath à 2 et à 3 joueurs, contre 0 à 2 IA, en local ou en ligne
  avec des sièges mixtes humains/IA.
- Ajouter un jeu revient à écrire un crate `games/<id>`, un composant
  `web/src/games/<id>/Board.tsx`, et une ligne dans chacun des deux registres
  (Rust et web). Aucun autre fichier de la plateforme ne change.
- Le niveau Difficile bat le niveau Facile dans au moins 80 % des parties
  (commande `arena`, 200 parties, couleurs alternées).
- Le site se publie sur GitHub Pages, sans backend.

### Hors périmètre

Mahjong (cycle 2), relais TURN, migration d'hôte, chat, et tout le serveur
(cycle 4) : API, comptes, base de données, matchmaking public, classements
(Elo/Glicko), tournois, graine partagée par engagement/révélation.

## 2. Structure du dépôt

```
ludo-rs/
  Cargo.toml              workspace
  rust-toolchain.toml     stable + rustfmt, clippy, wasm32-unknown-unknown
  CLAUDE.md               principes et commandes
  crates/
    ludo-core/            trait Game, Seat, Outcome, IA générique, conformance
    ludo-wasm/            pont wasm-bindgen + registre des jeux
    ludo-cli/             binaire : `arena`, `play` (partie en terminal)
  games/
    yavalath/             crate du jeu + RULES.md
  web/                    React + Vite + TypeScript
    src/
      shell/              accueil, lobby, cadre de partie
      session/            LocalSession, HostSession, GuestSession, protocole
      workers/            engine.worker.ts, ai.worker.ts
      games/
        registry.ts       { id, Board } par jeu
        yavalath/Board.tsx
  scripts/build-wasm.sh
  .github/workflows/      CI + Pages
```

## 3. Le contrat `Game` (`ludo-core`)

Chaque jeu est un **réducteur pur** : pas d'I/O, pas d'horloge, et le hasard
vient d'une graine stockée dans l'état.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Seat(pub u8);

pub enum Viewer { Seat(Seat), Spectator }

pub struct GameMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub min_players: u8,
    pub max_players: u8,
    pub hidden_info: bool,
}

/// Sièges qui doivent décider maintenant. Plusieurs sièges quand le jeu
/// attend des réponses simultanées (réclamations au Mahjong).
pub struct Pending(pub Vec<Seat>);

pub enum Outcome {
    /// Rang par siège (0 = premier) ; égalité = même rang.
    Ranked(Vec<u8>),
}

pub struct Illegal(pub String);

pub trait Game: 'static {
    type State: Clone + Serialize + DeserializeOwned;
    type Action: Clone + Eq + Hash + Serialize + DeserializeOwned;
    type View: Serialize;
    type Config: Clone + Serialize + DeserializeOwned + Default;

    fn meta() -> GameMeta;
    fn new(config: &Self::Config, players: u8, seed: u64) -> Self::State;
    fn pending(s: &Self::State) -> Pending;          // vide ⇔ partie finie
    fn legal_actions(s: &Self::State, seat: Seat) -> Vec<Self::Action>;
    fn apply(s: &mut Self::State, seat: Seat, a: &Self::Action) -> Result<(), Illegal>;
    fn view(s: &Self::State, viewer: Viewer) -> Self::View;
    fn outcome(s: &Self::State) -> Option<Outcome>;
}

pub trait Determinize: Game {
    /// Rebat tout ce que `observer` ne voit pas, de façon cohérente avec
    /// ce qu'il sait. Identité pour un jeu à information parfaite.
    fn determinize(s: &Self::State, observer: Seat, rng: &mut impl Rng) -> Self::State;
}

pub trait Heuristics: Game {
    fn rollout_action(s: &Self::State, seat: Seat, rng: &mut impl Rng) -> Self::Action;
    fn evaluate(_s: &Self::State) -> Option<Vec<f32>> { None }
}
```

Règles du contrat :

- **`apply` renvoie `Illegal`, ne panique jamais.** L'hôte vérifie ainsi toute
  action d'un invité ou de l'IA.
- **Décisions simultanées.** Quand `pending` contient plusieurs sièges, chacun
  soumet son action. Le jeu les met en attente et ne tranche qu'une fois toutes
  les réponses reçues. Le résultat ne dépend pas de l'ordre d'arrivée.
  L'arbitrage (par exemple ron > pon/kan > chi) appartient au jeu.
- **`view` est la seule chose qui sort de l'hôte vers un joueur.** Elle ne
  contient ni la graine, ni les données cachées d'un autre siège.
- **Les valeurs dérivées dont l'UI a besoin** (cases imposées, alignements à
  surligner, plus tard shanten ou attentes) sont calculées par le jeu et
  placées dans `View`. Le frontend ne calcule rien.
- **Effacement de type.** Un unique `impl<G: Game + Determinize + Heuristics> AnyGame for G`
  expose le jeu en JSON (`serde_json`) à `ludo-wasm` et `ludo-cli`. Le registre
  Rust est une liste statique : ajouter un jeu = une ligne.
- **Portabilité native.** `ludo-core` et les crates `games/*` ne dépendent
  jamais de `wasm-bindgen`, `web-sys` ni d'aucune API navigateur. Ils
  compilent en natif, pour que `ludo-cli` et un futur serveur les réutilisent
  tels quels. Seul `ludo-wasm` dépend de wasm-bindgen.

### 3.1 Journal de partie

```rust
pub struct MatchLog<G: Game> {
    pub game_id: String,
    pub config: G::Config,
    pub players: u8,
    pub seed: u64,
    pub actions: Vec<(Seat, G::Action)>,   // uniquement les `apply` réussis, dans l'ordre
}

pub fn replay<G: Game>(log: &MatchLog<G>) -> Result<G::State, Illegal>;
```

- `Match<G>` (dans `ludo-core`) regroupe l'état et son journal ; c'est lui que
  le Worker moteur détient, et chaque `apply` réussi y ajoute l'action.
- `replay` recrée l'état depuis `new(config, players, seed)` et réapplique
  chaque action ; il renvoie `Illegal` à la première action refusée.
- Usages : sauvegarde/reprise de l'hôte (on stocke le journal, pas l'état),
  débogage, futur mode « revoir la partie », et vérification côté serveur
  d'un résultat envoyé par un client (cycle 4).
- Exposé en JSON par `ludo-wasm` (`export_log(h)`, `import_log(game_id, json)`).

## 4. L'IA (`ludo-core::ai`)

- **MCTS (UCT)** quand `meta.hidden_info == false`, **ISMCTS** sinon. Les deux
  partagent le même code ; seule change la détermination en début d'itération
  (`determinize` depuis le siège de l'IA).
- **Récompenses par siège (max-n).** Chaque nœud stocke un vecteur de
  récompenses issu de `Outcome` (1 pour le premier rang, 0 pour le dernier,
  interpolation linéaire entre les deux). Chaque siège maximise la sienne.
- **Décisions simultanées en simulation.** Les sièges de `pending` décident
  un par un dans l'ordre croissant de `Seat`.
- **Simulation.** Par défaut `Heuristics::rollout_action`. Si `evaluate`
  renvoie une valeur, la simulation s'arrête au-delà de 200 coups et utilise
  cette évaluation.
- **Niveaux.** Seul le budget change : Facile 100 ms, Moyen 1 s, Difficile
  3 s. Le niveau Facile joue en plus un coup aléatoire parmi les légaux avec
  une probabilité de 15 %. En CLI et dans les tests, le budget s'exprime en
  itérations pour être reproductible.
- **Déterminisme.** À graine et budget en itérations égaux, l'IA choisit
  toujours le même coup.
- **Non-triche.** L'IA reçoit l'état complet mais commence toujours par
  `determinize` ; un test vérifie qu'elle ne dépend pas des données cachées
  des autres sièges (voir §9).

## 5. WASM et Workers

`ludo-wasm` (wasm-bindgen) expose, tout en JSON :

| Fonction | Rôle |
|---|---|
| `list_games()` | `GameMeta` de chaque jeu |
| `new_match(game_id, config, players, seed)` | crée une partie, renvoie un handle |
| `pending(h)` | sièges à décider |
| `legal_actions(h, seat)` | actions légales du siège |
| `apply(h, seat, action)` | applique ou renvoie `Illegal` |
| `view(h, viewer)` | vue d'un siège ou spectateur |
| `outcome(h)` | résultat si fini |
| `export_state(h)` | copie de l'état pour le Worker IA |
| `export_log(h)` / `import_log(game_id, json)` | sauvegarde / reprise par le journal (§3.1) |
| `ai_move(game_id, state, seat, budget_ms, seed)` | coup de l'IA |

Aucune logique n'y vit.

Deux Workers :

- **Worker moteur** : détient l'état de la partie. Rapide, ne cherche jamais.
- **Worker IA** : reçoit `export_state` + siège + budget, renvoie une action,
  que le Worker moteur valide via `apply` comme une action humaine.

Une recherche longue ne bloque donc jamais le traitement des messages réseau.

## 6. Sessions et réseau

### Interface unique pour l'UI

```ts
interface Session {
  seats: SeatInfo[];
  me: Seat | null;                        // null en spectateur / hot-seat
  onUpdate(cb: (u: Update) => void): () => void;
  submit(seat: Seat, action: unknown): void;
  close(): void;
}
```

`LocalSession` (IA et/ou hot-seat), `HostSession` et `GuestSession`
l'implémentent. L'UI ne sait pas lequel elle utilise. Un futur mode où le
serveur fait autorité (cycle 4) sera une `ServerSession` WebSocket de plus,
sans changement dans l'UI.

### P2P en étoile, hôte faisant autorité (PeerJS)

- L'hôte crée une salle et obtient un **code court** (6 caractères), préfixé
  `ludo-` dans l'identifiant PeerJS pour éviter les collisions.
- Les invités se connectent **uniquement à l'hôte**.
- Signalisation : serveur public PeerJS. ICE : STUN public uniquement.
  Limite connue : sans TURN, certaines connexions (NAT symétrique, CGNAT)
  échouent ; l'UI l'affiche clairement.
- Messages (JSON, `PROTOCOL_VERSION` refusé si différent) :

| Message | Sens | Contenu |
|---|---|---|
| `Hello` | invité → hôte | `protocol`, `game_id`, `name`, `seat_token?` |
| `Lobby` | hôte → invité | sièges, config, code |
| `Seated` | hôte → invité | `seat`, `seat_token` |
| `Update` | hôte → invité | `rev`, `view`, `legal_actions`, `pending` **de ce siège** |
| `Action` | invité → hôte | `rev`, `action` |
| `Rejected` | hôte → invité | `reason` |
| `Ping` / `Pong` | les deux | latence |

- `rev` s'incrémente à chaque `apply` réussi ; une `Action` portant un `rev`
  périmé est rejetée, un `Update` périmé est ignoré.
- **Reconnexion** : un invité qui revient avec son `seat_token` retrouve son
  siège et reçoit un `Update` complet. Son siège reste réservé pendant la
  partie ; les autres continuent d'attendre s'il doit jouer.
- **Départ de l'hôte** : la partie s'arrête. L'hôte sauvegarde le journal de
  partie et les jetons dans `localStorage` après chaque `apply` ; il peut reprendre avec le
  même code et les invités se reconnectent.

### Sièges

- Le lobby affiche `meta.max_players` sièges. Chaque siège est `Ouvert`,
  `Humain(nom)`, `IA(niveau)` ou `Fermé`.
- La partie démarre quand le nombre de sièges non fermés est dans
  `[min_players, max_players]` et qu'aucun n'est `Ouvert`.
- En ligne, l'hôte fait jouer les sièges IA dans son Worker IA.
- **Hot-seat** (plusieurs humains sur un écran) : proposé seulement si
  `meta.hidden_info == false`.

## 7. Shell web

- React + Vite + TypeScript, `pnpm`.
- **Accueil** : grille des jeux présents à la fois dans `list_games()` et dans
  `web/src/games/registry.ts`.
- **Lobby générique** : sièges, options du jeu (chaque jeu fournit un
  composant `ConfigForm` pour son `Config`), boutons « Jouer en local »,
  « Héberger », champ « Rejoindre ».
- **Cadre de partie commun** : joueurs, siège à qui c'est le tour, statut
  réseau et latence, écran de fin (classement), revanche.
- **Plateau par jeu** : `Board({ view, legalActions, pending, me, submit })`.
  Il affiche la vue et envoie des actions ; il ne calcule rien.
- Types TypeScript de chaque jeu dans `web/src/games/<id>/types.ts`, mis à
  jour dans le même changement que les types Rust sérialisés.

## 8. Yavalath (`games/yavalath`)

Source : le livret officiel nestorgames (règles de Cameron Browne / Ludi),
<https://nestorgames.com/rulebooks/YAVALATH_EN.pdf>. Chaque règle de
`RULES.md` cite **[R]** ; un point non tranché est marqué ❓.

### Règles [R]

- Plateau hexagonal de **61 cases** (côté 5), vide au départ.
- Couleurs : Blanc, Noir, et Rouge pour le 3ᵉ joueur. Blanc commence.
- À son tour, on pose une pierre de sa couleur sur une case vide.
- **Swap** (2 joueurs) : au premier coup de Noir, il peut échanger les
  couleurs au lieu de poser.
- **Victoire** : aligner 4 pierres ou plus de sa couleur.
- **Défaite** : aligner 3 pierres sans en aligner 4 par le même coup. Former
  3 et 4 d'un même coup est donc une victoire.
- **Nul** : le plateau est plein sans vainqueur ni perdant.
- **3 joueurs** :
  - on doit bloquer la victoire du joueur suivant si c'est possible ;
  - un joueur qui aligne 3 sans 4 est éliminé, ses pierres restent ;
  - gagne le dernier survivant ou le premier qui aligne 4.

### Choix sur les points non tranchés ❓

1. **Pas de swap à 3 joueurs.** Le livret ne le décrit que pour Noir à 2.
2. **Une victoire immédiate prime sur le blocage forcé** : un coup qui aligne
   4 reste légal même s'il ne bloque pas, puisque la partie s'arrête.
3. **Blocage impossible** : si le joueur suivant a plusieurs cases gagnantes,
   on doit jouer sur l'une d'elles ; s'il n'en a aucune, pas de contrainte.
4. **« Joueur suivant » à 3** : le prochain joueur non éliminé dans l'ordre
   du tour.
5. **Après une élimination à 3**, les deux restants continuent selon les
   règles à 2 joueurs (sans swap), blocage forcé compris envers le suivant.
6. **Classement à 3** : le vainqueur 1ᵉʳ ; les éliminés classés dans l'ordre
   inverse de leur élimination ; en cas de nul entre deux survivants, ils
   partagent le rang.

### Modèle

- Coordonnées axiales `(q, r)` avec `max(|q|, |r|, |q+r|) ≤ 4`.
- `Action = Place(Cell) | Swap`.
- `View` : plateau, couleurs des sièges, siège actif, sièges éliminés,
  `forced: Vec<Cell>` (cases imposées par le blocage), `last_move`, et en fin
  de partie `lines: Vec<Vec<Cell>>` (alignements à surligner).
- `Determinize` : identité. `Heuristics::rollout_action` : gagner si possible,
  sinon bloquer si imposé, sinon éviter les coups qui alignent 3 sans 4,
  sinon aléatoire.

## 9. Tests

- **Règles Yavalath** (Rust) : 3 et 4 du même coup, défaite sur 3, nul plateau
  plein, swap, blocage forcé à 3, victoire immédiate prioritaire sur le
  blocage, élimination avec pierres conservées, ordre du « joueur suivant »
  après élimination.
- **Conformance générique** `ludo_core::conformance::check::<G>()`, que tout
  jeu doit passer :
  - 500 parties aléatoires jouées jusqu'au bout sans erreur ;
  - chaque action de `legal_actions` est acceptée par `apply` ; une action
    hors de la liste est refusée ;
  - aller-retour serde de `State` et `Action` ;
  - `replay` du journal de chaque partie aléatoire redonne exactement l'état
    final (comparé après sérialisation) ; un journal contenant une action
    illégale est refusé ;
  - pour les jeux `hidden_info` : `view(Seat(i))` est identique pour deux
    déterminisations depuis le siège `i` (rien de caché n'y fuit).
- **IA** : `ludo-cli arena` (Difficile ≥ 80 % contre Facile) ; test de
  déterminisme ; pour les jeux `hidden_info`, test de non-triche (même coup
  quelle que soit la déterminisation de départ).
- **Web** : `tsc` ; Vitest sur les sessions (rejet d'un `rev` périmé,
  reconnexion par `seat_token`). Test P2P manuel à deux ou trois onglets.
- **CI** : `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`,
  `cargo test --workspace`, build WASM, `pnpm build` ; déploiement GitHub
  Pages sur `main`.
