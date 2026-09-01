import { cn } from "@/lib/cn"

function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      className={cn("cn:animate-pulse cn:rounded-md cn:bg-muted", className)}
      {...props}
    />
  )
}

export { Skeleton }
