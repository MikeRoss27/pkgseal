import { cn } from "@/lib/cn"

function Kbd({ className, ...props }: React.ComponentProps<"kbd">) {
  return (
    <kbd
      data-slot="kbd"
      className={cn(
        "cn:pointer-events-none cn:inline-flex cn:h-5 cn:w-fit cn:min-w-5 cn:items-center cn:justify-center cn:gap-1 cn:rounded-sm cn:bg-muted cn:px-1 cn:font-sans cn:text-xs cn:font-medium cn:text-muted-foreground cn:select-none cn:in-data-[slot=tooltip-content]:bg-background/20 cn:in-data-[slot=tooltip-content]:text-background cn:dark:in-data-[slot=tooltip-content]:bg-background/10 cn:[&_svg:not([class*=size-])]:size-3",
        className
      )}
      {...props}
    />
  )
}

function KbdGroup({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <kbd
      data-slot="kbd-group"
      className={cn("cn:inline-flex cn:items-center cn:gap-1", className)}
      {...props}
    />
  )
}

export { Kbd, KbdGroup }
