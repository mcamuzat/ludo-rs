// Short rule reminders for the Pairs games, shown at the table. The full
// rules and their sources: games/pairs/RULES.md (keep both in step).

export const CONTINUOUS_RULES = [
  "À ton tour : prends une carte de la pioche, ou couche-toi.",
  "Si la carte fait une paire avec ta pile, tu marques son rang et ta pile est défaussée.",
  "Te coucher : prends n'importe quelle carte en jeu, marque son rang, et défausse ta pile.",
  "Le premier à atteindre le score cible perd : 31 à 2 joueurs, 21 à 3, 16 à 4, 13 à 5, 11 au-delà.",
];

export const AUBERGE_RULES = [
  "Chacun a 3 cartes en main et en pose une visible devant lui ; la plus petite commence.",
  "Attaquer : pose une carte de ta main devant un autre joueur, sans y faire de paire. C'est à lui.",
  "Défier : pose la carte de la pioche devant un autre joueur. Si elle fait une paire, c'est toi qui es éliminé.",
  "Accuser : s'il a une paire entre sa main et ses cartes visibles, il est éliminé ; sinon, c'est toi.",
  "Piocher : une carte en main, en secret.",
  "Le dernier en jeu gagne la manche et marque 1 point ; 5 manches.",
];

export const GHOST_TOWN_RULES = [
  "Chacun mise 10 jetons, reçoit 5 cartes et en pose une en secret : ensemble, elles forment la rue (doublons défaussés). La plus petite commence.",
  "Jouer : paie 2 jetons et pose une carte dans ta maison, sans paire ni rang de la rue.",
  "Piocher : la carte va dans ta maison. Paire avec ta maison ou la rue : tu paies son rang et tu sors. Sinon tu gagnes son rang en jetons, pris au pot.",
  "Se coucher : quitter la main.",
  "La main finit quand le pot est vide, ou quand il reste un seul joueur, qui prend le pot. 5 mains par défaut (réglable dans le lobby), classement aux jetons.",
];

export const ZOMBIE_LIFE_RULES = [
  "Chacun commence avec une carte dans son Refuge ; la plus petite commence.",
  "Piocher : pose la carte dans ton Refuge ou dans la Horde, jamais là où son rang est déjà. Rang déjà dans les deux : tu es submergé et éliminé.",
  "Prendre une carte de la Horde pour ton Refuge, si son rang n'y est pas.",
  "Passer : tu te caches et retournes autant de cartes qu'il reste de joueurs en jeu ; une revient face visible à chaque élimination.",
  "En fin de manche, ceux qui ont passé marquent 1 point par carte visible. Le premier à 21 points gagne.",
];

export const GOBLIN_POKER_RULES = [
  "Chacun mise 1 jeton, reçoit 6 cartes et commence sa pile avec une carte choisie en secret.",
  "En partant de la pile la plus basse : jette une carte face cachée au centre, ou couche-toi en payant la plus petite carte en jeu.",
  "Les cartes jetées sont mélangées et redonnées, une par joueur : qui reçoit une paire paie son rang et sort.",
  "Le dernier en jeu prend le pot ; après 5 tours, les survivants le partagent. 5 mains, classement aux jetons.",
];

export const TO_THE_MOON_RULES = [
  "Chacun reçoit une main (8 cartes à 2 joueurs, une de moins par joueur en plus). Un compte à rebours a au plus une carte par rang.",
  "Construire : pose une carte de ta main d'un rang absent. Compléter les dix rangs gagne le pli aussitôt.",
  "Tester : retourne la carte de la pioche. Rang déjà là : tu sors du pli, cette carte en main.",
  "Te retirer : sors du pli en prenant une carte du compte à rebours (sauf s'il n'y en a qu'une).",
  "Le dernier en jeu gagne le pli, 1 point par carte, et lance le suivant. Pioche vide : fin de manche. 6 manches.",
];

export const DEADFALL_RULES = [
  "Chacun mise 1 jeton, reçoit 6 cartes (7 à deux, 5 à sept) et choisit en secret une carte de porte ; la plus basse commence.",
  "Jouer une carte sur ta pile, c'est prétendre qu'il en reste de ce rang dans une main.",
  "Appeler la carte du dessus d'un autre, c'est dire que son rang est mort : plus aucune en main, la tienne comprise. De deux cartes du dessus de même rang, seule la dernière jouée peut être appelée.",
  "Abattage : qui a tort paie le rang à l'autre, qui prend aussi le pot. 5 mains, classement aux jetons.",
];

export const CAPTAINS_ORDERS_RULES = [
  "6 cartes chacun. Le Capitaine tient la carte la plus basse de toutes les mains (égalité : plusieurs Capitaines) ; les autres sont des Mutins.",
  "Sans ordre : pose une carte comme ordre, passe une carte (le receveur en défausse une), demande une carte (puis défausses-en une), ou accuse.",
  "Ordre en cours, chacun à son tour : obéis en donnant au propriétaire une carte au moins égale, annule avec un ordre plus bas, ou accuse. Revenu à son propriétaire, l'ordre est défaussé.",
  "Main vide : tu te déclares Capitaine, et gagnes si c'est vrai. Accuser juste fait gagner un Mutin ; sinon le Mutin qui a le plus de points (son rang par carte) gagne, sauf celui qui s'est trompé.",
  "Le premier à 3 manches gagnées l'emporte.",
];

export const PENDLETON_RULES = [
  "Cinq cibles de rangs différents, triées. À ton tour, tu pioches une carte et la poses de ton côté d'une colonne.",
  "Une colonne n'accepte pas une carte dont le rang y est déjà : la cible, ou une carte de l'un ou l'autre côté.",
  "La manche s'arrête quand quelqu'un pioche une carte injouable partout.",
  "Dans chaque colonne, qui a posé le plus de points marque la valeur de la cible ; à égalité, chacun la moitié (arrondie en dessous).",
  "Qui a marqué le moins commence la manche suivante. Le premier à 100 points gagne.",
];

export const KITTY_RULES = [
  "Sept cartes face cachée forment la kitty ; le reste est distribué. À chaque manche, on retourne la cible.",
  "Chacun passe une carte à son voisin de gauche, puis mise une carte en secret ; on révèle ensemble.",
  "La plus basse mise unique prend la cible (les mises à égalité ne gagnent pas). Gagner avec le 1 oblige à donner la cible à un autre joueur.",
  "Kitty douce (à 3, 5 ou 6) : cibles de 1, 2, 2 et 2 cartes ; la plus basse mise unique choisit, la deuxième prend l'autre.",
  "À la fin, qui a le plus de cartes d'un rang marque sa valeur (égalité : chacun la marque). Le plus haut total gagne.",
];

export const OCTOPUS_RULES = [
  "Mise : 2 jetons à la première main et après une main gagnée par abandon ; sinon 1, et 5 pour le perdant de la main précédente. 8 cartes chacun.",
  "Chacun à son tour reste ou se couche (1 jeton). Si tous les autres se couchent, le dernier prend le pot.",
  "À chaque tour, chacun joue une carte en secret. La plus basse carte unique décide : faire passer une carte à gauche, ou l'abattage.",
  "Mains de séries (aucune carte seule), propres (aucun doublon) ou sales (le reste, toujours les pires). Qui appelle l'abattage choisit si les séries ou les propres gagnent ; à type égal, les cartes hautes sont mauvaises.",
  "La meilleure main prend le pot, la pire paiera 5 à la main suivante. 5 mains, classement aux jetons.",
];

export const PORT_RULES = [
  "Chacun reçoit une carte visible ; la plus basse commence (égalité : des cartes de plus, comme à Pairs).",
  "À ton tour : prends une carte, ou arrête-toi. Une paire te fait sauter : 0 point pour la manche.",
  "Quand tout le monde s'est arrêté ou a sauté, le plus gros total de cartes marque N points (N = nombre de joueurs), le suivant N−1, et ainsi de suite ; à égalité, chacun marque le rang le plus haut.",
  "Le premier à 21 points gagne ; les ex æquo au-delà de 20 rejouent une manche entre eux.",
];

export const STARBOARD_RULES = [
  "Chacun mise 1 jeton ; deux cartes de rangs différents vont au milieu : c'est la main que tout le monde joue.",
  "En partant du bouton, chacun à son tour tire ou se bloque. Tirer coûte 1 jeton et ajoute une carte au milieu.",
  "Si la carte fait une paire, tu es éliminé. Sinon, tous les joueurs bloqués sont éliminés.",
  "Se bloquer, c'est parier que le milieu ne peut plus prendre de carte sans paire. Le dernier en jeu se bloque d'office.",
  "Les survivants bloqués se partagent le pot (les jetons impairs restent). Le bouton tourne ; deux tours de table, classement aux jetons.",
];

export const SWEEP_RULES = [
  "5 cartes chacun. À chaque manche, 5 cibles visibles ; chacun à son tour tire une carte de sa main, face visible.",
  "Le plus bas tireur ramasse toutes les cibles du rang le plus haut ; puis, du plus bas au plus haut, chaque tireur ramasse toutes les cibles plus basses que lui. Après avoir tiré, un tireur devient une cible.",
  "Tireurs à égalité : ils ramassent une cible chacun à tour de rôle, en commençant par celui qui a joué le premier.",
  "Ce qui reste est défaussé ; qui a ramassé la dernière cible commence la manche suivante. On joue jusqu'au bout du paquet.",
  "À la fin, qui a le plus de cartes d'un rang en main marque sa valeur (égalité : chacun la marque).",
];

export const ROCKET_RULES = [
  "Chacun joue contre la banque, qui suit des règles fixes. Une carte visible à chacun, puis à la banque.",
  "À ton tour : tirer, ou te coucher en payant la plus basse carte en vue (ta pile ou celle de la banque).",
  "Une paire te coûte sa valeur, et tu sors de la manche.",
  "Après les joueurs, la banque tire toujours : une paire, ou un 3 (même en première carte), lui fait payer sa valeur à chaque joueur encore en jeu. Sinon on recommence le tour.",
  "10 manches, classement aux jetons.",
];

export const MONSTER_RULES = [
  "Mise de départ : 2 jetons à 4 ou 5 joueurs, 1 au-delà. Six cartes chacun ; on commence à gauche du bouton.",
  "Chacun à son tour écarte 0 à 3 cartes face cachée, qui forment la main du Monstre, ou se couche. Une main gardée ne doit contenir aucune paire.",
  "Abattage : la plus haute main (plus haute carte, puis la suivante… ; une carte vaut mieux que rien) est le Champion. Le Monstre ne compte chaque rang qu'une fois, et gagne les égalités.",
  "Le Champion qui bat le Monstre prend le pot. Sinon il paie 5 jetons et sort ; les joueurs couchés sortent aussi, et les autres rejouent une manche.",
  "Le dernier joueur restant gagne le pot. 3 parties, classement aux jetons.",
];

export const TROLL_GUTS_RULES = [
  "Chacun mise 1 jeton et reçoit 5 cartes cachées et une visible ; la plus basse jouera en premier.",
  "D'abord, chacun range en secret sa pile cachée dans l'ordre où ses cartes seront révélées. Plus de retour en arrière ensuite.",
  "À ton tour, prends une carte visible : piocher, voler le dessus de la pile d'un autre, ou acheter le dessus de la tienne pour 1 jeton.",
  "Si la carte fait une paire avec tes cartes visibles, tu paies sa valeur au pot et tu sors. On ne peut pas se coucher.",
  "Le dernier survivant prend le pot. 5 mains, classement aux jetons.",
];

export const BLACKSTONE_RULES = [
  "Les 10 sont des pierres noires, le reste des pierres blanches. Qui en a deux noires perd la main.",
  "Chacun mise 5 jetons, gardés devant soi, et reçoit une carte : la plus basse commence. On ne garde que ses pierres noires ; les blanches vont au milieu.",
  "À ton tour : annonce un nombre de cartes, au moins autant que le joueur précédent (1 pour le premier), puis tire-les une à une. Une seconde pierre noire arrête tout.",
  "Te coucher coûte ta mise plus la moitié des pierres blanches (arrondie au-dessus). Le perdant paie sa mise plus un jeton par pierre blanche.",
  "Les survivants reprennent leur mise et partagent le pot. 5 mains, classement aux jetons.",
];

export const HORN_OF_PLENTY_RULES = [
  "Chacun reçoit une carte visible et quatre cachées ; la plus basse visible commence.",
  "À ton tour : pose une carte cachée et annonce son rang, vrai ou faux.",
  "En partant de ta gauche, chacun est d'accord, pas d'accord, ou passe. Qui répond juste choisit qui prend la carte (lui ou toi) ; sinon c'est toi qui choisis. Si tout le monde passe, tu la places chez qui tu veux.",
  "Une paire dans une main visible est défaussée aussitôt.",
  "Fin de manche : la meilleure main visible (somme des rangs) marque autant de points que de joueurs, la suivante un de moins ; les égalités prennent la valeur haute. Partie en 11 points.",
];

export const THE_JUDGE_RULES = [
  "Chacun reçoit 7 cartes ; 8 cartes cachées forment le Juge.",
  "À ton tour : ajoute une carte de ta main à ta recette, face visible, ou passe. Qui passe a fini pour la manche. Pas de paire dans une recette.",
  "Quand tout le monde a passé, on retourne le Juge carte par carte : chaque carte élimine les recettes de son rang, jusqu'à ce qu'il n'en reste qu'une.",
  "Les recettes survivantes marquent la somme de leurs rangs. Le meilleur de la manche commence la suivante.",
  "Partie en 50 points à 2 ou 3 joueurs, 40 à 4, 30 à 5 ou 6.",
];
