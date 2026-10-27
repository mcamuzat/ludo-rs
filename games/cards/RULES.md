# Jeux de cartes traditionnels — règles implémentées

Sources :

- **[W]** Wikipédia, « Trou du cul (jeu) » (la page « Président (Jeu de
  cartes) » y redirige), consulté le 2026-10-15.
- **[P]** Pagat.com, « President », https://www.pagat.com/climbing/president.html,
  consulté le 2026-10-15.
- **[C]** David Parlett, « Collusion »,
  https://www.parlettgames.uk/oricards/collude.html, consultée le 2026-10-16.
- **[L]** James Ernest et Dave Howell, « Lamarckian Poker », *The Cheapass
  Games Poker Suite* (Crab Fragment Labs, 2011, 2022), p. 10 ; « Seven-Handed
  Poker », p. 12, pour « une carte d'appoint vaut mieux que rien ».
- **[S]** *Stamp Showdown*, la reprise de Lamarckian Poker : comptes rendus
  de cardgamer.com (« Become a Master Philatelist in Stamp Showdown ») et de
  tabletopping.games (2026-01-26), consultés le 2026-10-23 ; la règle
  elle-même n'a pas été consultée.
- **[H]** « Hattrick », règle parue dans *Jeux en boîte, la revue des jeux de
  société*, mise en ligne par l'Escale à jeux (`hattr_rg.pdf`), consultée le
  2026-10-27.

Un point que les sources ne tranchent pas est marqué ❓ ; ceux du Président
ont été validés par l'utilisateur le 2026-10-15 (spec 4, §3).

## Président (`cards-president`, 4 à 7 joueurs)

### Matériel et ordre

- [W] Paquet de 52 cartes (54 avec les jokers). Les couleurs ne comptent pas.
- [W] Ordre : 3 < 4 < … < 10 < valet < dame < roi < as < 2.

### Donne

- [W] Chacun reçoit le même nombre de cartes ; les cartes en trop sont
  montrées à tous et écartées pour la manche.
- [W] Première manche : le siège 0 donne ; ensuite, le Trou du cul. Le joueur
  à gauche du donneur entame.
- Places fixes : ce sont les parties « statiques » de [W].

### Jeu

- [W] L'entame pose une à quatre cartes de même valeur. Le suivant passe, ou
  pose autant de cartes d'une valeur strictement supérieure.
- [P] Passer n'empêche pas de jouer à son tour suivant dans le même pli.
- [W] Quand tous les autres joueurs encore en jeu ont passé à la suite, le
  dernier à avoir joué entame le pli suivant ; s'il est sorti, la main passe
  au joueur à sa gauche.
- Le moteur choisit les couleurs des cartes posées ou rendues (pique, cœur,
  carreau, trèfle, dans cet ordre) : elles ne changent rien au jeu.

### Titres et points

- [W] Premier sorti Président, deuxième Vice-Président, avant-dernier
  Secrétaire, dernier Trou du cul ; les autres sont Neutres. Le dernier
  joueur qui a encore des cartes est Trou du cul sans jouer.
- [W] Points : n−1 au Président, un de moins par place, 0 au Trou du cul.
- ❓1 La première manche rapporte des points comme les autres ([W] la
  réserve à fixer les titres).
- Fin de partie : après N manches (option, 5 par défaut, de 1 à 10) ;
  classement aux points, les ex-aequo au même rang.

### Échanges (à partir de la 2ᵉ manche)

- [W] Après la donne, le Trou du cul donne ses deux meilleures cartes au
  Président, qui lui rend deux cartes de son choix ; le Secrétaire donne sa
  meilleure carte au Vice-Président, qui lui rend une carte de son choix ;
  les Neutres n'échangent rien.
- Le Président et le Vice-Président choisissent en même temps, dans toute
  leur main. Pour l'échange, l'ordre normal compte et le joker est la
  meilleure carte.

### Variantes (options, désactivées par défaut)

- **Révolution** [W] : quatre cartes de même valeur inversent l'ordre (le 3
  devient la plus forte, le 2 la plus faible) jusqu'à la fin de la manche ;
  les jokers forts restent au-dessus de tout. ❓2 Un nouveau carré inverse de
  nouveau l'ordre. ❓8 Le changement d'ordre vaut aussi pour la combinaison
  déjà sur la table : après un carré de 9, seul un carré plus faible dans
  l'ordre normal peut le battre.
- **Interdiction de finir sur un 2** [W] : qui pose sa dernière carte avec un
  ou plusieurs 2, ou un ou plusieurs jokers, est Trou du cul.
  ❓3 Pendant une révolution, la carte interdite est le 3 ; les jokers restent
  interdits. ❓4 La carte interdite se juge selon l'ordre en vigueur quand
  elle est posée, avant la révolution que ce coup déclenche éventuellement.
  ❓5 Plusieurs fautifs se classent derrière tous les autres joueurs, le
  premier fautif tout en bas.
- **Jokers forts** [W][P] : au-dessus du 2 ; seuls ou à deux, jamais avec une
  autre valeur.
- **Jokers wild** [P] : le joker prend la valeur des cartes qu'il accompagne
  (quatre cartes au plus). Seuls, ou à deux, les jokers sont la combinaison
  la plus forte de leur taille. ❓6 Un joker vaut une carte naturelle de même
  valeur : à égalité, il faut monter au-dessus. ❓7 Un carré complété par des
  jokers déclenche la révolution.

### Hors périmètre

Putsch, passage de tour, « 2 utilisé comme atout », « … ou rien »,
changement de places, échanges négociés, parties à 3 joueurs ou à plus de 7.

## Collusion (`cards-collusion`, 4 joueurs)

- **Matériel** [C] : paquet de 52 cartes ; dans chaque couleur, A R D V 10 9 8
  7 6 5 4 3 2.
- **Donne** [C] : 13 cartes à chacun.
- ❓ Le siège 0 donne la première donne, puis la donne passe à gauche.
- **Jeu** [C] : le joueur à gauche du donneur entame le premier pli, le
  gagnant de chaque pli entame le suivant. Il faut fournir à la couleur si on
  peut, sinon on joue n'importe quelle carte. Pas d'atout : la plus forte
  carte de la couleur demandée gagne.
- **But** [C] : finir avec exactement autant de plis qu'un autre joueur (mais
  pas que deux autres).
- **Score d'une donne** [C] : un point par pli, plus au plus un bonus :
  - exactement deux joueurs au même nombre de plis : 10 à chacun d'eux ;
  - aucun joueur au même nombre qu'un autre : 20 à celui qui en a le moins ;
  - trois joueurs au même nombre : 30 au quatrième.

  Les quatre totaux font 13 : ni deux paires ni égalité à quatre.
- **Cible** [C] : 100 points, ou 50 en option. On n'atteint la cible qu'avec
  un bonus : sans bonus, des plis qui l'atteindraient sont retranchés du
  total au lieu d'y être ajoutés. [C] le dit pour 100 ; on l'applique à la
  cible choisie.
- ❓ **Fin** : la partie s'arrête après la donne où un joueur atteint la
  cible ; le plus haut total gagne, les ex æquo partagent leur rang.
- **Parler** [C] : on peut s'entendre à voix haute, sans rien dire de sa main,
  et rien n'engage. Sur le site, à son tour et avant de jouer, on peut dire
  une phrase : « Je vise N plis » (de ses plis pris à ses plis pris plus ses
  cartes en main), « Je ne sais pas encore » ou « Qui vise N ? » (0 à 13). On
  ne parle pas quand on n'a qu'une carte jouable. Les phrases s'effacent à la
  donne suivante.

## Lamarckian Poker (`cards-lamarckian`, 2 à 6 joueurs)

Règle de [L] avec les trois changements de [S].

- **Paquet** [S] : on retire de petites cartes selon le nombre de joueurs, de
  sorte que le paquet dure sept tours. ❓1 On retire les deux jusqu'aux
  (7 − joueurs) : rien à 6, les deux à 5, … les deux à cinq à 2 ; le paquet
  fait alors 4 × joueurs + 28 cartes, exactement les mains de départ et sept
  réserves.
- **Concours** [S] : quatre concours de sept tours, chacun suivi d'un
  abattage ; les points s'additionnent. Option : 1 à 4 concours.
- **Donne** [L] : chacun reçoit 4 cartes ; le paquet est remélangé à chaque
  concours.
- **Tour** [L] : 4 cartes face visible forment la réserve. Chacun pose une
  carte de sa main face cachée (sa « marque ») ; quand tous ont choisi, on les
  révèle. Personne ne sait qui a déjà choisi.
- **Ordre** [L][S] : les marques jouent de la plus forte à la plus faible, as
  fort ; à valeur égale, pique, cœur, carreau, trèfle.
- **Prise** [L] : une marque prend toutes les cartes de la réserve de sa
  valeur ou de sa couleur, puis tombe dans la réserve (les suivantes peuvent
  la prendre).
- **Rien à prendre** [S] : la marque prend la carte de la réserve de son
  choix. ❓4 Ce choix est obligatoire (une carte de plus ne nuit jamais : pas
  de limite de main, on garde les cinq meilleures) ; s'il n'y a qu'une carte,
  elle est prise sans demander. La réserve n'est jamais vide à ce moment.
- **Fin du tour** [L] : ce qui reste dans la réserve est défaussé.
- Une main ne rétrécit donc jamais : la règle d'extinction de [L] ne
  s'applique plus.
- **Cartes vues** : les cartes prises dans la réserve sont connues de tous ;
  le site les montre devant chaque joueur. Les cartes de départ restent
  cachées.
- **Abattage** [S] : chacun montre sa meilleure main de poker (cinq cartes au
  plus, ordre de [L] ; l'as compte aussi bas dans A-2-3-4-5). Une main de
  moins de cinq cartes joue ce qu'elle a. La meilleure main marque autant de
  points qu'il y a de joueurs, la suivante un de moins, et ainsi de suite.
  ❓2 [S] dit à la fois « jusqu'à zéro pour le dernier » et, en exemple à 4
  joueurs, « 4, 3, 2, 1 » : on suit l'exemple (seuls les écarts comptent).
  ❓3 Des mains égales (les couleurs ne départagent pas) partagent la
  meilleure des places qu'elles occupent.
- **Fin** [S] : après le dernier concours, le plus de points gagne ; ❓5 les
  ex æquo partagent leur rang.

## Hattrick (`cards-hattrick`, 4 à 6 joueurs)

- **Matériel** [H] : 60 cartes, de 1 à 20 en trois couleurs. [H] ne nomme
  pas les couleurs ; son exemple parle de rouges, bleues et vertes, que le
  site reprend.
- **But** [H] : réunir le plus de cartes d'une couleur et éviter les deux
  autres.
- **Donne** [H] : 15 cartes à 4 joueurs, 12 à 5, 10 à 6 (tout le paquet). Le
  joueur à gauche du donneur commence ; la donne passe à gauche à chaque
  manche. ❓ Le siège 0 donne la première manche.
- **Tour** [H] : le premier joueur pose la carte de son choix, puis chacun à
  son tour dans le sens horaire pose une carte de la même couleur ou d'une
  autre. Une carte d'une deuxième couleur ouvre une deuxième pile ; une
  troisième couleur est interdite. Le tour finit quand tous ont joué (ou
  passé) une fois.
- **Passer** [H] : seulement quand deux piles sont ouvertes, et au choix du
  joueur (« ne peut pas ou ne veut pas »). Il montre à tous une carte de la
  troisième couleur et la pose face cachée devant lui. Le site garde la carte
  montrée visible dans le tour en cours et le dernier tour.
- **Prise** [H] : la plus forte carte de chaque pile la remporte ; avec deux
  piles, deux gagnants. Les cartes gagnées restent en vue devant leur
  gagnant, une pile par couleur : le site montre leur nombre par couleur.
- **Tour suivant** [H] : le joueur de la carte la plus forte du tour commence
  le suivant ; si les deux piles ont la même plus forte carte, la 2e, puis la
  3e carte la plus forte de chaque pile départage. ❓ Une pile qui n'a plus
  de carte à comparer perd ; deux piles identiques donnent la main à la pile
  ouverte en premier.
- **Dernière carte** [H] : la dernière carte de chaque joueur n'est pas jouée
  et ne compte pas.
- **Points** [H] : la couleur dont on a le plus de cartes compte +1 par
  carte, les deux autres −1 par carte, chaque carte posée en passant −2,
  quelle que soit sa couleur (exemple de [H] : 9 rouges, 2 bleues, 4 vertes
  et une carte cachée font 9 − 2 − 4 − 2 = 1). Si deux couleurs sont à
  égalité en tête, le résultat est le même quelle que soit celle qui compte.
- **Fin** [H] : après deux fois plus de manches que de joueurs ; le plus haut
  total gagne, les ex æquo partagent leur rang.
- **Variante non retenue** [H] : jouer jusqu'à un total de points fixé. Les
  scores pouvant baisser, une partie pourrait ne jamais finir.
