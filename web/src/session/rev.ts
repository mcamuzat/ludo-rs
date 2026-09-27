/** The action was computed for a state the game has since left. */
export class StaleActionError extends Error {
  constructor() {
    super("La partie a avancé entre-temps : coup ignoré.");
    this.name = "StaleActionError";
  }
}

/** An action is only valid for the revision (of its seat's update) it was chosen at. */
export function isStale(actionRev: number, currentRev: number): boolean {
  return actionRev !== currentRev;
}
