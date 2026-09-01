import { useEffect, useState } from "react"
import { getCurrentWindow } from "@tauri-apps/api/window"

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
}

/**
 * Custom minimize/maximize/close controls for the borderless window
 * (`decorations: false` in tauri.conf.json — the native title bar was
 * removed because it didn't match the app's theme). No-op outside Tauri.
 */
export function WindowControls() {
  const [maximized, setMaximized] = useState(false)
  const [available, setAvailable] = useState(false)

  useEffect(() => {
    if (!isTauriRuntime()) return
    const win = getCurrentWindow()
    let unlisten: (() => void) | undefined
    void (async () => {
      setMaximized(await win.isMaximized())
      unlisten = await win.onResized(async () => {
        setMaximized(await win.isMaximized())
      })
      setAvailable(true)
    })()
    return () => unlisten?.()
  }, [])

  if (!available) return null

  return (
    <div className="cn:flex cn:items-center cn:-mr-1">
      <button
        type="button"
        aria-label="Minimize"
        onClick={() => void getCurrentWindow().minimize()}
        className="cn:inline-flex cn:h-11 cn:w-9 cn:items-center cn:justify-center cn:text-muted-foreground cn:hover:bg-muted cn:hover:text-foreground cn:transition-colors"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <path d="M1 5h8" stroke="currentColor" strokeWidth="1" />
        </svg>
      </button>
      <button
        type="button"
        aria-label={maximized ? "Restore" : "Maximize"}
        onClick={() => void getCurrentWindow().toggleMaximize()}
        className="cn:inline-flex cn:h-11 cn:w-9 cn:items-center cn:justify-center cn:text-muted-foreground cn:hover:bg-muted cn:hover:text-foreground cn:transition-colors"
      >
        {maximized ? (
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path d="M2.5 0.5h7v7h-2M0.5 2.5h7v7h-7z" fill="none" stroke="currentColor" strokeWidth="1" />
          </svg>
        ) : (
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" strokeWidth="1" />
          </svg>
        )}
      </button>
      <button
        type="button"
        aria-label="Close"
        onClick={() => void getCurrentWindow().close()}
        className="cn:inline-flex cn:h-11 cn:w-9 cn:items-center cn:justify-center cn:text-muted-foreground cn:hover:bg-destructive cn:hover:text-white cn:transition-colors"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" stroke="currentColor" strokeWidth="1" />
        </svg>
      </button>
    </div>
  )
}
