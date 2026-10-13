# ludo-rs

Jeux traditionnels dans le navigateur : contre l'IA, sur le même écran, ou en ligne entre amis (P2P, sans serveur de jeu).
Le moteur est en Rust, compilé en WebAssembly ; le site est en React.

Jeux :
- **Yavalath** (2–3 joueurs, Cameron Browne) ;
- **Mahjong Riichi** (4 joueurs, règles Mahjong Soul, moteur [riichienv-core](https://github.com/smly/RiichiEnv)) ;
- **Continuous Pairs** (2–8 joueurs, James Ernest et Paul Peterson, deck Pairs) ;
- **L'Auberge des Pirates** (2–8 joueurs, Regent multijoueur, deck Pairs).

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
