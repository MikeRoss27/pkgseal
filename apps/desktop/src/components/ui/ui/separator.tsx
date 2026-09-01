import { Separator as SeparatorPrimitive } from "@base-ui/react/separator"

import { cn } from "@/lib/cn"

function Separator({
  className,
  orientation = "horizontal",
  ...props
}: SeparatorPrimitive.Props) {
  return (
    <SeparatorPrimitive
      data-slot="separator"
      orientation={orientation}
      className={cn(
        "cn:shrink-0 cn:bg-border cn:data-[orientation=horizontal]:h-px cn:data-[orientation=horizontal]:w-full cn:data-[orientation=vertical]:w-px cn:data-[orientation=vertical]:self-stretch",
        className
      )}
      {...props}
    />
  )
}

export { Separator }
