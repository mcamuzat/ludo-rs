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
  "La main finit quand le pot est vide, ou quand il reste un seul joueur, qui prend le pot. 5 mains, classement aux jetons.",
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
