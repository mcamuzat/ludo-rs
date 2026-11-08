# Enemy Chocolatier — règles jouées

James Ernest, Cheapass Games, 2006 (CAG 043), 2 à 6 joueurs. Spec : `docs/superpowers/specs/2026-11-08-enemy-chocolatier-design.md`.

## Sources

- [R] Règle d'origine, `Chocolatier Rules.pdf` (print-and-play 2019, hors dépôt).
- [P] Planches, `Chocolatier Boards.pdf` : l'usine, six sections, huit recettes. Relevées par `scripts/chocolatier-board.py` (voir son en-tête).

## Matériel

- Six sections autour de l'usine [R] : une au-dessus et une au-dessous dans la colonne du milieu, deux debout de chaque côté. Sections et sens tirés au hasard (½ tour possible au milieu, ¼ de tour d'un côté ou de l'autre sur les côtés).
- 46 quartiers, 199 maisons [P] : 40 de chaque ingrédient, rose = Sucre, brun = Chocolat, beige = Noisettes, rouge = Amour, et 39 grises qui ne comptent pas.
- Huit recettes de 19 ingrédients [P], une par joueur, cachée aux autres.

## Tour

1. Revenu [R] : 1 point par tranche de 5 $ détenus pour chaque « 5 $ = 1 pt » (sur l'argent d'avant le revenu), les points des étoiles de ses quartiers fermés (+1 chacune par « Tous ★ +1 »), puis 2 $ et les billets de ses quartiers fermés (+1 chacun par « Tous $ +1 », jamais sur les 2 $ de base). À 20 points : victoire.
2. Achats [R] : 2 maisons au plus, 3 avec « Achat ×3 » (❓ ne se cumule pas), plus une maison par bon dépensé. Un tour sans achat rapporte un bon ; bons illimités (❓ « environ 20 jetons » mais « pas de limite »).

- Maison libre : son prix à la banque. Maison d'un autre : 6 $ (2 $ banque, 4 $ propriétaire), 10 $ (4 $ banque, 6 $ propriétaire) s'il a « Maisons à 10 $ ». Le propriétaire ne refuse pas.
- **Fermé** : toutes les maisons au même joueur. **Ouvert** : pas fermé, et touche l'usine ou un quartier fermé (de n'importe qui). On n'achète que dans un quartier ouvert ; « Achat fermé » lève la condition « pas fermé », « Achat partout » la condition « touche ».
- **Se toucher** [R] : en face, de l'autre côté d'une rue, pas en diagonale à un carrefour. Le moteur (`layout.rs`) assemble la ville sur une grille de 5 px et retient les paires qui se font face à travers 50 px de rue au plus, sur 15 px au moins, y compris d'une section à l'autre et avec l'usine.
- Revenus, points et pouvoir d'un quartier vont à celui qui l'a fermé, dès la fermeture, tant qu'on ne lui reprend pas une maison.

## Fin

- 20 points au revenu, ou recette complète : chaque maison colorée possédée est un ingrédient. ❓ La recette est vérifiée après chaque achat de son propriétaire (seul moment où elle peut se compléter) : il gagne aussitôt.
- Le gagnant est premier, les autres ex aequo.
- ❓ Limite du site : 60 tours de table, puis classement aux points, puis à l'argent.
- ❓ Siège 0 commence (4 $), puis 5 $, 6 $… dans l'ordre des sièges.
