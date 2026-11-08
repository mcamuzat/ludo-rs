import { t } from "../i18n";

/** The action was computed for a state the game has since left. */
export class StaleActionError extends Error {
  constructor() {
    super(t().net.reject.stale);
    this.name = "StaleActionError";
  }
}

/** An action is only valid for the revision (of its seat's update) it was chosen at. */
export function isStale(actionRev: number, currentRev: number): boolean {
  return actionRev !== currentRev;
}
