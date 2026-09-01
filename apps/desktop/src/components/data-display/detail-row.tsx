import type { ReactNode } from "react"
import { Separator } from "@/components/ui/ui/separator"

interface DetailRowProps {
  label: string
  value?: ReactNode
  hint?: string
  mono?: boolean
}

export function DetailRow({ label, value, hint, mono }: DetailRowProps) {
  const content = value ?? "—"
  const isEmpty = value === null || value === undefined || value === "" || value === "—"
  return (
    <div className="cn:flex cn:items-start cn:justify-between cn:gap-4 cn:py-2.5 cn:text-sm">
      <span className="cn:shrink-0 cn:text-xs cn:font-medium cn:uppercase cn:tracking-wide cn:text-muted-foreground cn:pt-0.5">
        {label}
      </span>
      <span
        className={`cn:text-right cn:max-w-[60%] cn:break-words ${mono ? "cn:font-mono cn:text-xs" : "cn:text-foreground"} ${isEmpty ? "cn:text-muted-foreground" : ""}`}
        title={hint}
      >
        {content}
      </span>
    </div>
  )
}

export function DetailRows({ children }: { children: ReactNode }) {
  return (
    <div className="cn:divide-y cn:divide-border/60">
      {children}
    </div>
  )
}

export function DetailSection({ title, children, action }: { title: string; children: ReactNode; action?: ReactNode }) {
  return (
    <section className="cn:space-y-2">
      <div className="cn:flex cn:items-center cn:justify-between">
        <h4 className="cn:text-xs cn:font-semibold cn:uppercase cn:tracking-widest cn:text-muted-foreground">
          {title}
        </h4>
        {action}
      </div>
      <div className="cn:rounded-xl cn:border cn:border-border/80 cn:bg-card cn:px-3 cn:py-1">
        {children}
      </div>
    </section>
  )
}

export { Separator }
