# Jeux de cartes traditionnels — règles implémentées

Sources :

- **[W]** Wikipédia, « Trou du cul (jeu) » (la page « Président (Jeu de
  cartes) » y redirige), consulté le 2026-10-15.
- **[P]** Pagat.com, « President », https://www.pagat.com/climbing/president.html,
  consulté le 2026-10-15.

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
