# Enemy Chocolatier — spec

## Pourquoi

L'utilisateur a déposé le print-and-play de Cheapass dans `~/Téléchargements/chocolatier` et demandé le jeu dans ludo-rs, sans questions (« comme d'habitude »). Jeu de plateau de James Ernest (2006, CAG 043), 2 à 6 joueurs, une demi-heure : on achète des maisons autour de l'usine du grand chocolatier pour gagner des revenus, des points et des pouvoirs, et compléter sa recette secrète.

## Sources

- [R] Règle d'origine, `Chocolatier Rules.pdf` (une feuille recto verso).
- [P] Planches, `Chocolatier Boards.pdf` : page 1 l'usine, pages 2 à 7 les six sections, page 8 les huit recettes. Images seulement, hors dépôt.
- [I] `READ ME Instructions.pdf` (matériel, découpe à 8,25 × 5,5 pouces).

## Plateau

- `scripts/chocolatier-board.py` relève les sections (voir son en-tête) : le contour de chaque quartier est lu dans l'image (tout ce que la rue n'atteint pas), les maisons, leurs prix et les symboles viennent d'un relevé à la main inscrit dans le script. Contrôles : chaque maison tombe dans un seul quartier, chaque quartier de l'image reçoit une ligne du relevé, et il y a exactement **40 maisons par ingrédient** (39 grises). Sorties : `games/enemy-chocolatier/src/board_data.rs` (prix, revenus, pouvoirs, et chaque section sur une grille de 5 px) et `web/src/games/enemy-chocolatier/board.json` (contours, maisons, symboles).
- 46 quartiers, 199 maisons. Couleurs : rose = Sucre, brun = Chocolat, beige = Noisettes, rouge = Amour, gris = rien.
- Disposition [R] : l'usine au centre de la colonne du milieu, une section au-dessus et une au-dessous, deux sections debout de chaque côté. Les six sections sont tirées au hasard à la création, ainsi que leur sens (½ tour possible au milieu, ¼ de tour d'un côté ou de l'autre sur les côtés). Choix du site : le hasard décide de tout, sans option.
- « Se toucher » [R] : être juste en face de l'autre côté d'une rue, pas en diagonale à un carrefour. Le moteur assemble les grilles et compte, ligne par ligne et colonne par colonne, les paires de quartiers séparées par 10 cellules de rue au plus (50 px, une rue mesure ~27 px, les rues obliques ~40 px en horizontal) ; deux quartiers se touchent s'ils se font face sur 3 lignes ou colonnes au moins (15 px). Même règle à travers les jointures des sections et avec l'usine. Vérifié sur l'image de contrôle du script (107 liens pour la disposition de contrôle), et le moteur retrouve ce même nombre.

## Règle jouée

- Siège 0 commence avec 4 $, le suivant 5 $, et ainsi de suite (9 $ à 6). ❓ Le premier joueur n'est pas tiré au sort (convention du site).
- Chacun reçoit une des 8 recettes, cachée aux autres ; les recettes restantes sont cachées à tous.
- Début du tour, dans l'ordre [R] : 1 point par tranche de 5 $ détenus pour chaque pouvoir « 5 $ = 1 pt », puis les points des étoiles (+1 par étoile et par « Tous ★ +1 »), puis 2 $ et le revenu des quartiers (+1 par billet et par « Tous $ +1 »). À 20 points, la partie s'arrête : victoire.
- Achats : 2 maisons au plus (3 avec « Achat ×3 », qui ne se cumule pas ❓), plus une par bon (Rain Check) dépensé. Le bon est dépensé seulement pour une maison au-delà de la limite. Un tour sans aucun achat rapporte un bon ; les bons sont illimités ❓ (la règle dit « environ 20 jetons » mais « pas de limite »).
- Maison libre : son prix, à la banque. Maison d'un autre : 6 $ (2 $ banque, 4 $ au propriétaire), ou 10 $ (4 $ banque, 6 $ propriétaire) si le propriétaire a « Maisons à 10 $ ». Le propriétaire ne peut refuser.
- Un quartier est **fermé** quand toutes ses maisons sont au même joueur, **ouvert** s'il n'est pas fermé et touche l'usine ou un quartier fermé (de n'importe qui). On n'achète que dans un quartier ouvert. « Achat fermé » lève seulement la condition « pas fermé », « Achat partout » seulement la condition « touche » (lecture littérale de [R]).
- Un joueur possède les revenus, points et pouvoir d'un quartier qu'il a fermé, dès la fermeture (même tour) et jusqu'à ce qu'on lui reprenne une maison.
- Recette : chaque maison colorée possédée compte comme un ingrédient de sa couleur, quartier fermé ou non. Dès qu'un achat complète sa recette, le joueur gagne ❓ (la règle ne dit pas quand on vérifie ; seul l'acheteur gagne des maisons, donc c'est forcément après un de ses achats).
- Le gagnant est premier, tous les autres ex aequo derniers (la règle ne classe pas les perdants).
- Limite de durée du site ❓ : après 60 tours de table, la partie s'arrête ; classement par points, puis par argent.

## Information

- Publiques : plateau, propriétaires, argent, points, bons, pouvoirs, revenus à venir (calculés par le moteur pour l'écran).
- Cachées : la recette des autres et les recettes non distribuées. `determinize` redistribue au hasard toutes les recettes que l'observateur ne voit pas. Toutes les recettes sont montrées à la fin.
- Pas de même écran (recettes secrètes).

## IA

ISMCTS commun. `search_actions` garde une douzaine d'achats classés par une heuristique (fermer un quartier selon sa valeur, avancer vers une fermeture, ingrédient utile à sa recette, casser le quartier fermé d'un adversaire, coût), plus « Finir le tour ». Les simulations suivent cette heuristique avec un peu de hasard ; `evaluate` coupe après quelques tours et estime chaque siège par sa distance à 20 points (points, revenu de points) et à sa recette. L'IA ne connaît pas les recettes des autres (déterminisation). Arène : Difficile ≥ 0.55 contre Facile.

## Écran

La ville en SVG sur le tapis commun : rues, usine au centre (motif sobre, sans copier le dessin), quartiers à leurs couleurs avec leurs contours relevés, maisons en tuiles portant leur prix et le jeton du propriétaire, symboles (billet, étoile, étiquette du pouvoir en français). Les chiffres restent droits quelle que soit l'orientation de la section. Les maisons achetables sont surlignées avec leur coût ; un clic achète. Sous la ville : sa recette et sa progression, ses bons, le bouton « Finir le tour » (« Prendre un bon » s'il n'a rien acheté) ; les autres joueurs (argent, points, bons, revenu, pouvoirs) ; le récit du dernier tour ; un rappel de règles.

Noms des pouvoirs à l'écran : « Achat ×3 », « Achat fermé », « Achat partout », « Tous $ +1 », « Tous ★ +1 », « 5 $ = 1 pt », « Maisons à 10 $ ». Nom du jeu inchangé, famille `board`.
