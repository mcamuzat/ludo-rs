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
