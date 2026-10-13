# Jeux du deck Pairs — règles implémentées

Le deck : 55 cartes sans couleur, une de rang 1, deux de rang 2, … dix de
rang 10. Une *paire* : deux cartes de même rang dans une même pile.

Les points marqués ❓ ne sont pas tranchés par la source : ce sont nos choix.

## Continuous Pairs (`pairs-continuous`, 2 à 8 joueurs)

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
