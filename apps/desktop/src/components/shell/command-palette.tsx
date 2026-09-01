import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/ui/dialog"
import { Input } from "@/components/ui/ui/input"
import { Kbd } from "@/components/ui/ui/kbd"
import { Search, SunMoon } from "lucide-react"
import { useTheme } from "@/lib/use-theme"

interface CommandPaletteProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onFocusSearch?: () => void
}

export function CommandPalette({ open, onOpenChange, onFocusSearch }: CommandPaletteProps) {
  const { toggleTheme } = useTheme()

  const commands = [
    { id: "search", label: "Search applications", hint: "Arch · AUR · Flatpak", icon: Search, run: () => onFocusSearch?.() },
    { id: "theme", label: "Toggle theme", hint: "Light / dark", icon: SunMoon, run: () => toggleTheme() },
  ]

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="cn:p-0 cn:gap-0 cn:overflow-hidden cn:sm:max-w-md cn:border cn:shadow-medium">
        <DialogHeader className="cn:sr-only">
          <DialogTitle>Command palette</DialogTitle>
          <DialogDescription>Quick actions</DialogDescription>
        </DialogHeader>

        <div className="cn:flex cn:items-center cn:gap-2 cn:border-b cn:border-border cn:px-3 cn:py-2">
          <Search className="cn:size-3.5 cn:text-muted-foreground cn:shrink-0" />
          <Input placeholder="Type a command…" autoFocus className="cn:h-6 cn:border-0 cn:bg-transparent cn:shadow-none cn:focus-visible:ring-0 cn:px-0 cn:text-[13px]" />
          <Kbd>ESC</Kbd>
        </div>

        <div className="cn:p-1.5">
          {commands.map((cmd) => (
            <button
              key={cmd.id}
              onClick={() => {
                onOpenChange(false)
                cmd.run()
              }}
              className="cn:flex cn:w-full cn:items-center cn:gap-2.5 cn:rounded-md cn:px-2 cn:py-1.5 cn:text-left cn:transition-colors cn:hover:bg-accent"
            >
              <cmd.icon className="cn:size-3.5 cn:text-muted-foreground cn:shrink-0" />
              <span className="cn:flex-1 cn:min-w-0 cn:text-[13px] cn:font-medium cn:truncate">{cmd.label}</span>
              <span className="cn:text-[11px] cn:text-muted-foreground cn:shrink-0">{cmd.hint}</span>
            </button>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  )
}
