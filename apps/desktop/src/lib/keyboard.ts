/**
 * Keyboard shortcuts vocabulary — single source of truth for labels and matchers.
 */

export const Kbd = {
  palette: "Cmd+K",
  search: "/",
  escape: "Escape",
  enter: "Enter",
} as const

export type KbdKey = (typeof Kbd)[keyof typeof Kbd]

/** Human-friendly label for the command palette shortcut (Cmd on mac, Ctrl elsewhere). */
export function getPaletteShortcutLabel(): string {
  if (typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/.test(navigator.platform)) {
    return "⌘K"
  }
  return "Ctrl+K"
}

export function isPaletteShortcut(e: KeyboardEvent): boolean {
  return (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k"
}

export function isSearchShortcut(e: KeyboardEvent): boolean {
  // "/" without modifiers when not in an input
  if (e.metaKey || e.ctrlKey || e.altKey) return false
  return e.key === "/"
}

export function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  const tag = target.tagName
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    target.isContentEditable
  )
}
