import { type ReactNode, useRef } from "react"
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
      <div className="cn:h-screen cn:bg-background cn:flex cn:selection:bg-[color:var(--brand-steel-tint)] cn:selection:text-foreground cn:overflow-hidden">
        <AppSidebar />
        <div className="cn:flex cn:flex-1 cn:min-w-0 cn:flex-col">
          <AppTopbar onFocusSearch={handleFocusSearch} />
          <main className="cn:flex-1 cn:min-h-0 cn:overflow-hidden">{children}</main>
        </div>
      </div>
    </TooltipProvider>
  )
}
