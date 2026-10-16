# ludo-rs

Jeux traditionnels dans le navigateur : contre l'IA, sur le même écran, ou en ligne entre amis (P2P, sans serveur de jeu).
Le moteur est en Rust, compilé en WebAssembly ; le site est en React.

Jeux :
- **Yavalath** (2–3 joueurs, Cameron Browne) ;
- **Mahjong Riichi** (4 joueurs, règles Mahjong Soul, moteur [riichienv-core](https://github.com/smly/RiichiEnv)), retiré de l'accueil pour l'instant ;
- **Jour de chance** (Continuous Pairs, 2–8 joueurs, James Ernest et Paul Peterson, deck Pairs) ;
- **L'Auberge des Pirates** (2–8 joueurs, Regent multijoueur, deck Pairs) ;
- **Ghost Town** (2–6 joueurs, jeu à jetons de James Ernest, deck Pairs) ;
- **Zombie Life** (3–6 joueurs, variante Origames de Pairs : Refuges, Horde, 21 points) ;
- **Goblin Poker** (3–8 joueurs, jeu à jetons de James Ernest, deck Pairs) ;
- **To the Moon!** (2–6 joueurs, jeu de plis Cheapass Games, deck Pairs) ;
- **Deadfall** (2–7 joueurs, jeu de bluff de James Ernest, deck Pairs) ;
- **Captain's Orders** (4–8 joueurs, Cheapass Games, deck Pairs) ;
- **Pendleton** (2 joueurs, deck Pairs) ;
- **Kitty** (3–6 joueurs, deck Pairs) ;
- **Octopus** (4–6 joueurs, jeu à jetons, deck Pairs) ;
- **Port** et **Starboard** (2–7 joueurs, deck Pairs) ;
- **Sweep** (3–6 joueurs, deck Pairs) ;
- **Rocket** (1–6 joueurs contre une banque automatique, deck Pairs) ;
- **Monster** (4–8 joueurs, deck Pairs) ;
- **Troll Guts** (3–6 joueurs, jeu à jetons, deck Pairs) ;
- **Blackstone** (2–8 joueurs, jeu à jetons, deck Pairs) ;
- **Président** (4–7 joueurs, cartes traditionnelles, règles d'après Wikipédia et Pagat ; révolution, fin sur un 2 et jokers en options) ;
- **Collusion** (4 joueurs, jeu de plis de David Parlett où l'on vise autant de plis qu'un autre joueur ; phrases toutes faites à table, cible de 50 à 100 points) ;
- **Pennywise** (2–6 joueurs, James Ernest / Cheapass Games, jeu de pièces ; personnage Cody, Darlene, Sugar ou Taylor en option).

## Lancer en local
```bash
cargo install wasm-bindgen-cli --version 0.2.121 --locked   # une fois
./scripts/build-wasm.sh dev
cd web && pnpm install && pnpm dev
```

## Architecture
- `crates/ludo-core` : le contrat `Game`, le journal de partie (`MatchLog`, `replay`), l'IA générique (ISMCTS), la conformance.
- `games/<jeu>` : les règles d'un jeu, sa vue, ses heuristiques ; `RULES.md` cite les sources.
- `crates/ludo-registry` : la liste des jeux. `crates/ludo-wasm` : le pont vers le navigateur. `crates/ludo-cli` : arène d'IA et partie en terminal.
- `web/` : accueil, lobby, sessions (locale, hôte, invité), un plateau par jeu.

En ligne, l'hôte fait tourner le moteur ; chaque invité, connecté à l'hôte seul via PeerJS, ne reçoit que ce que son siège peut voir.

Voir `docs/superpowers/specs/` pour la conception et `CLAUDE.md` pour les règles du dépôt et la marche à suivre pour ajouter un jeu.

## Licence
MIT. Licences tierces (moteur de Mahjong, tuiles) : voir `NOTICE`.
