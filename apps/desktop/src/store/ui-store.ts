import { useSyncExternalStore } from "react"

export type UiState = {
  sidebarOpen: boolean
  commandPaletteOpen: boolean
  density: "compact" | "comfortable"
}

type Listener = () => void

const defaultState: UiState = {
  sidebarOpen: true,
  commandPaletteOpen: false,
  density: "comfortable",
}

let state: UiState = { ...defaultState }
const listeners = new Set<Listener>()

function emit(): void {
  for (const l of listeners) l()
}

function subscribe(listener: Listener): () => void {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

function getSnapshot(): UiState {
  return state
}

function setState(patch: Partial<UiState>): void {
  const next = { ...state, ...patch }
  // shallow compare
  if (
    next.sidebarOpen === state.sidebarOpen &&
    next.commandPaletteOpen === state.commandPaletteOpen &&
    next.density === state.density
  ) {
    return
  }
  state = next
  emit()
}

// ── selectors / actions ────────────────────────────────────────────

export function useUiStore(): UiState {
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot)
}

export function useUiStoreSelector<T>(selector: (s: UiState) => T): T {
  return useSyncExternalStore(subscribe, () => selector(getSnapshot()), () => selector(getSnapshot()))
}

export const uiStore = {
  getState: getSnapshot,
  subscribe,
  setState,
  setSidebarOpen: (open: boolean) => setState({ sidebarOpen: open }),
  toggleSidebar: () => setState({ sidebarOpen: !state.sidebarOpen }),
  setCommandPaletteOpen: (open: boolean) => setState({ commandPaletteOpen: open }),
  toggleCommandPalette: () => setState({ commandPaletteOpen: !state.commandPaletteOpen }),
  setDensity: (density: UiState["density"]) => setState({ density }),
  reset: () => {
    state = { ...defaultState }
    emit()
  },
  /** For tests only */
  _resetForTests: () => {
    state = { ...defaultState }
    emit()
  },
}
