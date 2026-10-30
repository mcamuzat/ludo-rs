# Siberian Dice — règles implémentées

Sources :

- **[R]** *Siberian Dice*, Artem Borovkov et Eugene Panferov, 2015. PDF
  print-and-play, révision 2.0, mis en ligne sur BoardGameGeek. La page 1
  donne la règle de base, et la page 3 le plateau numéroté. Le site
  siberiandice.com a disparu, et le PDF n'est pas copié dans ce dépôt.
- **[U]** Décisions de l'utilisateur, le 2026-10-30.

Un point que les sources ne tranchent pas est marqué ❓. Seule la **règle de
base** est implémentée [U]. Les variantes de la page 2 (PRIMES, STRATEGIC,
GROSSMEISTER, PURITY, partie à 3) ne le sont pas.

## Matériel [R]

- Un plateau hexagonal de 37 cases, en rangées de 4, 5, 6, 7, 6, 5, 4. La
  case `ab` a deux coordonnées `a` et `b`, de 1 à 7, avec `|a − b| ≤ 3`
  (page 3). Ses voisines sont `(a±1, b±1)` (les deux signes identiques),
  `(a, b±1)` et `(a±1, b)`, quand ces cases existent.
- 12 dés par joueur : bleus pour le premier joueur, rouges pour le second.

## Dé parfait [R]

Un dé posé est **parfait** quand son nombre de points est égal au nombre de
dés posés sur les cases voisines, quelle que soit leur couleur et quelle que
soit leur valeur. Ce statut change à chaque coup. La figure d'exemple de la
page 1 est rejouée par le test `the_example_figure_of_the_rules`.

## Un coup [R][U]

- Le plateau est vide au départ, et bleu commence. On joue chacun son tour,
  et on doit jouer.
- On choisit un de ses dés, dans sa réserve ou sur le plateau. On le lance,
  puis on le pose, face obtenue visible, sur une **case vide** de son choix.
  Le dé est choisi avant le lancer, et la case après.
- Un dé repris sur le plateau libère sa case, et il **peut y être reposé**
  [U]. C'est la lecture littérale de « any vacant hex ».
- Une fois sa réserve vide, on joue un dé déjà posé. Il reste toujours au
  moins 13 cases vides.

## Fin [R][U]

- Les dés parfaits se comptent une fois le dé posé, et non pendant qu'un dé
  lancé attend d'être posé. C'est un choix du site.
- Si une seule couleur a **6 dés parfaits ou plus**, elle gagne, même quand
  c'est le coup de l'adversaire qui l'y amène [R].
- Si les deux couleurs atteignent 6 au même coup, la partie est nulle [R].
- **Limite de durée** [U] : c'est un ajout du site, absent de [R]. La partie
  est nulle après 30 tours de table (60 coups) sans **nouveau record**. Un
  record est battu quand une couleur a plus de dés parfaits qu'à tout moment
  depuis le début de la partie. Le seuil de 30 tours vient de l'utilisateur.
  La définition du record est un choix du site.
