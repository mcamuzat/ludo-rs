# TwixT — règles appliquées

Sources :

- **[LG]** Règle « PP » (papier-crayon) de TwixT, jouée sur Little Golem : comme la règle d'Alex Randolph, sauf que les liens ne se retirent jamais et que ses propres liens peuvent se croiser. La page d'aide de Little Golem ne répond plus ; décrite d'après Wikipédia (en.wikipedia.org/wiki/TwixT, « TwixT PP ») et le code de [TB].
- **[TB]** twixtbot, Jordan Lampe (MIT), github.com/BonyJordan/twixtbot, et sa reprise twixtbot-ui, stevens68 (MIT) : `twixt.py` (règles), réseau et MCTS.

## Plateau et camps

- [LG] 24 × 24 trous sans les quatre coins. Rouge relie le haut au bas, Noir la gauche à la droite.
- [LG] Les rangées du haut et du bas sont à Rouge, les colonnes de gauche et de droite à Noir : on ne pose jamais dans la bordure adverse.
- ❓ Le siège 0 est Rouge et commence (Rouge = WHITE de [TB], qui joue en premier).

## Tour

- [LG] On pose un pion de sa couleur dans un trou libre permis.
- [LG] Le pion est relié automatiquement à chaque pion ami à distance de cavalier, sauf si le lien croiserait un lien **adverse**. Ses propres liens peuvent se croiser ([LG], et [TB] `twixt.py` ne teste que les liens adverses). Deux liens qui se croisent ne sont pas reliés au croisement.
- [LG] Les liens ne se retirent jamais.

## Swap

- [LG] Au premier coup de Noir, il peut prendre le pion de Rouge au lieu de jouer. [TB] `play_swap` : le pion (x, y) devient un pion noir en (y, x) (reflet par la diagonale), puis Rouge joue. Les couleurs des joueurs ne changent pas.

## Fin

- [LG] Gagne qui relie ses deux bordures par une chaîne de liens.
- ❓ Nulle dès qu'aucun des deux ne peut plus relier ses bordures (choix du site, comme [TB] `_is_a_draw`) : un chemin reste possible tant qu'une suite de pas de cavalier par des trous libres ou à soi, sans croiser de lien adverse, joint les deux bordures.
- ❓ Nulle aussi si le joueur au trait n'a plus aucun trou permis (choix du site : cas extrême où l'un a rempli tous ses trous et l'autre peut encore relier ; la règle ne prévoit pas de passer).
- Pas de limite de durée : la partie finit toujours.
