import { Badge } from "@/components/ui/ui/badge"
import { Button } from "@/components/ui/ui/button"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/ui/tooltip"
import { Separator } from "@/components/ui/ui/separator"
import { Search, Package, History, ShieldCheck, Settings, Sparkles, ChevronLeft, ChevronRight } from "lucide-react"
import { uiStore, useUiStoreSelector } from "@/store/ui-store"

type NavItem = {
  id: string
  label: string
  icon: React.ElementType
  active?: boolean
  disabled?: boolean
  badge?: string
}

const NAV_ITEMS: NavItem[] = [
  { id: "discover", label: "Discover", icon: Search, active: true },
  { id: "installed", label: "Installed", icon: Package, disabled: true, badge: "Soon" },
  { id: "transactions", label: "Transactions", icon: History, disabled: true },
  { id: "security", label: "Security", icon: ShieldCheck, disabled: true },
  { id: "settings", label: "Settings", icon: Settings, disabled: true },
]

export function AppSidebar() {
  const sidebarOpen = useUiStoreSelector((s) => s.sidebarOpen)

  if (!sidebarOpen) {
    return (
      <aside className="cn:hidden cn:md:flex cn:w-[52px] cn:shrink-0 cn:flex-col cn:items-center cn:border-r cn:border-sidebar-border cn:bg-sidebar cn:py-3 cn:gap-1">
        <div className="cn:size-8 cn:rounded-lg cn:bg-foreground cn:text-background cn:grid cn:place-items-center cn:mb-2">
          <Sparkles className="cn:size-4" />
        </div>
        {NAV_ITEMS.map((item) => (
          <Tooltip key={item.id}>
            <TooltipTrigger
              render={
                <Button
                  variant={item.active ? "secondary" : "ghost"}
                  size="icon"
                  disabled={item.disabled}
                  aria-label={item.label}
                />
              }
            >
              <item.icon className="cn:size-4" />
            </TooltipTrigger>
            <TooltipContent side="right">
              {item.label} {item.disabled ? "— coming soon" : ""}
            </TooltipContent>
          </Tooltip>
        ))}
        <div className="cn:mt-auto">
          <Button variant="ghost" size="icon" aria-label="Expand sidebar" onClick={() => uiStore.setSidebarOpen(true)}>
            <ChevronRight className="cn:size-4" />
          </Button>
        </div>
      </aside>
    )
  }

  return (
    <aside className="cn:hidden cn:md:flex cn:w-56 cn:shrink-0 cn:flex-col cn:border-r cn:border-sidebar-border cn:bg-sidebar cn:sticky cn:top-0 cn:h-screen">
      {/* Brand */}
      <div className="cn:flex cn:h-14 cn:items-center cn:justify-between cn:gap-2 cn:border-b cn:border-sidebar-border cn:px-3">
        <div className="cn:flex cn:items-center cn:gap-2.5 cn:min-w-0">
          <div className="cn:size-7 cn:rounded-lg cn:bg-foreground cn:text-background cn:grid cn:place-items-center cn:shrink-0">
            <Sparkles className="cn:size-3.5" />
          </div>
          <div className="cn:min-w-0 cn:leading-none">
            <div className="cn:text-sm cn:font-semibold cn:tracking-tight">PkgSeal</div>
            <div className="cn:text-[10px] cn:text-muted-foreground cn:font-medium cn:uppercase cn:tracking-widest">Arch • AUR • Flatpak</div>
          </div>
        </div>
        <Button variant="ghost" size="icon-sm" aria-label="Collapse sidebar" onClick={() => uiStore.setSidebarOpen(false)}>
          <ChevronLeft className="cn:size-3.5" />
        </Button>
      </div>

      {/* Navigation */}
      <nav className="cn:flex-1 cn:space-y-4 cn:p-3 cn:overflow-y-auto">
        <div className="cn:space-y-1">
          <p className="cn:px-2 cn:pb-1 cn:text-[10px] cn:font-semibold cn:uppercase cn:tracking-widest cn:text-muted-foreground">Navigate</p>
          {NAV_ITEMS.map((item) => {
            if (item.disabled) {
              return (
                <Tooltip key={item.id}>
                  <TooltipTrigger
                    render={
                      <button
                        type="button"
                        disabled
                        className={`cn:flex cn:w-full cn:items-center cn:gap-2.5 cn:rounded-lg cn:px-2.5 cn:py-2 cn:text-sm cn:font-medium cn:transition-colors cn:text-left ${
                          item.active
                            ? "cn:bg-sidebar-accent cn:text-foreground"
                            : "cn:text-muted-foreground/60 cn:cursor-not-allowed"
                        }`}
                      />
                    }
                  >
                    <item.icon className="cn:size-4 cn:shrink-0" />
                    <span className="cn:flex-1 cn:truncate">{item.label}</span>
                    {item.badge && (
                      <Badge variant="secondary" className="cn:h-4 cn:px-1 cn:text-[10px] cn:font-medium">
                        {item.badge}
                      </Badge>
                    )}
                  </TooltipTrigger>
                  <TooltipContent side="right">Coming soon</TooltipContent>
                </Tooltip>
              )
            }
            return (
              <div key={item.id}>
                <button
                  type="button"
                  className={`cn:flex cn:w-full cn:items-center cn:gap-2.5 cn:rounded-lg cn:px-2.5 cn:py-2 cn:text-sm cn:font-medium cn:transition-colors cn:text-left ${
                    item.active ? "cn:bg-sidebar-accent cn:text-foreground" : "cn:text-muted-foreground cn:hover:bg-sidebar-accent cn:hover:text-foreground"
                  }`}
                >
                  <item.icon className="cn:size-4 cn:shrink-0" />
                  <span className="cn:flex-1 cn:truncate">{item.label}</span>
                  {item.badge && (
                    <Badge variant="secondary" className="cn:h-4 cn:px-1 cn:text-[10px] cn:font-medium">
                      {item.badge}
                    </Badge>
                  )}
                </button>
              </div>
            )
          })}
        </div>

        <Separator />

        <div className="cn:rounded-xl cn:border cn:border-dashed cn:bg-muted/30 cn:p-3 cn:space-y-2">
          <p className="cn:text-xs cn:font-semibold cn:text-foreground">Read-only preview</p>
          <p className="cn:text-xs cn:leading-relaxed cn:text-muted-foreground">
            Search, resolve, and compare candidates. Install comes after the recommendation engine is proven.
          </p>
          <div className="cn:flex cn:items-center cn:gap-1.5 cn:text-[11px] cn:text-muted-foreground">
            <span className="cn:size-1.5 cn:rounded-full cn:bg-emerald-500 cn:animate-pulse" /> Policy • Resolver • Evidence
          </div>
        </div>
      </nav>

      <div className="cn:border-t cn:border-sidebar-border cn:p-3">
        <div className="cn:flex cn:items-center cn:gap-2 cn:rounded-lg cn:bg-muted/50 cn:px-2.5 cn:py-2">
          <div className="cn:size-6 cn:rounded-full cn:bg-foreground/10 cn:grid cn:place-items-center">
            <span className="cn:text-xs">◐</span>
          </div>
          <div className="cn:min-w-0 cn:flex-1">
            <p className="cn:text-xs cn:font-medium cn:truncate">Local session</p>
            <p className="cn:text-[11px] cn:text-muted-foreground">No account required</p>
          </div>
        </div>
      </div>
    </aside>
  )
}
