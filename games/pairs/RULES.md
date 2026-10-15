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
1. ❓ Une partie se joue en plusieurs mains (5 par défaut). Chacun part de 0
   jeton et peut passer en négatif ; classement final aux jetons, égalités
   conservées.
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
