import type { ReactNode } from "react"
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/ui/empty"
import { Button } from "@/components/ui/ui/button"

interface EmptyStateProps {
  icon?: ReactNode
  title: string
  description?: ReactNode
  action?: { label: string; onClick: () => void }
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <Empty className="cn:border cn:border-dashed cn:bg-muted/20 cn:py-10">
      {icon && <EmptyMedia variant="icon">{icon}</EmptyMedia>}
      <EmptyHeader>
        <EmptyTitle>{title}</EmptyTitle>
        {description && <EmptyDescription>{description}</EmptyDescription>}
      </EmptyHeader>
      {action && (
        <Button variant="outline" size="sm" onClick={action.onClick}>
          {action.label}
        </Button>
      )}
    </Empty>
  )
}
