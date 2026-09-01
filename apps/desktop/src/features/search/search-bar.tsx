import { useRef, useEffect } from "react"
import { Input } from "@/components/ui/ui/input"
import { Kbd } from "@/components/ui/ui/kbd"
import { Search, X, Loader2, Command } from "lucide-react"
import { cn } from "@/lib/cn"

interface SearchBarProps {
  value: string
  onValueChange: (value: string) => void
  isLoading?: boolean
  autoFocus?: boolean
  placeholder?: string
}

export function SearchBar({ value, onValueChange, isLoading, autoFocus = true, placeholder = "Search for an application (e.g. Brave, Bitwarden, Discord)" }: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null)

  // "/" focuses search when not editing; "Escape" clears
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null
      const isEditable = target instanceof HTMLElement && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)
      if (e.key === "/" && !e.metaKey && !e.ctrlKey && !e.altKey && !isEditable) {
        e.preventDefault()
        inputRef.current?.focus()
      }
      if (e.key === "Escape" && document.activeElement === inputRef.current && value) {
        onValueChange("")
      }
    }
    window.addEventListener("keydown", handler)
    return () => window.removeEventListener("keydown", handler)
  }, [value, onValueChange])

  return (
    <div className="cn:relative cn:group">
      <div className="cn:absolute cn:left-3 cn:top-1/2 cn:-translate-y-1/2 cn:pointer-events-none cn:text-muted-foreground cn:transition-colors group-focus-within:cn:text-foreground">
        {isLoading ? <Loader2 className="cn:size-4 cn:animate-spin" /> : <Search className="cn:size-4" />}
      </div>

      <Input
        ref={inputRef}
        type="text"
        role="searchbox"
        placeholder={placeholder}
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        aria-label="Search for an application"
        autoFocus={autoFocus}
        className={cn(
          "cn:h-11 cn:pl-10 cn:pr-[88px] cn:rounded-xl cn:bg-card cn:text-[15px] cn:shadow-soft cn:border-border/80 cn:placeholder:text-muted-foreground/70",
          "cn:focus-visible:border-ring cn:focus-visible:ring-2 cn:focus-visible:ring-ring/10",
          value && "cn:pr-[104px]"
        )}
      />

      {/* Right adornments */}
      <div className="cn:absolute cn:right-1.5 cn:top-1/2 cn:-translate-y-1/2 cn:flex cn:items-center cn:gap-1">
        {value ? (
          <button
            type="button"
            onClick={() => onValueChange("")}
            aria-label="Clear search"
            className="cn:inline-flex cn:size-7 cn:items-center cn:justify-center cn:rounded-full cn:text-muted-foreground cn:hover:bg-muted cn:hover:text-foreground cn:transition-colors"
          >
            <X className="cn:size-3.5" />
          </button>
        ) : (
          <span className="cn:hidden cn:sm:inline-flex cn:items-center cn:gap-1 cn:rounded-md cn:border cn:bg-muted/60 cn:px-1.5 cn:py-1 cn:text-[11px] cn:text-muted-foreground">
            <Kbd className="cn:bg-background">/</Kbd>
          </span>
        )}
        <span className="cn:hidden cn:md:inline-flex cn:items-center cn:gap-1 cn:rounded-md cn:border cn:bg-muted/60 cn:px-1.5 cn:py-1 cn:text-[11px] cn:text-muted-foreground cn:ml-1">
          <Command className="cn:size-3" />K
        </span>
      </div>
    </div>
  )
}
