# Jeux du deck Pairs — règles implémentées

Le deck : 55 cartes sans couleur, une de rang 1, deux de rang 2, … dix de
rang 10. Une *paire* : deux cartes de même rang dans une même pile.

Les points marqués ❓ ne sont pas tranchés par la source : ce sont nos choix.

## Jour de chance — Continuous Pairs (`pairs-continuous`, 2 à 8 joueurs)

Affiché « Jour de chance », nom de l'édition française que l'on possède ;
les règles sont celles de Continuous Pairs.

Source : **[R]** *Pairs Companion Book, PDF Edition* (James Ernest, Hip
Pocket Games, 2018), chapitres « How to Play Pairs », « Breaking a Tie for
Low » et « Continuous Pairs ».

### Mise en place [R]
- On mélange et on brûle 5 cartes (face cachée, à la défausse). On en brûle 5
  à chaque nouveau mélange.
- Chaque joueur reçoit une carte visible. La plus petite commence.
- Égalité pour la plus petite : chaque joueur à égalité reçoit une carte de
  plus ; on compare ces nouvelles cartes entre eux, et on recommence tant
  qu'il reste une égalité. Une carte de départage qui ferait une paire est
  défaussée et remplacée. Les cartes de départage restent dans la pile.

### Tour [R]
- Le joueur prend une carte (*hit*) ou se couche (*fold*). Le tour passe à
  gauche.
- Prendre une carte : si elle fait une paire avec sa pile, il marque le rang
  de la paire, garde une des deux cartes comme carte de score, et le reste de
  **sa** pile est défaussé. Les autres piles ne bougent pas.
- Se coucher : il prend n'importe quelle carte en jeu (dans sa pile ou celle
  d'un autre), la garde comme carte de score et marque son rang ; **sa** pile
  est défaussée.
- Un joueur sans carte ne choisit pas : il reçoit une carte, et c'est son
  tour.
- Les cartes de score ne retournent pas dans la pioche.
- Pioche vide : on mélange la défausse (pas les piles, pas les cartes de
  score), on brûle 5 cartes et on continue.

### Fin [R]
Le premier joueur qui atteint le score cible perd ; il n'y a pas de gagnant
unique. Cible : 31 à 2 joueurs, 21 à 3, 16 à 4, 13 à 5, 11 de 6 à 8.

### Choix sur les points non tranchés
1. ❓ Classement : le perdant est dernier, tous les autres premiers ex æquo.
2. ❓ Au remélange, on ne brûle jamais la dernière carte : s'il y a 5 cartes
   ou moins à mélanger, on en garde une à piocher.
3. ❓ S'il ne reste aucune carte à piocher (pioche et défausse vides),
   prendre une carte n'est pas proposé, et un joueur sans carte passe son
   tour.
4. ❓ Si la pioche s'épuise pendant le départage initial, le premier des
   joueurs encore à égalité commence.

## L'Auberge des Pirates (`pairs-auberge`, 2 à 8 joueurs)

Source : **[A]** la règle de *Pairs – L'Auberge des Pirates* (Origames,
2017), adaptation de Regent (James Ernest et Paul Peterson).

### But [A]
Être le dernier joueur en jeu. On est éliminé si on fait une paire en
défiant, si on est accusé à raison d'avoir une paire, ou si on accuse à tort.

### Mise en place [A]
- Chaque joueur reçoit 3 cartes en main.
- Tous choisissent en même temps une carte de leur main et la posent visible
  devant eux. La plus petite commence.

### Tour [A]
Une action au choix :
- **Attaquer** : jouer une carte de sa main devant un autre joueur, sans
  faire de paire avec les cartes qu'il a devant lui. C'est à lui de jouer.
- **Défier** : poser la première carte de la pioche, visible, devant un
  autre joueur. Sans paire avec ses cartes visibles : c'est à lui de jouer.
  Avec une paire : on est éliminé, la carte est défaussée, et c'est au joueur
  à sa gauche de jouer.
- **Accuser** un joueur : il montre sa main à l'accusateur seul. S'il a une
  paire entre sa main et les cartes devant lui (ou dans sa main), il est
  éliminé et c'est au joueur à gauche de l'accusateur. Sinon l'accusateur
  est éliminé et c'est à l'accusé de jouer.
- **Piocher** : prendre une carte en main sans la montrer. Une paire ainsi
  faite ne fait pas perdre tant qu'on n'est pas accusé. C'est au joueur à
  gauche.

### Choix sur les points non tranchés
1. ❓ Égalité pour commencer : tirage au sort entre les joueurs à égalité
   (la règle dit « le plus âgé »).
2. ❓ Une partie se joue en plusieurs manches (5 par défaut). Le vainqueur
   d'une manche marque 1 point ; classement final aux points, égalités
   conservées.
3. ❓ Un joueur éliminé défausse ses cartes visibles face visible et sa main
   face cachée.
4. ❓ On ne peut pas accuser un joueur qui n'a aucune carte en main : il ne
   peut pas avoir de paire.
5. ❓ Pioche vide : la défausse est remélangée. Si pioche et défausse sont
   vides, piocher et défier ne sont pas proposés ; si le joueur au tour n'a
   alors aucune action, la manche s'arrête sans vainqueur.
6. ❓ Le résultat d'une accusation est connu de tous ; la main montrée et le
   rang de la paire, de l'accusateur seul.
7. ❓ « À gauche » saute les joueurs éliminés.

## Ghost Town (`pairs-ghost-town`, 2 à 6 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Ghost Town ».

### Mise en place [R]
- Chaque joueur mise 10 jetons dans le pot et reçoit 5 cartes en main.
- Tous choisissent une carte en secret ; révélées ensemble, elles forment la
  *rue*, commune à tous. S'il y a plusieurs cartes d'un même rang dans la
  rue, les doublons sont défaussés.
- Celui qui a posé la plus petite carte commence. Une égalité est départagée
  par des cartes de la pioche, qui ne servent qu'à cela et sont défaussées.

### Tour [R]
Le tour passe à gauche. Une action au choix :
- **Jouer** : payer 2 jetons au pot et poser une carte de sa main devant soi
  (sa *maison*). Pas de paire dans une maison, pas de carte du rang d'une
  carte de la rue. Si on a déjà un rang chez soi et que le joueur actif à sa
  gauche ne l'a pas, on peut poser une carte de ce rang chez lui (même coût).
- **Piocher** : prendre une carte de la pioche, visible, dans sa maison. Si
  elle fait une paire avec sa maison ou avec la rue, on est éliminé de la
  main et on paie son rang au pot. Sinon on gagne aussitôt son rang en
  jetons, pris dans le pot.
- **Se coucher** : quitter la main et défausser ses cartes.

### Fin de la main [R]
La main s'arrête quand le pot est vide, ou quand il ne reste qu'un joueur,
qui prend alors le pot.

### Choix sur les points non tranchés
1. ❓ Une partie se joue en plusieurs mains (5 par défaut, réglable dans le
   lobby de 1 à 15). Chacun part de 0 jeton et peut passer en négatif ;
   classement final aux jetons, égalités conservées.
2. ❓ Si le pot contient moins que la carte piochée, on prend ce qui reste.
3. ❓ Pioche vide : la défausse est remélangée. S'il n'y a plus rien à
   piocher, piocher n'est pas proposé (on peut toujours se coucher).
4. ❓ Un joueur qui quitte la main défausse sa maison face visible et sa main
   face cachée.
5. ❓ Si la pioche s'épuise pendant le départage, le premier des joueurs à
   égalité commence.

## Zombie Life (`pairs-zombie-life`, 3 à 6 joueurs)

Source : **[Z]** règle Origames en 4 pages, *Pairs – Zombie Life* (adapté de
Pairs de James Ernest et Paul Peterson).

### But [Z]
Être le premier à atteindre 21 points, en plusieurs manches.

### Mise en place [Z]
- Mélanger les 55 cartes. Première manche : une carte face visible devant
  chaque joueur, dans son *Refuge*.
- Celui qui a reçu la plus petite carte commence.

### Tour [Z]
Le tour passe à gauche, parmi les joueurs encore en jeu. Une action au choix :
- **Piocher** : regarder la carte, puis la poser face visible dans son Refuge
  ou dans la *Horde* (zone commune), jamais dans une zone qui contient déjà
  ce rang. Si le rang est déjà dans la Horde **et** dans son Refuge, on est
  submergé : éliminé de la manche, et tout son Refuge est défaussé.
- **Choisir une carte de la Horde** et la placer dans son Refuge, si ce rang
  n'y est pas déjà.
- **Passer** : on se cache et on ne joue plus de la manche. On retourne face
  cachée autant de cartes de son Refuge qu'il reste de joueurs en jeu ; à
  chaque élimination ensuite, on en remet une face visible.

Le dernier joueur en jeu peut continuer, mais ne peut plus prendre dans la
Horde.

### Fin de manche [Z]
- Quand tous ont passé ou sont éliminés, chaque joueur qui a passé marque
  1 point par carte face visible devant lui.
- Si un joueur a au moins 21 points, la partie s'arrête et le plus de points
  gagne. Sinon, nouvelle manche : le joueur au plus petit score (à égalité,
  le premier éliminé) pioche autant de cartes qu'il y a de joueurs et en
  place une devant chacun. La plus petite carte commence.

### Choix sur les points non tranchés
1. ❓ « Autant de cartes qu'il reste de joueurs en jeu » : celui qui passe
   ne se compte pas. S'il a moins de cartes, il les retourne toutes.
2. ❓ Toutes les cartes valent 1 point : on ne choisit pas lesquelles
   retourner, le jeu n'en retient que le nombre.
3. ❓ En fin de manche, la Horde est défaussée avec les Refuges.
4. ❓ La pioche n'est mélangée qu'au début ; vide, elle est refaite avec la
   défausse. La carte d'un joueur submergé est défaussée avec son Refuge.
5. ❓ Égalité sur la plus petite carte (« le joueur le plus âgé ») : tirage
   au sort.
6. ❓ Égalité pour donner sans qu'aucun des ex æquo ait été éliminé : tirage
   au sort.
7. ❓ Égalité au score final : victoire partagée.
8. ❓ La carte piochée et les cartes du donneur sont visibles de tous :
   personne d'autre n'agit avant qu'elles soient posées. Le donneur donne
   ses cartes une à une, à partir de sa gauche ; des cartes toutes
   identiques sont données sans demander.

## Goblin Poker (`pairs-goblin-poker`, 3 à 8 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Goblin Poker ».

### Mise en place [R]
- Chaque joueur mise 1 jeton dans le pot et reçoit 6 cartes en main.
- Tous choisissent une carte en secret ; révélées ensemble, elles commencent
  la *pile* visible de chacun.

### Tour de table [R]
En partant de la pile la plus basse, puis vers la gauche, chaque joueur
encore en jeu choisit :
- **Se coucher** : payer au pot la valeur de la plus petite carte en jeu, et
  sortir de la main.
- **Jeter** : mettre une carte de sa main, face cachée, au centre.

Quand tous ont agi, les cartes jetées sont mélangées et données face visible,
une par joueur restant, en partant de la pile la plus basse puis vers la
gauche. Qui reçoit une paire sort de la main et paie au pot le rang de la
carte. Les éliminations se font dans l'ordre : dès qu'il ne reste qu'un
joueur, il gagne, même s'il allait recevoir une paire.

### Pile la plus basse [R]
Celle qui a la plus petite carte, puis la deuxième plus petite, et ainsi de
suite. Une carte en plus est plus basse que pas de carte : 2-3-6 est plus bas
que 2-3. Une égalité parfaite est départagée par des cartes de la pioche, qui
rejoignent les piles ; une carte qui ferait une paire est défaussée et
remplacée.

### Fin de la main [R]
- S'il ne reste qu'un joueur, il prend le pot.
- Sinon on rejoue un tour de table, cinq au plus. Les survivants du dernier
  se partagent le pot ; le reste de la division reste au pot pour la main
  suivante.

### Choix sur les points non tranchés
1. ❓ Une partie se joue en plusieurs mains (5 par défaut). Chacun part de 0
   jeton et peut passer en négatif ; classement final aux jetons, égalités
   conservées. Les jetons restés au pot après la dernière main ne vont à
   personne.
2. ❓ « La plus petite carte en jeu » : la plus petite carte visible des
   piles des joueurs encore en jeu, celle du joueur qui se couche comprise.
3. ❓ La pile la plus basse est recalculée au début de chaque tour de table,
   et de nouveau avant la redistribution, parmi les joueurs restants.
4. ❓ Seule l'égalité pour la pile la plus basse est départagée.
5. ❓ La pioche n'est pas refaite pour un départage : vide, c'est le premier
   des joueurs à égalité, dans l'ordre des sièges, qui est le plus bas.
6. ❓ Un joueur qui sort défausse sa pile face visible et sa main face
   cachée.

## To the Moon! (`pairs-to-the-moon`, 2 à 6 joueurs)

Source : **[M]** fiche *To the Moon!* (James Ernest, Cheapass Games, 2018,
jeu Pairs du mois n° 5, `5CAGToTheMoon.pdf`).

### Manche [M]
- On mélange tout le deck et on distribue une main à chacun : 8 cartes à 2
  joueurs, 7 à 3, 6 à 4, 5 à 5, 4 à 6.
- Le donneur retourne la carte du dessus de la pioche : elle commence le
  premier *compte à rebours* (le pli). Un compte à rebours contient au plus
  une carte de chaque rang.

### Tour [M]
On joue dans le sens horaire en sautant les joueurs sortis. À son tour, on :
- **Construit** : on pose une carte de sa main d'un rang absent du compte à
  rebours. Qui complète un compte à rebours parfait (les dix rangs) gagne le
  pli aussitôt ; sinon on reste en jeu.
- **Teste** : on retourne la carte du dessus de la pioche. Rang absent : elle
  rejoint le compte à rebours et on reste en jeu. Rang présent : on sort du
  pli et on prend cette carte en main.
- **Se retire** : on sort du pli et on prend en main une carte du compte à
  rebours, au choix, sauf si elle était la seule sur la table.

### Fin du pli et de la manche [M]
- Quand il ne reste qu'un joueur en jeu, il gagne le pli : un point par
  carte. Tout le monde revient en jeu et le gagnant entame le pli suivant
  avec la carte du dessus de la pioche.
- Si la pioche est vide à ce moment, la manche est finie.
- Six manches ; le plus haut total gagne.

### Choix sur les points non tranchés
1. ❓ La donne passe à gauche à chaque manche.
2. ❓ Le joueur à gauche de celui qui entame agit le premier ; celui qui
   entame est en jeu.
3. ❓ Un test réussi qui complète les dix rangs gagne aussi le pli aussitôt.
4. ❓ La pioche n'est jamais refaite pendant une manche : pioche vide, on ne
   peut plus tester.
5. ❓ Classement final aux points, égalités conservées. Nombre de manches
   réglable (`Config { rounds }`, plafond 15).
6. ❓ Les cartes restées en main à la fin d'une manche ne valent rien.
7. ❓ Les cartes entrées dans une main à la vue de tous (prise en se
   retirant, carte d'un test raté) restent affichées face visible jusqu'à
   ce qu'une carte de ce rang soit jouée de cette main.

## Deadfall (`pairs-deadfall`, 2 à 7 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Deadfall » (règles de
base ; les variantes du chapitre ne sont pas implémentées).

### Mise en place [R]
- Chacun mise 1 jeton dans le pot et reçoit 6 cartes (7 à deux joueurs,
  5 à sept).
- Chacun choisit en secret une *carte de porte* ; révélées ensemble, elles
  commencent la pile visible de chacun. La plus basse joue la première, puis
  on tourne dans le sens horaire.
- Égalité pour la plus basse : tout le monde joue une autre carte de porte.
  Seule la nouvelle carte compte, la précédente départage ; si l'égalité
  demeure, on recommence avec une troisième, etc.

### Tour [R]
- **Jouer** une carte de sa main sur sa pile : on prétend qu'il reste des
  cartes de ce rang en main chez quelqu'un.
- **Appeler** la carte du dessus d'un autre joueur : on prétend que ce rang
  est *mort*, qu'il n'en reste dans aucune main, la sienne comprise. Si
  plusieurs cartes du dessus ont le même rang, seule la dernière jouée peut
  être appelée. Les cartes de porte comptent comme jouées dans le sens
  horaire à partir du premier joueur.

### Abattage [R]
L'appel met fin à la main ; toutes les mains sont montrées. Rang mort :
l'appelant gagne ; sinon le joueur appelé gagne. Le perdant paie au gagnant
le rang de la carte appelée, et le gagnant prend les mises.

### Choix sur les points non tranchés
1. ❓ Une partie se joue en plusieurs mains (5 par défaut, `Config
   { hands }`, plafond 15). Chacun part de 0 jeton et peut passer en
   négatif ; classement final aux jetons, égalités conservées.
2. ❓ La restriction d'appel ne compare que les cartes du dessus des
   *autres* joueurs : la sienne n'empêche jamais d'appeler une carte de même
   rang. Ainsi on a toujours au moins un appel possible.
3. ❓ Si les mains sont vides alors que l'égalité de porte demeure, le
   premier des ex æquo dans l'ordre des sièges commence.
4. ❓ Un joueur sans carte en main ne peut qu'appeler.
5. ❓ Les cartes non distribuées ne comptent pas : un rang dont il ne reste
   des cartes que là est mort.

## Captain's Orders (`pairs-captains-orders`, 4 à 8 joueurs)

Source : **[C]** fiche *Captain's Orders* (James Ernest, Cheapass Games,
2018, jeu Pairs du mois n° 8, `8CAGCapsOrders.pdf`).

### Rôles [C]
- Le *Capitaine* est le joueur qui tient la carte la plus basse de toutes
  les mains ; il change dès qu'une carte bouge. Plusieurs joueurs à égalité
  sont tous Capitaines. Les autres sont des *Mutins*.
- Un Mutin gagne la manche en démasquant un Capitaine, ou en ayant le plus
  de points en main (chaque carte vaut son rang). Un Capitaine ne gagne
  qu'en vidant sa main.

### Manche [C]
On mélange tout le deck et on distribue 6 cartes à chacun ; le reste n'est
pas joué. Le joueur à gauche du donneur commence, puis on tourne dans le
sens horaire. La donne passe à gauche à chaque manche.

### Sans ordre sur la table [C]
À son tour, on fait une des choses suivantes :
- **Ordonner** : poser une carte de sa main face visible au centre. Elle
  devient un *ordre* dont on est le propriétaire.
- **Passer une carte** face cachée à un autre joueur, qui défausse ensuite
  une carte face cachée.
- **Demander une carte** à un autre joueur, qui en donne une face cachée ;
  on défausse ensuite une carte face cachée.
- **Accuser** un joueur d'être le Capitaine.

### Ordre en cours [C]
L'ordre fait le tour de la table à partir de la gauche de son propriétaire,
qui ne répond pas à son propre ordre. Chacun à son tour :
- **Obéit** : donne face cachée au propriétaire une carte de rang supérieur
  ou égal à l'ordre (sans défausse ensuite) ;
- **Annule** : pose un nouvel ordre plus bas ; l'ancien est défaussé et le
  nouveau lui appartient ;
- ou **accuse**.

Quand l'ordre a fait tout le tour, son propriétaire le défausse et c'est au
joueur à sa gauche.

### Fin de manche [C]
- **Accusation** : toutes les mains sont révélées. Un Mutin qui désigne un
  Capitaine gagne la manche. Sinon il ne peut pas gagner, et l'autre Mutin
  qui a le plus de points la gagne. Un Capitaine ne gagne pas en en
  désignant un autre.
- **Main vide** : qui vide sa main (en passant une carte ou en posant un
  ordre) révèle cette carte et se déclare Capitaine ; s'il a raison, il
  gagne. Sinon le Mutin qui a le plus de points gagne, et une carte passée
  reste dans la main de celui qui l'a reçue.
- Égalité aux points : chaque ex æquo gagne la manche. Le premier à trois
  manches gagnées remporte la partie.

### Choix sur les points non tranchés
1. ❓ La déclaration de Capitaine est automatique : ne pas se déclarer ne
   peut rien rapporter (une main vide vaut 0 point).
2. ❓ Vider sa main en donnant la carte demandée, ou en obéissant à un
   ordre, compte comme la passer.
3. ❓ Qui vide sa main a raison si la carte révélée est inférieure ou égale
   à toutes les cartes des autres mains, celle qui la reçoit comprise (avant
   de la recevoir).
4. ❓ Les Capitaines de la fin de manche (pour désigner les Mutins qui
   comptent leurs points) sont lus sur les mains révélées. Qui s'est
   déclaré Capitaine à tort ne peut pas gagner aux points, comme
   l'accusateur qui s'est trompé. S'il ne reste aucun Mutin pour gagner,
   personne ne gagne la manche.
5. ❓ Après un passage ou une demande, c'est au joueur à gauche de celui
   qui a joué.
6. ❓ La partie se joue en 3 manches gagnées par défaut (`Config
   { wins }`, plafond 9). Plusieurs joueurs peuvent l'atteindre ensemble :
   classement aux manches gagnées, égalités conservées.

## Pendleton (`pairs-pendleton`, 2 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Pendleton » (autrefois
« Five Cards »).

### Mise en place [R]
- On distribue cinq cartes de rangs différents en ligne (les doublons sont
  écartés, puis remélangés dans la pioche) et on les trie de la plus basse à
  la plus haute : ce sont les *cibles*, une par colonne.
- Un joueur tiré au sort commence. Aux manches suivantes, celui qui a marqué
  le moins de points à la manche précédente commence.

### Tour [R]
- On pioche une carte et on la pose face visible de son côté d'une colonne.
- Une colonne n'accepte pas une carte de même rang qu'une de ses cartes : la
  cible ou une carte de l'un ou l'autre côté.
- La manche s'arrête dès qu'un joueur pioche une carte qu'aucune colonne
  n'accepte.

### Score [R]
- Dans chaque colonne, qui a posé le plus de points de son côté marque la
  valeur de la cible ; à égalité, chacun en marque la moitié, arrondie en
  dessous.
- On rejoue des manches ; le premier à 100 points gagne.

### Choix sur les points non tranchés
1. ❓ La partie s'arrête à la fin de la manche où un joueur atteint 100
   points (`Config { target }`, plafond 1000) avec plus de points que
   l'autre. À égalité, on joue une manche de plus.
2. ❓ Après une manche nulle, celui qui jouait en second commence.
3. ❓ Une colonne où personne n'a posé de carte ne rapporte rien (le livret
   parle d'égalité, mais personne n'y a « posé le plus de points »).
4. ❓ Si la pioche s'épuise (rarissime : 50 cartes), la manche s'arrête
   comme sur une carte injouable.

## Kitty (`pairs-kitty`, 3 à 6 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitres « Kitty » et « Kitty
Variants » (variante *Soft* ; ni *Sticky*, ni partenaires, ni jetons).

### Mise en place [R]
- On pose 7 cartes face cachée : la *kitty*. Le reste est distribué : 12
  cartes chacun à 4 joueurs. À 5 joueurs, 3 cartes sont écartées sans être
  vues pour que les mains tombent juste (9 chacun ; 16 à 3, 8 à 6).

### Manche [R]
- On retourne la carte du dessus de la kitty : la *cible*.
- **Passe** : chacun choisit en secret une carte de sa main et la passe à
  son voisin de gauche.
- **Mise** : chacun choisit en secret une carte à miser ; on révèle
  ensemble.
- **Prise** : la plus basse mise *unique* prend la cible et l'ajoute à sa
  main ; des mises à égalité ne peuvent pas gagner. Sans mise unique, la
  cible est défaussée. Le 1 est imbattable mais ne prend pas : qui gagne
  avec le 1 doit donner la cible à un autre joueur.
- Les mises sont défaussées.

### Kitty douce [R]
- Après la première, chaque cible compte deux cartes (1-2-2-2 : quatre
  manches au lieu de sept). La plus basse mise unique choisit une carte, la
  deuxième plus basse prend l'autre ; une carte non prise est défaussée.

### Score [R]
- Qui a le plus de cartes d'un rang en main marque la valeur de ce rang ; à
  égalité, chacun la marque en entier. Le plus haut total gagne.

### Choix sur les points non tranchés
1. ❓ La variante dépend du nombre de joueurs : dure à 4 (règles de base),
   douce à 3, 5 et 6 (le livret la propose pour jouer de 3 à 6).
2. ❓ Kitty douce : qui gagne avec le 1 choisit la carte qu'il donne, et la
   deuxième mise unique prend l'autre.
3. ❓ Une partie se joue en une seule donne ; classement aux points,
   égalités conservées.

## Octopus (`pairs-octopus`, 4 à 6 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Octopus ».

### Mise en place [R]
- Chacun mise 1 jeton. Exceptions : le perdant de la main précédente mise
  5 ; à la première main, et après une main sans perdant, tout le monde
  mise 2.
- Chacun reçoit 8 cartes.

### Déclaration [R]
- Chacun à son tour, en partant du gagnant de la main précédente (du
  donneur à la première main), reste ou se couche. Se coucher coûte 1
  jeton au pot ; c'est la seule occasion de le faire.
- Si tous se couchent sauf un, celui-là prend le pot et ouvre la main
  suivante (où chacun mise 2).

### Tours [R]
- Chacun choisit une carte de sa main en secret ; on révèle ensemble.
- La plus basse carte *unique* décide : **passer** (chacun passe une carte
  au joueur encore en jeu à sa gauche, puis nouveau tour) ou **abattre**.
- Sans carte unique, on rejoue un tour, sauf s'il ne reste qu'une carte en
  main : abattage immédiat.

### Abattage [R]
- Mains *de séries* : aucune carte seule (99-888-33). Mains *propres* :
  aucun doublon (3-4-5-7-9). Mains *sales* : le reste, toujours les pires.
- Qui appelle l'abattage choisit si les séries ou les propres sont les
  meilleures.
- À type égal, les cartes hautes sont mauvaises : on compare la plus haute,
  puis la suivante, etc. (999-44 perd contre 99-888).
- La meilleure main prend le pot (les ex æquo le partagent) ; la pire
  mise 5 à la main suivante (les ex æquo aussi).
- Dernière main : la pénalité du perdant va au gagnant plutôt qu'au pot
  suivant.

### Choix sur les points non tranchés
1. ❓ Une partie se joue en 5 mains (`Config { hands }`, plafond 15) ; la
   dernière applique la règle « Last Hand ». Chacun part de 0 jeton et peut
   passer en négatif ; classement final aux jetons, égalités conservées.
2. ❓ La carte jouée à chaque tour est défaussée face visible (le livret
   l'implique : les mains finissent par n'avoir qu'une carte).
3. ❓ L'abattage est immédiat dès que les mains n'ont plus qu'une carte,
   qu'il y ait une carte unique ou non ; une main d'une carte est propre et
   la plus basse gagne.
4. ❓ Un pot partagé laisse ses jetons impairs dans le pot.
5. ❓ Si toutes les mains se valent, elles partagent le pot et personne ne
   paie de pénalité.
6. ❓ Plusieurs gagnants : le premier d'entre eux en partant de celui qui
   parlait en premier ouvre la main suivante.

## Port (`pairs-port`, 2 à 7 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Port » (jeu frère de
Starboard, paru dans le deck *Pirate Pairs*).

### Mise en place [R]
- On mélange et on brûle 5 cartes ; on joue plusieurs manches sur le même
  mélange, et on brûle 5 cartes à chaque nouveau mélange, comme à Pairs.
- Chaque manche, chacun reçoit une carte visible. La plus basse commence ;
  les égalités se départagent comme à Pairs (une carte de plus aux ex
  æquo, une carte qui ferait paire est remplacée, ces cartes restent dans
  la pile).

### Tour [R]
- Prendre une carte ou s'arrêter. Une paire fait sauter le joueur : il ne
  marquera rien à cette manche. Qui s'arrête ne reçoit plus de carte.
- La manche continue jusqu'à ce que tout le monde se soit arrêté ou ait
  sauté.

### Score [R]
- Le plus gros total de cartes marque N points (N : le nombre de joueurs),
  le suivant N−1, et ainsi de suite jusqu'à 1. À égalité, chacun reçoit le
  plus haut des deux. Qui a sauté ne marque rien. (23, 22, 22, 17 et un
  saut à cinq : 5, 4, 4, 2, 0.)
- On joue jusqu'à 21 points. Si plusieurs joueurs sont à égalité au-delà de
  20, eux seuls rejouent une manche pour se départager.

### Choix sur les points non tranchés
1. ❓ Dans une manche de départage, N est le nombre de joueurs de la
   manche. On rejoue tant que l'égalité en tête demeure.
2. ❓ S'il n'y a plus aucune carte à piocher (toutes en jeu), on ne peut
   que s'arrêter.
3. ❓ `Config { target }` change les 21 points (plafond 99).

## Starboard (`pairs-starboard`, 2 à 7 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Starboard » (jeu frère
de Port).

### Mise en place [R]
- Chacun mise 1 jeton au pot. Le bouton, tiré au hasard à la première
  manche, avance d'un siège vers la gauche à chaque manche.
- On mélange et on pose deux cartes visibles au milieu ; si elles font une
  paire, on en défausse une et on la remplace. C'est l'unique main que
  tout le monde joue.

### Tour [R]
- En partant du bouton, chacun à son tour **tire** ou **se bloque**.
- Tirer : payer 1 jeton au pot et poser une carte au milieu. Si elle fait
  une paire, le tireur est éliminé (la carte est défaussée). Sinon, tous
  les joueurs bloqués sont éliminés.
- Se bloquer : prétendre que le milieu ne peut plus prendre de carte sans
  paire. On ne rejoue plus de la manche.
- La manche finit quand chacun a sauté ou s'est bloqué ; le dernier joueur
  encore en jeu se bloque d'office.
- Les survivants se partagent le pot ; les jetons impairs restent pour le
  pot suivant.

### Choix sur les points non tranchés
1. ❓ Une partie se joue en deux tours de bouton (2 manches par joueur,
   `Config { rounds }`, plafond 30). Chacun part de 0 jeton et peut passer
   en négatif ; classement final aux jetons, égalités conservées.
2. ❓ Si personne ne survit, tout le pot reste pour la manche suivante.
3. ❓ Chaque manche se joue sur un paquet complet fraîchement mélangé.

## Sweep (`pairs-sweep`, 3 à 6 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Sweep » (jeu proche de
Kitty).

### Mise en place [R]
- Chacun reçoit 5 cartes. Un joueur tiré au sort commence la première
  manche.

### Manche [R]
- On pose 5 cartes visibles au milieu : les *cibles*.
- **Tir** : en partant de celui qui a ramassé la dernière cible à la manche
  précédente, chacun à son tour joue une carte de sa main face visible, son
  *tireur*.
- **Prise**, dans l'ordre croissant des tireurs : le plus bas ramasse toutes
  les cibles du rang le plus haut ; chacun des autres ramasse toutes les
  cibles plus basses que lui. Les cartes ramassées vont en main. Après sa
  prise, un tireur tombe au milieu et devient une cible.
- Tireurs de même rang : ils ramassent à tour de rôle, en commençant par
  celui qui a joué le premier. Plusieurs plus bas tireurs à égalité se
  partagent ainsi les cibles du rang le plus haut de la donne d'origine.
- Les cibles restantes sont défaussées. On recommence jusqu'à épuiser le
  paquet (11 − N manches à N joueurs).

### Score [R]
- Qui a le plus de cartes d'un rang marque la valeur du rang ; à égalité,
  chacun la marque.

### Choix sur les points non tranchés
1. ❓ À tour de rôle, des tireurs à égalité ramassent une cible chacun, au
   choix quand elles n'ont pas toutes le même rang, jusqu'à ce qu'il n'en
   reste plus à leur portée ; puis ils tombent au milieu ensemble.
2. ❓ Un joueur sans carte en main ne tire pas.
3. ❓ Le score compte toute la main (cartes reçues et ramassées). Une
   partie se joue en une seule donne ; classement aux points.
4. ❓ Si personne ne ramasse rien, le même joueur commence la manche
   suivante.

## Rocket (`pairs-rocket`, 1 à 6 joueurs contre la banque)

Source : **[R]** *Pairs Companion Book*, chapitre « Rocket » (un seul paquet
Pairs, sans l'option du bouton).

### Manche [R]
- Chaque joueur mise ; les paiements se comptent en dixièmes de la mise.
- Une carte visible à chaque joueur, puis à la banque. On commence à gauche
  de la banque (le premier siège).
- À son tour, un joueur **tire** ou **se couche**. Se coucher coûte la plus
  basse carte de sa pile ou de celle de la banque. Tirer une paire coûte la
  valeur de la carte, et sort le joueur de la manche.
- La banque agit après tous les joueurs et tire toujours. Une paire lui fait
  payer sa valeur à chaque joueur encore en jeu. Un 3 la tue aussitôt, même
  en première carte : elle paie 3 à chaque joueur encore en jeu (le 3 n'a
  rien de spécial pour les joueurs).
- Sinon, on reprend au premier joueur. La manche finit quand tous les
  joueurs sont sortis, ou quand la banque perd.
- On remélange quand il reste environ 20 % du paquet.

### Choix sur les points non tranchés
1. ❓ Tout le monde mise 10 : un dixième de mise vaut 1 jeton.
2. ❓ Une partie se joue en 10 manches (`Config { rounds }`, plafond 30) ;
   chacun part de 0 jeton contre la banque et peut passer en négatif ;
   classement final aux jetons, égalités conservées.
3. ❓ Le paquet est remélangé en début de manche s'il reste moins de 11
   cartes (20 % de 55).
4. ❓ S'il n'y a plus aucune carte à tirer (toutes en jeu), on ne peut que
   se coucher ; si c'est la banque qui ne peut plus tirer, la manche finit
   sans paiement.

## Monster (`pairs-monster`, 4 à 8 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Monster ».

### Partie [R]
- Une *partie* dure jusqu'à ce que quelqu'un gagne le pot. On mise au début
  de la partie : 2 jetons à 4 ou 5 joueurs, 1 jeton de 6 à 8. Le bouton
  avance après chaque partie.

### Manche [R]
- On mélange et on donne 6 cartes à chaque joueur encore en jeu. Le joueur
  à gauche du bouton agit le premier.
- Chacun agit une fois, à son tour : écarter 0 à 3 cartes, ou se coucher.
  On ne peut pas garder une main qui contient une paire ; si c'est le seul
  choix, on doit se coucher. Les cartes écartées vont face cachée au
  milieu : c'est la main du Monstre. Les cartes d'un joueur couché sortent
  du jeu (pas au Monstre). Ce n'est pas du poker : les cartes écartées ne
  sont pas remplacées.

### Abattage [R]
- Les mains se comparent par la plus haute carte, puis la suivante, etc. ;
  une carte vaut mieux que rien (10-8-7-4 bat 10-8-7). Le Monstre ne compte
  qu'une carte par rang.
- La meilleure main est le *Champion*. Pour prendre le pot, il doit battre
  le Monstre ; le Monstre gagne les égalités. Battu, le Champion paie 5
  jetons au pot et sort de la partie.
- Les joueurs couchés sortent aussi ; les autres rejouent une manche, avec
  un nouveau mélange (le Monstre ne garde pas ses cartes).
- S'il ne reste qu'un joueur, il gagne. S'il n'en reste aucun (tous couchés,
  ou le dernier n'a pas battu ses propres écarts), les joueurs de la manche
  la rejouent.
- Deux Champions identiques qui gagnent partagent le pot ; s'ils perdent,
  ils paient 5 chacun et sortent, sauf s'ils sont les deux derniers : ils
  paient mais rejouent.

### Choix sur les points non tranchés
1. ❓ Un match compte 3 parties (`Config { games }`, plafond 15). Chacun
   part de 0 jeton et peut passer en négatif ; classement final aux jetons,
   égalités conservées.
2. ❓ Un pot partagé laisse ses jetons impairs pour la partie suivante.
3. ❓ Le siège 0 a le bouton à la première partie.
4. ❓ Le cas des deux derniers Champions battus est généralisé : quand plus
   personne ne reste, les joueurs de la manche la rejouent.

## Troll Guts (`pairs-troll-guts`, 3 à 6 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Troll Guts » (une version
plus chaotique de Goblin Poker : on ne peut pas se coucher).

### Mise en place [R]
- Chacun mise 1 jeton, reçoit 5 cartes face cachée et une carte visible. La
  plus basse carte visible jouera en premier, puis on tourne à gauche ; les
  égalités se départagent comme à Pairs, en gardant les cartes en plus.
- Chacun empile ses cartes cachées dans l'ordre où elles seront révélées.
  Une fois rangée, une pile ne se range plus (on peut toujours la
  regarder).

### Tour [R]
- Chacun prend une carte visible, d'une de trois façons : **piocher** (la
  pioche), **voler** (le dessus de la pile cachée de quelqu'un), **acheter**
  (payer 1 jeton au pot et prendre le dessus de sa propre pile).
- Si la carte fait une paire avec ses cartes visibles, le joueur perd : il
  paie la valeur de la paire au pot et retourne ses cartes face cachée.
- Le survivant prend le pot.

### Choix sur les points non tranchés
1. ❓ Une partie se joue en 5 mains (`Config { hands }`, plafond 15).
   Chacun part de 0 jeton et peut passer en négatif ; classement final aux
   jetons, égalités conservées.
2. ❓ On range sa pile en secret, tous en même temps, après avoir vu les
   cartes visibles ; on ne vole que la pile d'un joueur encore en jeu.
3. ❓ Si plus aucune carte ne peut être prise (pioche, défausse et piles
   vides), les survivants se partagent le pot ; les jetons impairs restent.
4. ❓ Les cartes d'un joueur éliminé vont à la défausse, face cachée ; une
   pioche vide est refaite avec la défausse.

## Blackstone (`pairs-blackstone`, 2 à 8 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Blackstone » (le plus
vieux jeu du livret, joué à l'origine avec un vrai sac de pierres).

### Principe [R]
- Les 10 sont des *pierres noires*, toutes les autres cartes des *pierres
  blanches*. La main s'arrête dès qu'un joueur a deux pierres noires : il a
  perdu.

### Mise en place [R]
- Chacun mise 5 jetons, laissés devant lui tant qu'il est en jeu.
- On mélange et on donne une carte visible à chacun ; la plus basse
  commence. Les égalités se départagent avec des cartes en plus, comme à
  Pairs, mais les paires de pierres blanches restent (elles ne font pas de
  mal). Qui reçoit plus d'un 10 pendant le départage n'est pas éliminé : les
  10 en trop retournent au paquet, remélangé une fois le premier joueur
  désigné.
- Les pierres blanches vont au milieu ; chacun garde ses pierres noires.

### Tour [R]
- **Prendre** : annoncer un nombre de cartes, au moins autant que le joueur
  précédent (au moins une pour le premier), avant d'en voir aucune. Les
  cartes viennent une à une : les blanches au milieu, les noires gardées. On
  arrête de distribuer dès la seconde pierre noire.
- **Se coucher** : quitter la main en payant au pot sa mise plus la moitié
  des pierres blanches distribuées, arrondie au-dessus.
- Le perdant paie sa mise plus un jeton par pierre blanche distribuée. Les
  survivants reprennent leur mise et partagent le pot ; les jetons impairs
  restent pour la main suivante. S'il ne reste qu'un joueur, il gagne.

### Choix sur les points non tranchés
1. ❓ Une partie se joue en 5 mains (`Config { hands }`, plafond 15).
   Chacun part de 0 jeton et peut passer en négatif ; classement final aux
   jetons, égalités conservées.
2. ❓ On ne peut pas annoncer plus de cartes qu'il n'en reste au paquet.
   (Le paquet ne s'épuise jamais avant la fin : au plus 8 pierres noires
   sont tenues sans perdre, il y en a 10.)
3. ❓ « Le joueur précédent » est le dernier qui a pris des cartes : se
   coucher ne change pas le minimum.

## Horn of Plenty (`pairs-horn-of-plenty`, 2 à 6 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « Horn of Plenty ».

### Mise en place [R]
- On mélange et on donne à chacun cinq cartes : une visible, qui ouvre sa
  main visible, et quatre cachées, que lui seul regarde.
- La plus basse carte visible commence. Égalité : des cartes en plus aux ex
  æquo, comme à Pairs, puis défaussées. On joue ensuite vers la gauche.

### Tour [R]
- Poser une carte cachée face cachée et annoncer son rang, vrai ou faux.
- En partant de la gauche du poseur, chacun à son tour est d'accord, pas
  d'accord, ou passe. Le premier qui ne passe pas fait révéler la carte :
  s'il a vu juste, il choisit qui la prend dans sa main visible, lui ou le
  poseur ; sinon le poseur choisit, lui ou le répondant.
- Si tout le monde passe, la carte est révélée et le poseur la place où il
  veut.
- Deux cartes de même rang dans une main visible sont défaussées aussitôt.

### Score [R]
- La manche finit quand toutes les cartes cachées sont jouées. Chaque carte
  visible vaut son rang ; la meilleure main marque autant de points de
  partie que de joueurs, la suivante un de moins, et ainsi de suite. Des
  mains à égalité prennent toutes la valeur la plus haute.
- La partie se joue en 11 points ; deux joueurs à égalité avec un score
  gagnant jouent une autre manche.

### Choix sur les points non tranchés
1. ❓ Chaque manche est une nouvelle donne : la plus basse carte visible
   désigne à nouveau le premier joueur.
2. ❓ Garde-fou : la partie s'arrête après 30 manches, égalités en tête
   conservées.
3. ❓ « Où il veut » : dans la main visible de n'importe quel joueur, la
   sienne comprise.
4. ❓ Après une égalité, la place suivante est sautée (décision de
   l'utilisateur : 5-5-3-2-1 à cinq joueurs). L'égalité en tête rejoue une
   manche avec tous les joueurs, et le plus haut total à 11 ou plus gagne.

## The Judge (`pairs-the-judge`, 2 à 6 joueurs)

Source : **[R]** *Pairs Companion Book*, chapitre « The Judge » (règle de
base ; la variante « The Judge Variant » n'est pas implémentée, décision de
l'utilisateur).

### Mise en place [R]
- On mélange et on donne 7 cartes à chacun, puis une pile cachée de 8
  cartes au milieu : le Juge.
- Le donneur commence la première manche ; ensuite, celui qui a le plus
  marqué à la manche précédente.

### Cuisine [R]
- À son tour, on ajoute une carte de sa main face visible à sa recette, ou
  on passe. Qui passe a fini de cuisiner pour la manche.
- Une recette ne peut pas contenir de paire.
- L'étape finit quand tout le monde a passé.

### Concours [R]
- On retourne le Juge carte par carte : toute recette qui contient ce rang
  est éliminée. On s'arrête quand le Juge est vide ou qu'il ne reste qu'un
  joueur.
- Les recettes survivantes marquent la somme de leurs rangs. Si une même
  carte élimine toutes les recettes restantes, personne ne marque, et le
  même joueur commence la manche suivante.
- On joue jusqu'à ce que quelqu'un atteigne la cible : 50 points à 2 ou 3
  joueurs, 40 à 4, 30 à 5 ou 6.

### Choix sur les points non tranchés
1. ❓ Le donneur de la première manche est tiré au sort. Égalité pour le
   meilleur score d'une manche : le premier des ex æquo dans l'ordre du tour
   à partir de l'ancien premier joueur commence.
2. ❓ Égalité en tête au-dessus de la cible : on joue une autre manche ;
   sinon le plus haut total gagne.
3. ❓ Un joueur dont aucune carte ne peut entrer dans sa recette passe de
   lui-même.
4. ❓ Une recette vide n'entre pas au concours : elle ne marque rien et ne
   compte pas pour « il ne reste qu'un joueur ». Une seule recette en lice :
   elle marque sans que le Juge soit retourné.
5. ❓ Garde-fou : la partie s'arrête après 40 manches, au classement du
   moment.

## Bread Basket (`pairs-bread-basket`, 2 à 6 joueurs)

Source : **[B]** *Bread Basket Rules* v3.11, Crab Fragment Labs (James
Ernest), 29/10/2025. Elle remplace la fiche mensuelle *Lunch Rush* (Cheapass
2018). Décision de l'utilisateur : la version 2025 entière, avec son paquet.

### Matériel [B]
- Un paquet propre au jeu de 55 cartes : cinq exemplaires de chaque rang de
  1 à 10 et cinq Oignons, qui valent −5. L'Oignon est la carte la plus
  basse.

### Mise en place [B]
- On mélange et on donne 10, 9, 8, 7 ou 6 cartes à chacun, pour 2, 3, 4, 5
  ou 6 joueurs. Une carte de la pioche ouvre la file au centre.
- Le joueur à gauche du donneur commence.

### Tour [B]
- On pose une carte de sa main au bout de la file, ou on pioche une carte.
- **Sandwich** : une suite d'au moins trois cartes dont la première et la
  dernière sont égales et toutes celles du milieu plus basses. Qui le
  complète le ramasse, même sans le vouloir : les deux pains sont défaussés
  et ne valent rien, les cartes du milieu restent face visible devant lui
  et comptent leur valeur (un Oignon −5).
- **Sandwich d'air** : deux cartes égales côte à côte (deux Oignons
  compris). On les défausse et on doit piocher une carte.
- Si la file est vide, la pioche y remet une carte.

### Fin de la main [B]
- Si la pioche est vide, la main s'arrête tout de suite.
- Si un joueur n'a plus de cartes, chacun des autres joue encore un tour,
  puis la main s'arrête.
- Chacun ajoute les points ramassés et retire ses cartes restées en main
  (un Oignon en main compte −5, soit 5 points de moins). Les scores peuvent
  être négatifs.
- La donne passe à gauche. La partie dure 6 mains, ou une donne par joueur
  (option du lobby). Le plus haut total gagne.

### Choix sur les points non tranchés
1. ❓ Le premier donneur est tiré au sort.
2. ❓ Après un sandwich d'air, on pioche avant de remettre une carte dans la
   file. Si la pioche est vide à ce moment-là, la main s'arrête.
3. ❓ Si un autre joueur vide sa main pendant le dernier tour de table, rien
   ne change : la main s'arrête après le dernier joueur de ce tour.
4. ❓ Égalité en tête en fin de partie : victoire partagée.
5. La permission de défausser les cartes qui s'annulent (un Oignon contre
   5 points) ne sert qu'à la table : le site fait le compte.

## The Cave (`pairs-the-cave`, 2 à 6 joueurs)

Source : **[C]** fiche mensuelle Cheapass 2018 n° 4, *The Cave*.

### Mise en place [C]
- On donne 7, 6, 5, 5 ou 4 cartes pour 2, 3, 4, 5 ou 6 joueurs, puis 5
  cartes face visible au centre : la grotte.
- Le joueur à gauche du donneur commence ; on joue dans le sens horaire.

### Tour [C]
- On pose une carte de sa main. S'il y a plus bas dans la grotte, elle
  capture toutes les cartes du rang le plus proche en dessous (grotte
  4-7-7-8-9 et un 8 : les deux 7). Sinon, elle capture toutes les cartes du
  rang le plus haut, même s'il est égal au sien.
- La carte posée entre dans la grotte, que la pioche complète à 5 cartes.
  Pioche vide : la grotte n'est plus complétée et le jeu continue.

### Fin de manche et partie [C]
- La manche finit quand toutes les mains sont vides. Pour chaque rang, le ou
  les joueurs qui en ont capturé le plus marquent 1 point par carte de ce
  rang ; les autres ne marquent rien pour ce rang.
- Une manche par joueur, la donne passe à gauche ; le plus haut total gagne.

### Choix sur les points non tranchés
1. ❓ Le premier donneur est tiré au sort.
2. ❓ Une carte posée dans une grotte vide n'y capture rien.
3. ❓ Égalité en tête : victoire partagée.

## Pickup (`pairs-pickup`, 2 à 6 joueurs)

Source : **[P]** fiche mensuelle Cheapass 2018 n° 11, *Pickup*.

### Mise en place [P]
- On donne 7, 6 ou 5 cartes pour 2, 3–4 ou 5–6 joueurs.

### Tour [P]
- 5 cartes de la pioche forment la réserve, face visible.
- Chacun choisit en secret une carte de sa main, puis on révèle.
- Les cartes agissent de la plus basse à la plus haute. À égalité, le joueur
  le plus proche à gauche de celui qui a agi en dernier passe d'abord (au
  premier coup d'un tour, à gauche du dernier à avoir agi au tour
  précédent ; au tout premier coup, à gauche du donneur).
- **Prise** : on prend une carte de la réserve de rang inférieur ou égal à
  la sienne et on la garde dans sa pile ; la carte jouée est défaussée.
  Toutes les cartes de ce rang chez les autres joueurs et dans la réserve
  sont défaussées.
- **Empilement** : rien de prenable (ou réserve vide) : la carte jouée va
  dans sa propre pile, sans rien éliminer.
- Fin du tour : le reste de la réserve est défaussé, une nouvelle est
  donnée.

### Fin [P]
- La donne finit quand les mains sont vides ; chaque carte d'une pile vaut
  sa valeur. Partie courte : une donne. Partie longue (option du lobby) :
  on redonne jusqu'à 150, 100 ou 75 points à 2, 3–4 ou 5–6 joueurs.

### Choix sur les points non tranchés
1. ❓ Le joueur choisit la carte prise à son rang d'action ; s'il n'y a
   qu'un rang prenable, le moteur la prend sans demander.
2. ❓ Pioche trop courte : la réserve prend ce qui reste.
3. ❓ Partie longue : elle s'arrête à la fin de la donne où quelqu'un atteint
   la cible, le plus haut total gagne ; garde-fou de 30 donnes. La donne
   passe à gauche.
4. ❓ Le premier donneur est tiré au sort ; égalité en tête : victoire
   partagée.

## Market Day (`pairs-market-day`, 2 à 8 joueurs)

Source : **[M]** fiche mensuelle Cheapass 2018 n° 6, *Market Day*.

### Mise en place [M]
- 6 cartes chacun ; à 2 ou 3 joueurs, 10 cartes, et chacun pose deux cartes
  par tour (on fait deux fois le tour de table).

### Tour [M]
- Chacun pose une carte face visible, à tour de rôle. Au premier tour de la
  main, le joueur à gauche du donneur commence ; ensuite celui qui a le plus
  marqué au tour précédent (égalité : le même meneur).
- Chaque carte vaut son rang. Un rang posé plusieurs fois : seul le dernier
  marque, les précédents sont gâchés.
- **1 Bricoleur** : en fin de tour, il peut échanger sa place avec une autre
  carte.
- **2 Chevalier** : avant le compte, il tue la carte qui marque le plus ;
  jamais un rang 3 ou moins. Un second Chevalier ne gâche pas le premier,
  chacun tue une carte.
- **3 Royal** : jamais échangé, tué ni gâché ; il marque toujours.
- **7 Pluie** : une Pluie seule gâche tout sauf les Royaux (1 et 2 compris)
  et marque elle-même ; plusieurs Pluies sont des 7 ordinaires.
- **9 Bandit** : le premier Bandit marque, les suivants sont gâchés.
- Ordre : retirer les cartes gâchées, puis Royaux, Pluie, Chevaliers,
  Bricoleur.

### Fin [M]
- La main finit quand toutes les cartes sont posées. La partie compte six
  mains, ou une par joueur s'ils sont plus de six ; la donne passe à
  gauche. Le plus haut total gagne.

### Choix sur les points non tranchés
1. ❓ Bricoleur (lecture validée par l'utilisateur) : son joueur peut
   échanger son 1 avec la carte d'un autre joueur qui marque encore, sauf un
   Royal ; chacun marque la carte reçue. Il peut aussi garder. Un Bricoleur
   gâché (doublon, Pluie) n'agit pas.
2. ❓ Les Chevaliers tuent l'un après l'autre, dans l'ordre de pose, la
   carte de plus haut rang (4 ou plus) qui marque encore ; à rang égal, la
   dernière posée.
3. ❓ À 2–3 joueurs, le meneur pose sa première carte, puis tout le tour de
   table, puis sa seconde, etc. ; « le plus marqué » compte les deux cartes.
4. ❓ Premier donneur tiré au sort ; égalité en tête : victoire partagée.
