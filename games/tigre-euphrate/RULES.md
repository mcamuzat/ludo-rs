# Tigre et Euphrate — règles appliquées

Source : **[R]** règle française de l'édition Hans im Glück de 1997 (traduction Bruno Faidutti, mise en page et aide de jeu François Haffner), hors dépôt. Les points non tranchés par [R] sont marqués ❓.

## Matériel

- Plateau de 16 × 11 cases, rivières, dix cases au sphinx ailé (temple + trésor au départ), dont quatre trésors de coin. [R] renvoie aux « règles allemandes » pour le plateau : relevé d'après le plateau de 1997 ❓ (à vérifier sur la boîte), dans `src/board.rs`.
- 153 tuiles civilisation [R] ; par couleur ❓ (non donné par [R]) : 30 noires (population), 57 rouges (temples), 36 bleues (fermes), 30 vertes (marchés).
- Six monuments bicolores [R] : toutes les paires des quatre couleurs.
- Chaque joueur : une dynastie (Lion, Buffle, Arc, Poterie), quatre chefs (roi noir, prêtre rouge, fermier bleu, marchand vert), deux catastrophes, six tuiles cachées [R].

## Tour

Le siège 0 commence ❓. Jusqu'à deux actions par tour, la même deux fois si on veut ; on peut en faire moins [R] :

1. **Chef** : poser un chef de sa réserve, en déplacer un, ou en reprendre un. La case est vide, hors rivière, à côté (par un côté) d'un temple, et ne réunit pas deux royaumes ; un déplacement est jugé chef soulevé [R].
2. **Tuile** : sur une case vide ; bleue seulement sur une rivière, les autres jamais ; jamais en réunissant plus de deux royaumes [R]. Posée dans un royaume, elle rapporte un point de sa couleur au propriétaire du chef de sa couleur, sinon du roi ; rien hors royaume ni en réunissant deux royaumes [R].
3. **Catastrophe** : sur une case vide ou une tuile (retirée du jeu), jamais sur un trésor, un monument, un chef ou une autre catastrophe ; elle coupe les liaisons [R].
4. **Échange** : défausser face cachée des tuiles de sa main et en piocher autant [R].

Un chef qui n'a plus de temple voisin (catastrophe, monument, conflit) rentre dans la réserve de son propriétaire [R].

## Conflits

- **Interne** : un chef posé dans un royaume où se trouve un chef de sa couleur. Chacun compte ses temples voisins ; l'attaquant (celui qui pose), puis le défenseur, ajoutent des temples de leur main une seule fois. Plus grand total gagne, égalité au défenseur. Le perdant reprend son chef, le vainqueur marque 1 point rouge, les renforts sont défaussés [R].
- **Externe** : une tuile réunit deux royaumes où se trouvent deux chefs d'une même couleur. Marqueur de jonction sur la tuile. Plusieurs conflits : le joueur actif choisit l'ordre. Attaquant : le joueur actif s'il est concerné, sinon le premier concerné dans le sens horaire après lui. Partisans : les tuiles de la couleur dans l'ancien royaume de chaque chef ; la tuile de jonction ne compte pour personne ❓. Renforts de la couleur, attaquant puis défenseur. Le perdant retire son chef et ses partisans ; le vainqueur marque 1 point + 1 par partisan retiré, dans la couleur. Exception des prêtres : les temples qui portent un trésor ou touchent un chef hors du conflit restent et ne rapportent rien. Si le royaume se divise, les conflits restants n'ont plus lieu [R].
- **Renforts**, option `reinforcements` : `English` (défaut, choix de l'utilisateur) = « variante anglaise » de [R] : l'attaquant n'en ajoute que si son total dépasse alors celui du défenseur ; le défenseur ajoute exactement de quoi égaliser, ou rien. `Free` = règle de base de [R]. Un choix qui n'a qu'une possibilité s'applique d'office.

## Monuments

Une tuile qui complète un carré de quatre tuiles d'une couleur permet, après ses conflits, d'y bâtir un monument libre ayant cette couleur ; les quatre tuiles sont retournées (elles relient toujours, ne comptent plus comme partisans), un trésor y reste. Sans monument bâti, le carré n'en recevra jamais [R]. Plusieurs carrés : le joueur en choisit un, ou aucun (tous perdus) ❓. Un carré dont une tuile disparaît redevient possible ❓.

À la fin de son tour, le joueur actif marque 1 point par monument situé dans le royaume d'un de ses chefs de la couleur ; le roi seulement pour le noir [R].

## Trésors

À la fin de chaque action, le propriétaire du marchand d'un royaume qui contient deux trésors ou plus les prend tous sauf un ; il doit prendre ceux des coins s'il le peut [R]. Une seule façon possible : appliquée d'office. Les trésors pris sont visibles de tous (ajout du site : comme à la table, on voit qui les prend).

## Fin du tour et de la partie

Fin du tour : points des monuments, puis chacun complète sa main à six, le joueur actif d'abord [R]. La partie s'arrête quand il ne reste qu'un ou deux trésors à la fin d'un tour, ou quand un joueur doit piocher (fin de tour ou échange) sans que le sac suffise [R]. Ajout du site : elle s'arrête aussi après 20 tours de table sans tuile posée, catastrophe ni échange.

Score : les points de sa couleur la plus faible, les trésors servant de jokers (chacun va à la couleur la plus faible du moment, ce qui maximise le classement) ; égalité départagée par la deuxième couleur la plus faible, et ainsi de suite [R].

## Information cachée

Mains et points sont derrière les écrans [R] : la vue ne montre que les siens, tout est révélé à la fin (décision de l'utilisateur). L'IA ne garde qu'un souvenir flou (±20 % par couleur) des points des autres.

Écartée : la Ziggourat (variante de [R], pièces hors boîte).
