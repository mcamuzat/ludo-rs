# Mahjong Riichi — règles implémentées

Les règles sont celles de **Mahjong Soul**, telles qu'implémentées par
[riichienv-core](https://github.com/smly/RiichiEnv) 0.4.10 [R1] : preset
`GameRule::default_mjsoul` (`src/rule.rs`), mode de jeu 2 (hanchan). ludo ne
réécrit aucune règle de jeu : la section « Ajouts de ludo » liste ce qu'il
ajoute autour du moteur.

## Ce que fait le moteur [R1]
- **Hanchan** : mains de l'Est puis du Sud, 25 000 points de départ.
  Prolongation au vent d'Ouest en mort subite si personne n'a 30 000 points
  (`state/mod.rs`, fin de main).
- **Agari-yame** : le donneur de la dernière main, premier avec au moins
  30 000 points, peut terminer la partie en gagnant ou en étant tenpai.
- Trois cinq rouges (un par famille), kuikae interdit, doubles yakuman
  (kokushi 13 attentes, suuankou tanki, junsei chuuren, daisuushii), pao.
- Bâtons de riichi restant sur la table en fin de partie : au premier.

## Ajouts de ludo
- **Classement final** : par score ; à égalité, le siège le plus proche du
  premier donneur passe devant (comme le moteur pour les bâtons restants).
- **Action par défaut au chrono** (parties en ligne) : passer un appel ;
  sinon défausser la tuile piochée ; sinon la première défausse permise.
- **Information** : pendant une fenêtre d'appel, un joueur ne sait pas qui
  d'autre peut appeler.
- **Résultat d'une main** : yaku, han et fu sont calculés par l'évaluateur
  de riichienv-core avec les mêmes conditions que le moteur, juste avant
  qu'il ne passe à la main suivante. Les variations de score affichées sont
  celles du règlement de la main (le bâton d'un riichi est payé à la
  déclaration).

## Sources
- [R1] riichienv-core 0.4.10, Apache-2.0 : `src/rule.rs`
  (`default_mjsoul`), `src/state/mod.rs` (déroulement, fin de partie).
