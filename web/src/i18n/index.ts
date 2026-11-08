// The site's language: French or English, chosen on the home screen and
// remembered in this browser. It never travels over the network.
import { useSyncExternalStore } from "react";
import { en } from "./en";
import { fr, type Dict } from "./fr";

export type { Dict } from "./fr";
export type Lang = "fr" | "en";

const DICTS: Record<Lang, Dict> = { fr, en };
const KEY = "ludo-lang";

/** French for a French browser, English otherwise. */
export function detectLang(language: string | undefined): Lang {
  return language?.toLowerCase().startsWith("fr") ? "fr" : "en";
}

/** The stored choice, or null when absent, unknown or unreadable. */
export function readStoredLang(read: () => string | null): Lang | null {
  try {
    const value = read();
    return value === "fr" || value === "en" ? value : null;
  } catch {
    return null;
  }
}

let current: Lang | null = null;
const listeners = new Set<() => void>();

export function getLang(): Lang {
  current ??=
    readStoredLang(() => localStorage.getItem(KEY)) ??
    detectLang(typeof navigator === "undefined" ? undefined : navigator.language);
  return current;
}

export function setLang(lang: Lang): void {
  current = lang;
  try {
    localStorage.setItem(KEY, lang);
  } catch {
    // Private window or blocked storage: the choice lasts until reload.
  }
  for (const listener of listeners) listener();
}

/** The current dictionary, for code outside React (sessions). */
export function t(): Dict {
  return DICTS[getLang()];
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useLang(): Lang {
  return useSyncExternalStore(subscribe, getLang, getLang);
}

/** The current dictionary in a component, re-rendered when it changes. */
export function useT(): Dict {
  return DICTS[useLang()];
}

/** A game's name in the current language (the engine's name by default). */
export function gameName(meta: { id: string; name: string }): string {
  return t().gameNames[meta.id] ?? meta.name;
}
