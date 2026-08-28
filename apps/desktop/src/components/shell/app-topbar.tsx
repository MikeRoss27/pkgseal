import { useEffect, useState } from "react"
import { Button } from "@/components/ui/ui/button"
import { Kbd } from "@/components/ui/ui/kbd"
import { Separator } from "@/components/ui/ui/separator"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/ui/tooltip"
import { StatusBar } from "@/components/shell/status-bar"
import { ThemeToggle } from "@/components/shell/theme-toggle"
import { CommandPalette } from "@/components/shell/command-palette"
import { Search, Command, PanelLeft } from "lucide-react"
import { uiStore } from "@/store/ui-store"
import { getPaletteShortcutLabel } from "@/lib/keyboard"

function dispatchFocusSearch() {
  // Prefer imperative handle if DiscoverPage registered it, fallback to custom event
  const w = window as unknown as { __pkgseal_focusSearch?: () => void }
  if (w.__pkgseal_focusSearch) w.__pkgseal_focusSearch()
  else window.dispatchEvent(new CustomEvent("pkgseal:focus-search"))
}

export function AppTopbar({ onFocusSearch }: { onFocusSearch?: () => void }) {
  const [paletteOpen, setPaletteOpen] = useState(false)

  const handleFocusSearch = () => {
    if (onFocusSearch) onFocusSearch()
    dispatchFocusSearch()
  }

  // Global hotkey for palette
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault()
        setPaletteOpen((v) => !v)
      }
    }
    window.addEventListener("keydown", handler)
    return () => window.removeEventListener("keydown", handler)
  }, [])

  return (
    <>
      <header className="cn:sticky cn:top-0 cn:z-30 cn:flex cn:h-14 cn:items-center cn:justify-between cn:gap-3 cn:border-b cn:border-border/80 cn:bg-background/80 cn:backdrop-blur-xl cn:px-4 cn:md:px-6">
        <div className="cn:flex cn:items-center cn:gap-3 cn:min-w-0">
          <Button variant="ghost" size="icon" className="cn:md:hidden cn:shrink-0" aria-label="Toggle sidebar" onClick={() => uiStore.toggleSidebar()}>
            <PanelLeft className="cn:size-4" />
          </Button>
          <div className="cn:flex cn:items-center cn:gap-2 cn:md:hidden">
            <div className="cn:size-7 cn:rounded-lg cn:bg-foreground cn:text-background cn:grid cn:place-items-center">
              <Search className="cn:size-3.5" />
            </div>
            <span className="cn:text-sm cn:font-semibold">PkgSeal</span>
          </div>
          {/* Breadcrumb / title */}
          <div className="cn:hidden cn:md:flex cn:items-center cn:gap-2 cn:min-w-0">
            <h2 className="cn:text-sm cn:font-semibold cn:text-foreground cn:tracking-tight">Discover</h2>
            <span className="cn:hidden cn:sm:inline cn:text-muted-foreground/40">/</span>
            <span className="cn:hidden cn:sm:inline cn:text-sm cn:text-muted-foreground">Search & resolve applications</span>
          </div>
        </div>

        <div className="cn:flex cn:items-center cn:gap-1.5 cn:shrink-0">
          {/* Search affordance */}
          <Tooltip>
            <TooltipTrigger
              render={
                <Button variant="outline" size="sm" className="cn:gap-2 cn:text-muted-foreground cn:font-normal cn:h-8 cn:px-2.5 cn:rounded-full cn:md:rounded-lg" onClick={handleFocusSearch} />
              }
            >
              <Search className="cn:size-3.5" />
              <span className="cn:hidden cn:md:inline">Search</span>
              <Kbd className="cn:hidden cn:lg:inline-flex cn:ml-1">/</Kbd>
            </TooltipTrigger>
            <TooltipContent>Focus search (/)</TooltipContent>
          </Tooltip>

          <Separator orientation="vertical" className="cn:h-4 cn:mx-1 cn:hidden cn:md:block" />

          <Tooltip>
            <TooltipTrigger
              render={
                <Button variant="ghost" size="icon" aria-label="Open command palette" onClick={() => setPaletteOpen(true)} />
              }
            >
              <Command className="cn:size-4" />
            </TooltipTrigger>
            <TooltipContent>Command palette ({getPaletteShortcutLabel()})</TooltipContent>
          </Tooltip>

          <StatusBar />

          <Separator orientation="vertical" className="cn:h-4 cn:mx-1 cn:hidden cn:sm:block" />

          <ThemeToggle />
        </div>
      </header>

      <CommandPalette open={paletteOpen} onOpenChange={setPaletteOpen} onFocusSearch={handleFocusSearch} />
    </>
  )
}
