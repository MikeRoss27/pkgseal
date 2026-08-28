import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/ui/dialog"
import { Input } from "@/components/ui/ui/input"
import { Badge } from "@/components/ui/ui/badge"
import { Kbd } from "@/components/ui/ui/kbd"
import { Separator } from "@/components/ui/ui/separator"
import { Search, Sparkles, Package, ShieldCheck, History, Settings } from "lucide-react"

interface CommandPaletteProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onFocusSearch?: () => void
}

const COMMANDS = [
  { id: "search", label: "Search applications", icon: Search, hint: "Find Arch / AUR / Flatpak", action: "focus-search" as const },
  { id: "installed", label: "View installed packages", icon: Package, hint: "Coming soon", disabled: true },
  { id: "security", label: "Security overview", icon: ShieldCheck, hint: "Coming soon", disabled: true },
  { id: "transactions", label: "Transaction history", icon: History, hint: "Coming soon", disabled: true },
  { id: "settings", label: "Open settings", icon: Settings, hint: "Coming soon", disabled: true },
]

export function CommandPalette({ open, onOpenChange, onFocusSearch }: CommandPaletteProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="cn:p-0 cn:gap-0 cn:overflow-hidden cn:sm:max-w-lg cn:border cn:shadow-medium">
        <DialogHeader className="cn:sr-only">
          <DialogTitle>Command palette</DialogTitle>
          <DialogDescription>Quick actions and navigation</DialogDescription>
        </DialogHeader>

        <div className="cn:flex cn:items-center cn:gap-2 cn:border-b cn:px-3 cn:py-2.5">
          <Search className="cn:size-4 cn:text-muted-foreground cn:shrink-0" />
          <Input
            placeholder="Type a command or search..."
            autoFocus
            className="cn:h-7 cn:border-0 cn:bg-transparent cn:shadow-none cn:focus-visible:ring-0 cn:px-0"
          />
          <Kbd>ESC</Kbd>
        </div>

        <div className="cn:p-2 cn:space-y-1 cn:max-h-80 cn:overflow-auto">
          <p className="cn:px-2 cn:py-1 cn:text-[11px] cn:font-semibold cn:uppercase cn:tracking-widest cn:text-muted-foreground">Suggestions</p>
          {COMMANDS.map((cmd) => (
            <button
              key={cmd.id}
              disabled={cmd.disabled}
              onClick={() => {
                if (cmd.action === "focus-search") {
                  onOpenChange(false)
                  onFocusSearch?.()
                }
              }}
              className={`cn:flex cn:w-full cn:items-center cn:gap-3 cn:rounded-lg cn:px-2.5 cn:py-2 cn:text-left cn:transition-colors ${
                cmd.disabled ? "cn:opacity-50 cn:cursor-not-allowed" : "cn:hover:bg-accent cn:hover:text-accent-foreground"
              }`}
            >
              <span className="cn:size-7 cn:rounded-md cn:bg-muted cn:grid cn:place-items-center cn:shrink-0">
                <cmd.icon className="cn:size-3.5" />
              </span>
              <span className="cn:flex-1 cn:min-w-0">
                <span className="cn:text-sm cn:font-medium cn:block cn:truncate">{cmd.label}</span>
                <span className="cn:text-xs cn:text-muted-foreground cn:block cn:truncate">{cmd.hint}</span>
              </span>
              {cmd.disabled && <Badge variant="secondary" className="cn:text-[10px] cn:h-5">Soon</Badge>}
            </button>
          ))}

          <Separator className="cn:my-2" />

          <div className="cn:rounded-lg cn:bg-muted/50 cn:px-3 cn:py-2.5 cn:flex cn:gap-2.5 cn:items-start">
            <Sparkles className="cn:size-3.5 cn:mt-0.5 cn:text-muted-foreground" />
            <div className="cn:space-y-1">
              <p className="cn:text-xs cn:font-medium">PkgSeal tip</p>
              <p className="cn:text-xs cn:text-muted-foreground cn:leading-relaxed">
                PkgSeal doesn’t claim software is “100% safe”. It shows <span className="cn:text-foreground cn:font-medium">Evidence → Policy → Recommendation</span>.
              </p>
            </div>
          </div>
        </div>

        <div className="cn:flex cn:items-center cn:justify-between cn:border-t cn:bg-muted/30 cn:px-3 cn:py-2 cn:text-[11px] cn:text-muted-foreground">
          <span className="cn:flex cn:items-center cn:gap-1.5">
            <Kbd>↑↓</Kbd> navigate <Kbd className="cn:ml-1">↵</Kbd> select
          </span>
          <span>v0.1.0-alpha</span>
        </div>
      </DialogContent>
    </Dialog>
  )
}
