# ludo-rs — TwixT

Date : 2026-11-06
Statut : conçu avec l'utilisateur (conversation du 2026-11-06). Ses décisions sont marquées *(utilisateur)* ; les choix faits sans lui, *(choix du site)*.

## 1. Intention

L'utilisateur possède TwixT et ne trouve aucune IA assez forte : toutes celles qu'il a essayées sont faibles. Il veut deux choses à égalité :

1. **une IA forte**, celle de twixtbot (style AlphaZero : réseau de neurones + MCTS), qui jouait au niveau des meilleurs humains sur Little Golem ;
2. **un coach chiffré** sur le modèle de Reversiology et Backgammonics (github.com/KilledByAPixel). *(Utilisateur)* « Rien qu'un retour de chiffre serait énorme » : pas d'explication en phrases, le réseau juge.

Réussite : l'utilisateur perd contre Difficile, et peut voir après coup où il a perdu la partie.

### Non-objectifs

- Raisons écrites des notes (« tu laisses passer son chemin ») : écartées *(utilisateur)*.
- Règle d'origine 3M avec retrait de liens : écartée, la règle Little Golem suffit *(utilisateur)*.
- Entraîner ou améliorer le réseau : on reprend les poids tels quels.
- Autres tailles de plateau que 24×24 (le réseau n'a vu que celle-là).
- Énigmes, ouvertures nommées : pas demandés.

### Anti-objectifs

- Un coach qui s'affiche sans qu'on le demande : *(utilisateur)* les chiffres sont **à la demande**, pour ne pas être influencé.
- Un « TwixT » dont l'IA est faible : si le réseau ne peut pas tourner dans le navigateur à une vitesse jouable, il faut le dire avant de livrer, pas retomber en silence sur l'ISMCTS générique.
- Alourdir le chargement de tout le site de plusieurs Mo pour un seul jeu.

## 2. Sources

- **[LG]** Règle TwixT « PP » (papier-crayon) jouée sur Little Golem : la page d'aide du site ne répond plus (410) ; la décrire d'après Wikipédia (en.wikipedia.org/wiki/TwixT, section sur les règles PP) et le code de [TB], à citer dans `RULES.md`. Le livret de la boîte de l'utilisateur peut servir pour le reste.
- **[TB]** twixtbot, Jordan Lampe, licence MIT : github.com/BonyJordan/twixtbot. Moteur de règles (`twixt.py`), MCTS (`nnmcts.py`), modèle de swap (`swapmodel.py`).
- **[UI]** twixtbot-ui, stevens68, licence MIT : github.com/stevens68/twixtbot-ui. Réseau converti en PyTorch : `model/torch.pt` (≈ 10 Mo), architecture dans `src/backend/torchmodel.py`, encodage des entrées dans `src/backend/naf.py`, évaluation dans `nneval.py`. C'est la référence numérique du portage.
- Les deux licences MIT et les auteurs vont dans `NOTICE`.

## 3. Règles (`twixt`, 2 joueurs, [LG])

- Plateau de 24×24 trous sans les quatre coins. Rouge (siège 0, commence *(choix du site)*) relie le haut au bas, Noir la gauche à la droite. Les rangées de bord haut/bas appartiennent à Rouge, les colonnes gauche/droite à Noir : on ne pose pas dans la bordure de l'adversaire.
- À son tour, on pose un pion de sa couleur dans un trou libre. Tout pion ami à distance de cavalier est relié automatiquement, sauf si le lien croiserait un lien **adverse**. Ses propres liens peuvent se croiser (règle « PP » de Little Golem, et celle sur laquelle le réseau a été entraîné : [TB] `twixt.py` ne teste que les liens adverses). Deux liens qui se croisent ne sont pas reliés au croisement. Les liens ne se retirent jamais.
- **Swap** : au premier coup du second joueur, il peut prendre le premier pion à son compte au lieu de jouer (comme [TB] : le pion change de couleur et est reflété par la diagonale ; vérifier contre [LG] et l'écrire dans `RULES.md`).
- Victoire : une chaîne de liens relie ses deux bords.
- **Nulle** : *(choix du site, comme [TB])* dès qu'aucun des deux ne peut plus relier ses bords. Pas de limite de durée : la partie finit toujours.
- Abandon : comme sur les autres jeux du site s'il existe déjà ; sinon pas de nouveauté ici.

## 4. Moteur

- Crate `games/twixt` : règles, `view.rs`, `heuristics.rs`, `net.rs` (réseau), `mcts.rs` (recherche guidée), `RULES.md`, tests de règles et `tests/conformance.rs`.
- Actions : `Place { x, y }` et `Swap`.
- Vue : pions, liens, joueur au trait, swap possible, résultat ; à la fin, la chaîne gagnante.
- Information complète : `shared_screen()` vrai, « Humain (même écran) » proposé, et le jeu en ligne marche comme Black Hole.
- `games/*` restent natifs (CLAUDE.md du dépôt) : le réseau est du Rust pur, sans `wasm-bindgen`.

### 4.1 Réseau

- Portage en Rust de `TwixtNet` [UI] : 40 canaux, 12 blocs résiduels de convolutions 5×5 (activation `abs`), tête de politique (une valeur par case) et tête de valeur (probabilité de gagner). Normalisations par lot fusionnées dans les poids au chargement. Encodage des entrées et retournement quand Noir est au trait exactement comme `naf.py`.
- **Exactitude** : un script hors exécution (Python + PyTorch, dans `scripts/`) exporte les poids dans un format simple et un jeu de positions de référence avec les sorties PyTorch. Un test Rust doit retrouver politique et valeur à ~1e-4 près. C'est le premier jalon : tant qu'il échoue, rien d'autre.
- **Poids hors du paquet principal** : le chargement du site ne doit pas grossir de ~4 Mo. Au builder de choisir le moyen (poids en float16 ou quantifiés si l'exactitude reste suffisante, fichier chargé seulement à l'ouverture de TwixT, etc.). Le moteur reçoit les poids en octets ; les règles ne dépendent jamais d'eux (le rejeu d'un `MatchLog` n'en a pas besoin).
- **Vitesse** : ~0,5 milliard de multiplications par évaluation. Mesurer tôt le nombre d'évaluations par seconde dans le navigateur (worker existant, `web/src/engine/clients.ts`). SIMD wasm souhaitable. Le site est sur GitHub Pages (pas d'en-têtes COOP/COEP, donc pas de fils wasm) : remèdes permis *(choix du site)* : SIMD, poids en float16/int8, plusieurs workers en parallèle. WebGPU, `coi-serviceworker` ou une inférence en JS (onnxruntime-web) seulement après en avoir parlé à l'utilisateur.

### 4.2 IA

- L'IA de TwixT n'utilise pas l'ISMCTS générique : MCTS guidé par le réseau (PUCT, comme `nnmcts.py`). Elle passe par un crochet de `Heuristics` ; si `policy_action` ne suffit pas (il ne reçoit ni budget ni horloge), l'étendre de façon générique dans `ludo-core` plutôt que de contourner.
- Niveaux : les trois du site, Facile / Moyen / Difficile *(choix du site ; quatre avaient été évoqués, trois suffisent sans toucher à `Level`)*. Facile : politique seule avec un tirage pondéré assez chaud ; Moyen : politique seule, meilleur coup ; Difficile : MCTS d'environ 10 secondes par coup ou plus *(utilisateur : « plus de 10 secondes me va » si c'est le prix d'une IA forte)* ; budget exact réglé d'après la vitesse mesurée, avec une indication « réfléchit… » visible.
- Swap de l'IA : par la valeur du réseau, ou par le modèle de [TB] (`swapmodel.py`) si plus simple.
- `timeout_action` : meilleur coup de la politique, ou première case libre si les poids ne sont pas chargés.
- Arène : Difficile ≥ 0,80 contre Facile, et Moyen > Facile. Garder une poignée de parties : chaque évaluation coûte cher.

### 4.3 Coach (à la demande)

Calculé par le moteur (fonction exposée par `ludo-wasm`, sans logique dans le web), pour une position donnée :

- probabilité de gagner pour le joueur qui demande ;
- note de son dernier coup d'après la chute de cette probabilité : Meilleur / Bon / Imprécision / Erreur / Gaffe *(seuils : choix du builder, documentés dans `RULES.md` ou la spec)* ;
- les 3 meilleurs coups, avec leur probabilité de gagner ;
- les valeurs de politique de toutes les cases, pour une carte de chaleur.

Le coach se sert de la même évaluation que Difficile (valeur et politique, plus une courte recherche si le temps le permet).

## 5. Web

- `web/src/games/twixt/` : `types.ts`, `Board.tsx`, `rules.ts`, un test ; famille `board`.
- Plateau SVG à la manière du plateau réel : trous, lignes de bordure rouges et noires, pions, liens. Le dernier coup est marqué.
- **Bouton « Coach »** contre l'IA (pas en ligne contre un humain, pas pour un spectateur *(choix du site)*). Il ouvre un panneau : probabilité, note du dernier coup, 3 meilleurs coups (survol = pion fantôme sur le plateau), case à cocher « carte de chaleur ». Rien ne s'affiche sans clic.
- **Annuler et réessayer** : contre l'IA, reprendre son dernier coup pour en chercher un meilleur soi-même. Si l'annulation n'existe pas encore sur le site, l'ajouter là où elle sert TwixT sans casser la règle « l'hôte fait autorité ».
- **Revue après la partie** *(choix du site)* : proposée contre l'IA et en même écran ; en ligne, reportée (un invité n'a pas le journal ni forcément les poids).
- Contenu de la revue : courbe de la probabilité de gagner de Rouge coup par coup, erreurs marquées par couleur de note, clic = retour à la position de ce coup (lecture seule), 2 ou 3 moments clés signalés (plus grandes chutes). Calculée par le moteur après la partie, avec une progression visible si elle prend du temps.

## 6. Vérification

- Tests de règles : liens automatiques, croisement interdit, bordures, swap, victoire, nulle.
- Test d'exactitude du réseau contre les sorties PyTorch (§4.1).
- Conformance `ludo_core::conformance::check::<Twixt>()`.
- `ludo-cli arena twixt … hard easy` : ≥ 0,80, et la ligne dans « Commandes » du `CLAUDE.md`.
- Navigateur (playwright-core sur `pnpm preview`) : une partie contre Difficile, le coach, la revue.
- Mesure du temps d'une évaluation dans le navigateur, notée dans la spec.

## 7. Laissé au builder

Format des poids et façon de les charger, seuils des notes, budget exact de Difficile, détails du dessin, moyen technique de l'annulation.
