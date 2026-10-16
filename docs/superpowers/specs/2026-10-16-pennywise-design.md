# ludo-rs — Pennywise (spec 5)

Date : 2026-10-16
Statut : plan livré
Précédent : `2026-10-15-president-design.md` (cycle 4)

## 1. Objectif

Ajouter **Pennywise** (James Ernest, Cheapass Games / Crab Fragment Labs), un
jeu de pièces sans hasard ni information cachée, jouable contre l'IA, sur le
même écran ou en ligne, comme les autres jeux du site.

### Hors périmètre

Série de plusieurs parties avec score cumulé et premier joueur alterné (la
règle la suggère ; l'utilisateur a choisi une partie unique, arrêtée au premier
joueur ruiné) ; mains de départ libres (on choisit un personnage) ; personnages
différents selon les joueurs ; animations et sons.

## 2. Sources

- **[J]** Fiche « Pennywise » de JoCo Cheapass, `crabfragmentlabs.com/s/JocoCheapassPennywise.pdf`,
  consultée le 2026-10-16. C'est la seule règle écrite.
- **[C]** Cartes personnages de l'édition en boîte de Cheapass Games, d'après
  la photo de l'utilisateur (2026-10-16). Elles donnent les mains de départ.
- **[U]** Précisions de l'utilisateur, qui possède le jeu : on choisit un
  personnage, et tout le monde joue avec sa main.

`games/pennywise/RULES.md` cite [J], [C] ou [U] pour chaque règle et marque ❓
ce qui n'est pas tranché.

## 3. Règles (`pennywise`, 2 à 6 joueurs)

- **Mains de départ** [C][U] : un personnage est choisi dans le lobby, et
  chaque joueur reçoit la même main :

  | Personnage | Pièces | Total |
  |---|---|---|
  | Cody (défaut) | 4 × 1¢, 3 × 5¢, 2 × 10¢, 1 × 25¢ | 64¢ |
  | Darlene | 3 × 1¢, 3 × 3¢, 2 × 10¢, 1 × 20¢ | 52¢ |
  | Sugar | 2 × 1¢, 2 × 2¢, 2 × 5¢, 1 × 10¢ | 26¢ |
  | Taylor | 3 × 1¢, 2 × 5¢, 1 × 10¢ | 23¢ |

  Cody est exactement la main d'exemple de [J], d'où le défaut.
- **Pot** [J] : vide au départ, au milieu de la table.
- **Tour** [J] : le joueur pose une pièce de sa main dans le pot, puis reprend
  dans le pot un assortiment de pièces de valeur totale strictement inférieure
  à la pièce posée. Le tour passe à gauche (siège suivant).
- **Reprendre le maximum** [J][U] : « You should always take back as much as
  you can. » Le site l'impose (choix de l'utilisateur, 2026-10-16) : le joueur
  reprend le plus grand montant que le pot permet sous la valeur posée (rien si
  le pot ne le permet pas). Quand plusieurs assortiments différents (en valeurs de
  pièces) atteignent le montant repris, le joueur choisit lequel ; par exemple,
  5¢ se reprend en 3+2 ou en 2+2+1.
- **Fin** [J][U] : la partie s'arrête dès qu'un joueur n'a plus de pièce à la
  fin de son tour (on ne perd des pièces qu'en jouant, donc l'élimination tombe
  toujours à la fin du tour de celui qui vient de jouer).
  - À deux [J] : l'autre joueur gagne ; son score est le nombre de cents qui lui
    restent.
  - À plus de deux [J] : le joueur ruiné est dernier ; les autres sont classés
    par argent restant, les égalités comptant comme égalités.
- **Terminaison** : chaque tour fait baisser strictement l'argent total des
  joueurs (on pose v, on reprend au plus v − 1), la partie finit donc toujours.
- **Premier joueur** (mon choix) : tiré de la graine, puisque [J] fait alterner
  le premier joueur et que le site ne joue qu'une partie.

## 4. Moteur (`games/pennywise`, crate à part)

Le jeu n'appartient ni au deck Pairs ni aux cartes classiques : il a sa propre
crate et suit « Ajouter un jeu » du `CLAUDE.md`.

- `Config { character: Character }`, où `Character` vaut `Cody` (défaut),
  `Darlene`, `Sugar` ou `Taylor`. L'option apparaît dans le lobby via
  `Game::options` (`Choice`) et ses libellés vont dans `web/src/games/options.ts`.
- Les valeurs sont représentées par un type `Coin` (1, 2, 3, 5, 10, 20, 25¢).
  Une main et le pot sont des multiensembles (un compte par valeur).
- `Action::Play { coin, take }`, où `take` est un multiensemble de pièces
  tirées du pot. `legal_actions` énumère, pour chaque valeur de pièce en main,
  chaque assortiment distinct qui atteint le montant maximal repris (une somme
  de sous-ensembles bornée par v − 1). `apply` refuse tout le reste
  sans toucher à l'état.
- La `View` montre tout : les mains de chacun (par valeur, avec le total), le
  pot, le joueur actif, le dernier coup joué (pièce posée et pièces reprises) et,
  en fin de partie, le classement et les scores. Aucune valeur n'est calculée
  côté web.
- `Determinize` est l'identité (information complète).
- Chrono en ligne (`clock: true`) : `timeout_action` joue le premier coup légal
  (mon choix).
- Les tests de règles portent sur la reprise, les assortiments multiples, la
  fin à deux et à plusieurs, et le refus des coups illégaux. La conformance
  passe pour chaque nombre de joueurs et chaque personnage.

## 5. IA

L'ISMCTS générique, avec des parties simulées au hasard (aucune évaluation :
la mesure dépasse largement le seuil, 0.965 à 2 joueurs et 0.856 à 4). Mesure : `ludo-cli arena pennywise 200 hard easy`, où Difficile doit
marquer ≥ 0.55 contre Facile. La valeur mesurée, à 2 et à 4 joueurs, va dans
« Commandes » du `CLAUDE.md`.

## 6. Web

- `web/src/games/pennywise/` : `types.ts`, `Board.tsx`, `rules.ts`, un test
  dans `__tests__/`, une entrée dans `web/src/games/registry.ts`.
- Pièces affichées en jetons aux couleurs des cartes [C] : 1¢ blanc, 2¢ mauve,
  3¢ vert, 5¢ rouge, 10¢ bleu, 20¢ violet, 25¢ brun. Le pot est au centre, les
  mains des adversaires sont visibles autour.
- Pour jouer, on clique une pièce de sa main. Si un seul assortiment est permis,
  le coup part directement ; sinon le plateau propose les assortiments venus de
  `legalActions`, et le joueur en choisit un.
- Accueil (mon choix) : une nouvelle famille `coins` et une étagère « Jeux de
  pièces » après « Cartes traditionnelles ».
- Un rappel de règles s'affiche à table, d'accord avec `RULES.md`.

## 7. Livraison

Un plan (`docs/superpowers/plans/2026-10-16-pennywise.md`), un commit par
étape (moteur, registre, web, docs), puis une relecture.
