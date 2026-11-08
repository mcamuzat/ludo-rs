# Tigre et Euphrate — spec

## Pourquoi

L'utilisateur possède la boîte Hans im Glück de 1997 et y joue d'habitude à 3 ou 4, mais n'a personne avec qui jouer pour l'instant : le but premier est de **jouer seul contre l'IA à 3 ou 4 joueurs**. Le jeu en ligne vient avec la plateforme. Il a découvert le jeu via le dépôt `sillle14/cubenations` (TypeScript, boardgame.io), mais préfère le nom d'origine.

## Sources

- [R] Règle française de l'édition 1997 (traduction Bruno Faidutti, mise en page et aide de jeu François Haffner), `~/Téléchargements/euphr_rg.pdf`, hors dépôt.
- Le plateau n'est pas décrit par [R] (« comme indiqué dans les règles allemandes ») : relevé d'après le plateau d'origine, recoupé avec `cubenations` (sans licence : rien n'est copié, ni code ni image), **à valider par l'utilisateur sur sa boîte**.

## Décisions de l'utilisateur

- Nom « Tigre et Euphrate », règle de 1997.
- **Points de victoire cachés** : chacun ne voit que les siens pendant la partie, l'IA ne s'en souvient pas exactement non plus ; on montre tout à la fin. Une option « points visibles » pourra venir plus tard.
- **Variante anglaise** des renforts par défaut (« on interdit » les renforts inutiles) : l'attaquant ne renforce que pour passer devant, le défenseur exactement de quoi égaliser, ou rien.
- Pas d'autres questions : « commence à coder, je verrai bien ».

## Choix faits sans l'utilisateur (à vérifier)

- 2 à 4 joueurs (la boîte le permet), siège 0 commence.
- Option `reinforcements` : `English` (défaut) / `Free` (règle de base de [R]).
- La Ziggourat (variante de [R]) est écartée : pièces absentes de la boîte.
- Mémoire de l'IA : sa déterminisation brouille les points des adversaires (±20 % par couleur) ; elle connaît les trésors pris, faciles à retenir.
- Limite de durée du site : 20 tours de table sans tuile posée, catastrophe ni échange terminent la partie (décompte normal).
- Trésors : quand le Marchand n'a qu'une façon de les prendre, elle s'applique d'office.
- Plusieurs carrés complétés par une même tuile : le joueur choisit un carré et un monument, ou n'en bâtit aucun (tous les carrés proposés sont alors perdus).
- Une tuile de jonction ne compte comme partisan d'aucun des deux camps.

## IA

ISMCTS commun (100 / 1000 / 3000 itérations, ou ms dans le navigateur). `search_actions` garde une douzaine de coups sensés (tuiles qui rapportent, chefs près d'un temple, conflits gagnables, une catastrophe utile, un échange, finir son tour) ; `evaluate` coupe la simulation après quelques tours et estime chaque siège par sa couleur la plus faible. Arène : Difficile ≥ 0.55 contre Facile.

## Écran

Plateau 16 × 11 en SVG sur le tapis commun : rivières, temples et trésors (coins marqués), chefs colorés par rôle avec le symbole de la dynastie (Lion, Buffle, Arc, Poterie), monuments sur 2 × 2. Sous le plateau : sa main (6 tuiles), ses chefs, ses catastrophes, ses points, et les boutons d'action ; un panneau pour les renforts, le choix du conflit, du monument, du trésor laissé ; le récit du dernier tour.
