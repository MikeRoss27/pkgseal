import { Button } from "@/components/ui/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/ui/tooltip";
import { Search, ChevronLeft, ChevronRight } from "lucide-react";
import { uiStore, useUiStoreSelector } from "@/store/ui-store";
import pkgsealMark from "@/assets/pkgseal-mark.svg";

function BrandMark({ size = 24 }: { size?: number }) {
  return <img src={pkgsealMark} alt="" width={size} height={size} className="cn:shrink-0" />;
}

// Only one destination exists today — Discover. No disabled placeholder items.
const NAV_ITEMS = [{ id: "discover", label: "Discover", icon: Search, active: true }] as const;

export function AppSidebar() {
  const sidebarOpen = useUiStoreSelector((s) => s.sidebarOpen);

  if (!sidebarOpen) {
    return (
      <aside className="cn:hidden cn:md:flex cn:w-11 cn:shrink-0 cn:flex-col cn:items-center cn:border-r cn:border-sidebar-border cn:bg-sidebar cn:py-2.5 cn:gap-1">
        <div className="cn:size-8 cn:grid cn:place-items-center cn:mb-1.5" aria-hidden="true">
          <BrandMark size={20} />
        </div>
        {NAV_ITEMS.map((item) => (
          <Tooltip key={item.id}>
            <TooltipTrigger
              render={
                <Button
                  variant={item.active ? "secondary" : "ghost"}
                  size="icon-sm"
                  aria-label={item.label}
                />
              }
            >
              <item.icon className="cn:size-3.5" />
            </TooltipTrigger>
            <TooltipContent side="right">{item.label}</TooltipContent>
          </Tooltip>
        ))}
        <div className="cn:mt-auto">
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label="Expand sidebar"
            onClick={() => uiStore.setSidebarOpen(true)}
          >
            <ChevronRight className="cn:size-3.5" />
          </Button>
        </div>
      </aside>
    );
  }

  return (
    <aside className="cn:hidden cn:md:flex cn:w-44 cn:shrink-0 cn:flex-col cn:border-r cn:border-sidebar-border cn:bg-sidebar">
      <div className="cn:flex cn:h-11 cn:items-center cn:justify-between cn:gap-2 cn:border-b cn:border-sidebar-border cn:px-3">
        <div className="cn:flex cn:items-center cn:gap-2 cn:min-w-0 cn:text-foreground">
          <BrandMark />
          <span className="cn:text-[13px] cn:font-semibold cn:tracking-tight cn:truncate">
            PkgSeal
          </span>
        </div>
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label="Collapse sidebar"
          onClick={() => uiStore.setSidebarOpen(false)}
        >
          <ChevronLeft className="cn:size-3.5" />
        </Button>
      </div>

      <nav className="cn:flex-1 cn:p-2">
        {NAV_ITEMS.map((item) => (
          <button
            key={item.id}
            type="button"
            className={`cn:flex cn:w-full cn:items-center cn:gap-2 cn:rounded-md cn:px-2 cn:py-1.5 cn:text-[13px] cn:font-medium cn:text-left cn:transition-colors ${
              item.active
                ? "cn:bg-sidebar-accent cn:text-foreground"
                : "cn:text-muted-foreground cn:hover:bg-sidebar-accent cn:hover:text-foreground"
            }`}
          >
            <item.icon className="cn:size-3.5 cn:shrink-0" />
            <span className="cn:flex-1 cn:truncate">{item.label}</span>
          </button>
        ))}
      </nav>

      <div className="cn:flex cn:h-6 cn:shrink-0 cn:items-center cn:border-t cn:border-sidebar-border cn:bg-muted/30 cn:px-3">
        <span className="cn:text-[10px] cn:font-medium cn:uppercase cn:tracking-wider cn:text-muted-foreground/70">
          Arch · AUR · Flatpak
        </span>
      </div>
    </aside>
  );
}
