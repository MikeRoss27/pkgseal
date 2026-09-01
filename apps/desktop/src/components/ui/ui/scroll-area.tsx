"use client"

import { ScrollArea as ScrollAreaPrimitive } from "@base-ui/react/scroll-area"

import { cn } from "@/lib/cn"

function ScrollArea({
  className,
  children,
  ...props
}: ScrollAreaPrimitive.Root.Props) {
  return (
    <ScrollAreaPrimitive.Root
      data-slot="scroll-area"
      className={cn("cn:relative", className)}
      {...props}
    >
      <ScrollAreaPrimitive.Viewport
        data-slot="scroll-area-viewport"
        className="cn:size-full cn:rounded-[inherit] cn:transition-[color,box-shadow] cn:outline-none cn:focus-visible:ring-[3px] cn:focus-visible:ring-ring/50 cn:focus-visible:outline-1"
      >
        {children}
      </ScrollAreaPrimitive.Viewport>
      <ScrollBar />
      <ScrollAreaPrimitive.Corner />
    </ScrollAreaPrimitive.Root>
  )
}

function ScrollBar({
  className,
  orientation = "vertical",
  ...props
}: ScrollAreaPrimitive.Scrollbar.Props) {
  return (
    <ScrollAreaPrimitive.Scrollbar
      data-slot="scroll-area-scrollbar"
      data-orientation={orientation}
      orientation={orientation}
      className={cn(
        "cn:flex cn:touch-none cn:p-px cn:transition-colors cn:select-none cn:data-[orientation=horizontal]:h-2.5 cn:data-[orientation=horizontal]:flex-col cn:data-[orientation=horizontal]:border-t cn:data-[orientation=horizontal]:border-t-transparent cn:data-[orientation=vertical]:h-full cn:data-[orientation=vertical]:w-2.5 cn:data-[orientation=vertical]:border-l cn:data-[orientation=vertical]:border-l-transparent",
        className
      )}
      {...props}
    >
      <ScrollAreaPrimitive.Thumb
        data-slot="scroll-area-thumb"
        className="cn:relative cn:flex-1 cn:rounded-full cn:bg-border"
      />
    </ScrollAreaPrimitive.Scrollbar>
  )
}

export { ScrollArea, ScrollBar }
