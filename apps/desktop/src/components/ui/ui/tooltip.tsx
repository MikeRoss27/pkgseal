import { Tooltip as TooltipPrimitive } from "@base-ui/react/tooltip"

import { cn } from "@/lib/cn"

function TooltipProvider({
  delay = 0,
  ...props
}: TooltipPrimitive.Provider.Props) {
  return (
    <TooltipPrimitive.Provider
      data-slot="tooltip-provider"
      delay={delay}
      {...props}
    />
  )
}

function Tooltip({ ...props }: TooltipPrimitive.Root.Props) {
  return <TooltipPrimitive.Root data-slot="tooltip" {...props} />
}

function TooltipTrigger({ ...props }: TooltipPrimitive.Trigger.Props) {
  return <TooltipPrimitive.Trigger data-slot="tooltip-trigger" {...props} />
}

function TooltipContent({
  className,
  side = "top",
  sideOffset = 4,
  align = "center",
  alignOffset = 0,
  children,
  ...props
}: TooltipPrimitive.Popup.Props &
  Pick<
    TooltipPrimitive.Positioner.Props,
    "align" | "alignOffset" | "side" | "sideOffset"
  >) {
  return (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Positioner
        align={align}
        alignOffset={alignOffset}
        side={side}
        sideOffset={sideOffset}
        className="cn:isolate cn:z-50"
      >
        <TooltipPrimitive.Popup
          data-slot="tooltip-content"
          className={cn(
            "cn:z-50 cn:inline-flex cn:w-fit cn:max-w-xs cn:origin-(--transform-origin) cn:items-center cn:gap-1.5 cn:rounded-md cn:bg-foreground cn:px-3 cn:py-1.5 cn:text-xs cn:text-background cn:has-data-[slot=kbd]:pr-1.5 cn:data-[side=bottom]:slide-in-from-top-2 cn:data-[side=inline-end]:slide-in-from-left-2 cn:data-[side=inline-start]:slide-in-from-right-2 cn:data-[side=left]:slide-in-from-right-2 cn:data-[side=right]:slide-in-from-left-2 cn:data-[side=top]:slide-in-from-bottom-2 cn:**:data-[slot=kbd]:relative cn:**:data-[slot=kbd]:isolate cn:**:data-[slot=kbd]:z-50 cn:**:data-[slot=kbd]:rounded-sm cn:data-[state=delayed-open]:animate-in cn:data-[state=delayed-open]:fade-in-0 cn:data-[state=delayed-open]:zoom-in-95 cn:data-open:animate-in cn:data-open:fade-in-0 cn:data-open:zoom-in-95 cn:data-closed:animate-out cn:data-closed:fade-out-0 cn:data-closed:zoom-out-95",
            className
          )}
          {...props}
        >
          {children}
          <TooltipPrimitive.Arrow className="cn:z-50 cn:size-2.5 cn:translate-y-[calc(-50%-2px)] cn:rotate-45 cn:rounded-[2px] cn:bg-foreground cn:fill-foreground cn:data-[side=bottom]:top-1 cn:data-[side=inline-end]:top-1/2! cn:data-[side=inline-end]:-left-1 cn:data-[side=inline-end]:-translate-y-1/2 cn:data-[side=inline-start]:top-1/2! cn:data-[side=inline-start]:-right-1 cn:data-[side=inline-start]:-translate-y-1/2 cn:data-[side=left]:top-1/2! cn:data-[side=left]:-right-1 cn:data-[side=left]:-translate-y-1/2 cn:data-[side=right]:top-1/2! cn:data-[side=right]:-left-1 cn:data-[side=right]:-translate-y-1/2 cn:data-[side=top]:-bottom-2.5" />
        </TooltipPrimitive.Popup>
      </TooltipPrimitive.Positioner>
    </TooltipPrimitive.Portal>
  )
}

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider }
