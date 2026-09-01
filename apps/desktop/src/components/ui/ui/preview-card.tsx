import { PreviewCard as PreviewCardPrimitive } from "@base-ui/react/preview-card"

import { cn } from "@/lib/cn"

function PreviewCard({ ...props }: PreviewCardPrimitive.Root.Props) {
  return <PreviewCardPrimitive.Root data-slot="preview-card" {...props} />
}

function PreviewCardTrigger({ ...props }: PreviewCardPrimitive.Trigger.Props) {
  return <PreviewCardPrimitive.Trigger data-slot="preview-card-trigger" {...props} />
}

function PreviewCardContent({
  className,
  side = "bottom",
  sideOffset = 8,
  align = "center",
  alignOffset = 0,
  children,
  ...props
}: PreviewCardPrimitive.Popup.Props &
  Pick<PreviewCardPrimitive.Positioner.Props, "align" | "alignOffset" | "side" | "sideOffset">) {
  return (
    <PreviewCardPrimitive.Portal>
      <PreviewCardPrimitive.Positioner align={align} alignOffset={alignOffset} side={side} sideOffset={sideOffset} className="cn:isolate cn:z-50">
        <PreviewCardPrimitive.Popup
          data-slot="preview-card-content"
          className={cn(
            "cn:z-50 cn:w-72 cn:origin-(--transform-origin) cn:rounded-lg cn:border cn:border-border cn:bg-popover cn:p-3 cn:text-popover-foreground cn:shadow-md cn:outline-none cn:data-open:animate-in cn:data-open:fade-in-0 cn:data-open:zoom-in-95 cn:data-closed:animate-out cn:data-closed:fade-out-0 cn:data-closed:zoom-out-95",
            className
          )}
          {...props}
        >
          {children}
        </PreviewCardPrimitive.Popup>
      </PreviewCardPrimitive.Positioner>
    </PreviewCardPrimitive.Portal>
  )
}

export { PreviewCard, PreviewCardTrigger, PreviewCardContent }
