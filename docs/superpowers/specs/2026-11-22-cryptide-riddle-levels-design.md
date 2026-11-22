# Énigmes de Cryptide : quatre niveaux

Date : 2026-11-22. Petit chantier (document court).

## Intention

L'énigme actuelle (`games/cryptide/src/riddle.rs`, 2026-10-22) est trop dure
pour l'utilisateur : elle demande d'emblée la déduction complète d'un joueur
Difficile, et sa solution ne fait pas comprendre comment la case tombe (la liste
des indices possibles par joueur en laisse souvent plusieurs, car Difficile
raisonne par **combinaisons** d'indices distincts qui désignent une seule case).
Il veut une progression qui apprend à lire le jeu, du plus simple au problème
actuel.

## Niveaux

1. **Échauffement** : une carte et tous les indices, sans jetons ni partie
   jouée. Le joueur clique la case qui les vérifie tous. La solution colore, pour
   chaque indice, les cases qu'il garde.
2. **L'indice d'un joueur** (décision de l'utilisateur : on ne connaît pas la
   case du cryptide) : carte et jetons d'une partie d'IA, arrêtée dès que les
   disques et cubes d'**un** joueur ne laissent qu'un indice possible du mode.
   Le joueur choisit cet indice dans l'aide de jeu. La solution montre, pour
   chaque autre indice, un jeton qui l'élimine (disque hors de l'indice, ou cube
   dans l'indice).
3. **Énigme facile** : partie arrêtée quand les jetons ne laissent qu'un indice
   possible à chaque autre joueur (vu du résolveur) ; le joueur reçoit l'indice
   du résolveur et clique la case.
4. **Énigme** : inchangée ; sa solution montre en plus la ou les combinaisons
   restantes et, pour les autres, pourquoi elles tombent (aucune case, ou
   plusieurs).

L'écran garde sa forme et gagne un sélecteur « Niveau » à côté de Joueurs et
Mode ; l'énoncé dit ce qu'il faut trouver (« Trouve la case » / « Trouve
l'indice de Léa »).

## Approches

- **Retenue** : le générateur du moteur prend un niveau et renvoie l'énoncé, la
  réponse et l'explication ; l'écran affiche et vérifie. Une seule fonction
  WASM `cryptideRiddle(level, mode, players, seed)`.
- Écartée : calculer l'échauffement dans l'écran (contraire à « le moteur
  possède toute la logique », et l'explication vient du moteur de toute façon).
- Écartée : une fonction par niveau (les niveaux 2–4 partagent la partie
  jouée, seule la condition d'arrêt change).

## Choix de Claude (validés par l'utilisateur en bloc)

- Niveau 2 : seuls les jetons du joueur visé comptent, sans croiser avec les
  autres joueurs (explication simple : un jeton par indice éliminé).
- Le joueur visé au niveau 2 est choisi par le générateur.
- Niveaux en accès libre, sans déblocage.
- Niveau 3 : si l'arrêt « un indice par joueur » s'avère trop rare (parties
  finies avant), on mesure, et l'on se rabat sur « au plus deux indices
  possibles par joueur ».

## Hors périmètre

L'onglet « Jeux solo / puzzles » et le blog : chantiers séparés, après celui-ci.

## Laissé au constructeur

Forme exacte du type `Riddle` (énumération par niveau ou champs communs),
mise à jour de `web/src/games/cryptide/types.ts`, textes dans `web/src/i18n/`,
tests du générateur par niveau, mode et nombre de joueurs.
