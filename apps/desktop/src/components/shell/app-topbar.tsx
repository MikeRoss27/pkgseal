import { useEffect, useState } from "react";
import { Button } from "@/components/ui/ui/button";
import { Separator } from "@/components/ui/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/ui/tooltip";
import { ThemeToggle } from "@/components/shell/theme-toggle";
import { CommandPalette } from "@/components/shell/command-palette";
import { WindowControls } from "@/components/shell/window-controls";
import { Command, PanelLeft, Eye } from "lucide-react";
import { uiStore } from "@/store/ui-store";
import { getPaletteShortcutLabel } from "@/lib/keyboard";
import pkgsealMark from "@/assets/pkgseal-mark.svg";
import { getCurrentWindow } from "@tauri-apps/api/window";

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function handleTitleBarDoubleClick() {
  if (!isTauriRuntime()) return;
  void getCurrentWindow().toggleMaximize();
}

function dispatchFocusSearch() {
  const w = window as unknown as { __pkgseal_focusSearch?: () => void };
  if (w.__pkgseal_focusSearch) w.__pkgseal_focusSearch();
  else window.dispatchEvent(new CustomEvent("pkgseal:focus-search"));
}

export function AppTopbar({ onFocusSearch }: { onFocusSearch?: () => void }) {
  const [paletteOpen, setPaletteOpen] = useState(false);

  const handleFocusSearch = () => {
    if (onFocusSearch) onFocusSearch();
    dispatchFocusSearch();
  };

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPaletteOpen((v) => !v);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <>
      {/* The native OS title bar is disabled (decorations: false) — this row is
          the whole title bar: draggable background, window controls flush right. */}
      <header
        data-tauri-drag-region
        onDoubleClick={handleTitleBarDoubleClick}
        className="cn:flex cn:h-11 cn:shrink-0 cn:items-center cn:justify-between cn:border-b cn:border-border cn:bg-background"
      >
        <div className="cn:flex cn:items-center cn:gap-2 cn:min-w-0 cn:pl-3">
          <Button
            variant="ghost"
            size="icon-sm"
            className="cn:md:hidden cn:shrink-0"
            aria-label="Toggle sidebar"
            onClick={() => uiStore.toggleSidebar()}
          >
            <PanelLeft className="cn:size-3.5" />
          </Button>
          {/* The sidebar already carries the PkgSeal mark on desktop — only repeat it
              here on mobile, where the sidebar is hidden entirely. */}
          <img src={pkgsealMark} alt="" width={40} height={40} className="cn:md:hidden" />
          <span className="cn:md:hidden cn:text-[13px] cn:font-semibold cn:tracking-tight">
            PkgSeal
          </span>
        </div>

        <div className="cn:flex cn:items-center cn:shrink-0">
          <div className="cn:flex cn:items-center cn:gap-2 cn:pr-2">
            <Tooltip>
              <TooltipTrigger
                render={
                  <span className="cn:hidden cn:sm:inline-flex cn:items-center cn:gap-1.5 cn:rounded-full cn:border cn:border-border cn:px-2 cn:py-0.5 cn:text-[11px] cn:font-medium cn:text-muted-foreground" />
                }
              >
                <Eye className="cn:size-3" /> Preview · Read only
              </TooltipTrigger>
              <TooltipContent>
                PkgSeal does not install or remove packages yet — every result is inspect-only.
              </TooltipContent>
            </Tooltip>

            <Separator orientation="vertical" className="cn:h-3.5 cn:hidden cn:sm:block" />

            <Tooltip>
              <TooltipTrigger
                render={
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label="Open command palette"
                    onClick={() => setPaletteOpen(true)}
                  />
                }
              >
                <Command className="cn:size-3.5" />
              </TooltipTrigger>
              <TooltipContent>Command palette ({getPaletteShortcutLabel()})</TooltipContent>
            </Tooltip>

            <ThemeToggle />

            <span className="cn:hidden cn:lg:inline cn:text-[10px] cn:font-mono cn:text-muted-foreground/50">
              v0.1.0-alpha
            </span>
          </div>

          <WindowControls />
        </div>
      </header>

      <CommandPalette
        open={paletteOpen}
        onOpenChange={setPaletteOpen}
        onFocusSearch={handleFocusSearch}
      />
    </>
  );
}
