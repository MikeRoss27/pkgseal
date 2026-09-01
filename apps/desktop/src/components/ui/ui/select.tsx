"use client"

import * as React from "react"
import { Select as SelectPrimitive } from "@base-ui/react/select"
import { ChevronDownIcon, CheckIcon, ChevronUpIcon } from "lucide-react"

import { cn } from "@/lib/cn"

function Select({ ...props }: SelectPrimitive.Root.Props<string, false>) {
  return <SelectPrimitive.Root data-slot="select" {...props} />
}

function SelectGroup({ ...props }: SelectPrimitive.Group.Props) {
  return <SelectPrimitive.Group data-slot="select-group" {...props} />
}

function SelectValue({ ...props }: SelectPrimitive.Value.Props) {
  return <SelectPrimitive.Value data-slot="select-value" {...props} />
}

function SelectTrigger({
  className,
  children,
  ...props
}: SelectPrimitive.Trigger.Props) {
  return (
    <SelectPrimitive.Trigger
      data-slot="select-trigger"
      className={cn(
        "cn:flex cn:h-8 cn:w-full cn:items-center cn:justify-between cn:gap-1.5 cn:rounded-lg cn:border cn:border-input cn:bg-background cn:px-2.5 cn:py-0 cn:text-sm cn:whitespace-nowrap cn:shadow-xs cn:outline-none cn:transition-all cn:placeholder:text-muted-foreground cn:focus-visible:border-ring cn:focus-visible:ring-[3px] cn:focus-visible:ring-ring/50 cn:disabled:cursor-not-allowed cn:disabled:opacity-50 cn:dark:bg-input/30 cn:[&>svg]:pointer-events-none cn:[&>svg]:shrink-0 cn:[&_svg:not([class*=size-])]:size-4",
        "cn:data-placeholder:text-muted-foreground",
        className
      )}
      {...props}
    >
      {children}
      <SelectPrimitive.Icon render={<ChevronDownIcon className="cn:size-4 cn:opacity-50" />} />
    </SelectPrimitive.Trigger>
  )
}

function SelectContent({
  className,
  children,
  side = "bottom",
  sideOffset = 4,
  align = "start",
  alignOffset = 0,
  ...props
}: SelectPrimitive.Popup.Props &
  Pick<SelectPrimitive.Positioner.Props, "align" | "alignOffset" | "side" | "sideOffset">) {
  return (
    <SelectPrimitive.Portal>
      <SelectPrimitive.Backdrop />
      <SelectPrimitive.Positioner
        className="cn:isolate cn:z-50 cn:outline-none"
        side={side}
        sideOffset={sideOffset}
        align={align}
        alignOffset={alignOffset}
      >
        <SelectPrimitive.Popup
          data-slot="select-content"
          className={cn(
            "cn:relative cn:z-50 cn:max-h-[--available-height] cn:min-w-[8rem] cn:origin-[--transform-origin] cn:overflow-x-hidden cn:overflow-y-auto cn:rounded-lg cn:border cn:bg-popover cn:p-1 cn:text-popover-foreground cn:shadow-md cn:ring-1 cn:ring-foreground/10 cn:data-open:animate-in cn:data-open:fade-in-0 cn:data-open:zoom-in-95 cn:data-closed:animate-out cn:data-closed:fade-out-0 cn:data-closed:zoom-out-95 cn:data-[side=bottom]:slide-in-from-top-2 cn:data-[side=top]:slide-in-from-bottom-2 cn:data-[side=left]:slide-in-from-right-2 cn:data-[side=right]:slide-in-from-left-2",
            className
          )}
          {...props}
        >
          <SelectPrimitive.List className="cn:flex cn:flex-col cn:gap-0.5">
            {children}
          </SelectPrimitive.List>
        </SelectPrimitive.Popup>
      </SelectPrimitive.Positioner>
    </SelectPrimitive.Portal>
  )
}

function SelectLabel({
  className,
  ...props
}: SelectPrimitive.GroupLabel.Props) {
  return (
    <SelectPrimitive.GroupLabel
      data-slot="select-label"
      className={cn("cn:px-1.5 cn:py-1 cn:text-xs cn:font-medium cn:text-muted-foreground", className)}
      {...props}
    />
  )
}

function SelectItem({
  className,
  children,
  ...props
}: SelectPrimitive.Item.Props) {
  return (
    <SelectPrimitive.Item
      data-slot="select-item"
      className={cn(
        "cn:relative cn:flex cn:w-full cn:cursor-default cn:items-center cn:gap-1.5 cn:rounded-md cn:px-1.5 cn:py-1 cn:pr-6 cn:text-sm cn:outline-none cn:select-none cn:focus:bg-accent cn:focus:text-accent-foreground cn:data-highlighted:bg-accent cn:data-highlighted:text-accent-foreground cn:data-disabled:pointer-events-none cn:data-disabled:opacity-50 cn:[&_svg]:pointer-events-none cn:[&_svg]:shrink-0 cn:[&_svg:not([class*=size-])]:size-3.5",
        className
      )}
      {...props}
    >
      <span className="cn:absolute cn:right-1.5 cn:flex cn:size-3.5 cn:items-center cn:justify-center">
        <SelectPrimitive.ItemIndicator>
          <CheckIcon className="cn:size-3.5" />
        </SelectPrimitive.ItemIndicator>
      </span>
      <SelectPrimitive.ItemText className="cn:flex cn:flex-1 cn:min-w-0 cn:flex-col cn:gap-0.5">
        {children}
      </SelectPrimitive.ItemText>
    </SelectPrimitive.Item>
  )
}

function SelectSeparator({
  className,
  ...props
}: SelectPrimitive.Separator.Props) {
  return (
    <SelectPrimitive.Separator
      data-slot="select-separator"
      className={cn("cn:-mx-1 cn:my-1 cn:h-px cn:bg-border", className)}
      {...props}
    />
  )
}

function SelectScrollUpButton({
  className,
  ...props
}: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="select-scroll-up-button"
      className={cn("cn:flex cn:cursor-default cn:items-center cn:justify-center cn:py-1", className)}
      {...props}
    >
      <ChevronUpIcon className="cn:size-4" />
    </div>
  )
}

function SelectScrollDownButton({
  className,
  ...props
}: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="select-scroll-down-button"
      className={cn("cn:flex cn:cursor-default cn:items-center cn:justify-center cn:py-1", className)}
      {...props}
    >
      <ChevronDownIcon className="cn:size-4" />
    </div>
  )
}

export {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectScrollDownButton,
  SelectScrollUpButton,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
}
