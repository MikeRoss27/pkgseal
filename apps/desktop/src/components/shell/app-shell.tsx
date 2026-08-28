import { type ReactNode, useRef } from "react"
import { Badge } from "@/components/ui/ui/badge"
import { AppSidebar } from "@/components/shell/app-sidebar"
import { AppTopbar } from "@/components/shell/app-topbar"
import { TooltipProvider } from "@/components/ui/ui/tooltip"

interface AppShellProps {
  children: ReactNode
  onFocusSearch?: () => void
}

export function AppShell({ children, onFocusSearch }: AppShellProps) {
  const searchRef = useRef<(() => void) | null>(null)

  const handleFocusSearch = () => {
    if (onFocusSearch) onFocusSearch()
    else searchRef.current?.()
  }

  return (
    <TooltipProvider delay={200}>
      <div className="cn:min-h-screen cn:bg-background cn:flex cn:selection:bg-foreground cn:selection:text-background">
        <AppSidebar />
        <div className="cn:flex cn:flex-1 cn:min-w-0 cn:flex-col">
          <AppTopbar onFocusSearch={handleFocusSearch} />
          <main className="cn:flex-1 cn:px-4 cn:py-6 cn:md:px-6 cn:lg:px-8 cn:max-w-[1400px] cn:w-full cn:mx-auto">
            {children}
          </main>
          <footer className="cn:border-t cn:border-border/60 cn:bg-muted/20 cn:px-4 cn:py-2.5 cn:md:px-6 cn:flex cn:items-center cn:justify-between cn:text-xs cn:text-muted-foreground cn:gap-3">
            <span className="cn:flex cn:items-center cn:gap-2">
              PkgSeal • Read-only preview
              <Badge variant="secondary" className="cn:text-[10px] cn:h-5 cn:px-1.5 cn:font-medium">
                v0.1.0-alpha
              </Badge>
              <span className="cn:hidden cn:sm:inline">— no system changes are performed in this milestone.</span>
            </span>
            <span className="cn:hidden cn:md:inline">Evidence → Policy → Recommendation</span>
          </footer>
        </div>
      </div>
    </TooltipProvider>
  )
}
