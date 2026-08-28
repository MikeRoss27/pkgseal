import * as React from "react"
import { Menu as MenuPrimitive } from "@base-ui/react/menu"

import { cn } from "@/lib/cn"
import { ChevronRightIcon, CheckIcon } from "lucide-react"

function DropdownMenu({ ...props }: MenuPrimitive.Root.Props) {
  return <MenuPrimitive.Root data-slot="dropdown-menu" {...props} />
}

function DropdownMenuPortal({ ...props }: MenuPrimitive.Portal.Props) {
  return <MenuPrimitive.Portal data-slot="dropdown-menu-portal" {...props} />
}

function DropdownMenuTrigger({ ...props }: MenuPrimitive.Trigger.Props) {
  return <MenuPrimitive.Trigger data-slot="dropdown-menu-trigger" {...props} />
}

function DropdownMenuContent({
  align = "start",
  alignOffset = 0,
  side = "bottom",
  sideOffset = 4,
  className,
  ...props
}: MenuPrimitive.Popup.Props &
  Pick<
    MenuPrimitive.Positioner.Props,
    "align" | "alignOffset" | "side" | "sideOffset"
  >) {
  return (
    <MenuPrimitive.Portal>
      <MenuPrimitive.Positioner
        className="cn:isolate cn:z-50 cn:outline-none"
        align={align}
        alignOffset={alignOffset}
        side={side}
        sideOffset={sideOffset}
      >
        <MenuPrimitive.Popup
          data-slot="dropdown-menu-content"
          className={cn("cn: cn: cn:z-50 cn:max-h-(--available-height) cn:w-(--anchor-width) cn:min-w-32 cn:origin-(--transform-origin) cn:overflow-x-hidden cn:overflow-y-auto cn:rounded-lg cn:bg-popover cn:p-1 cn:text-popover-foreground cn:shadow-md cn:ring-1 cn:ring-foreground/10 cn:duration-100 cn:outline-none cn:data-[side=bottom]:slide-in-from-top-2 cn:data-[side=inline-end]:slide-in-from-left-2 cn:data-[side=inline-start]:slide-in-from-right-2 cn:data-[side=left]:slide-in-from-right-2 cn:data-[side=right]:slide-in-from-left-2 cn:data-[side=top]:slide-in-from-bottom-2 cn:data-open:animate-in cn:data-open:fade-in-0 cn:data-open:zoom-in-95 cn:data-closed:animate-out cn:data-closed:overflow-hidden cn:data-closed:fade-out-0 cn:data-closed:zoom-out-95", className )}
          {...props}
        />
      </MenuPrimitive.Positioner>
    </MenuPrimitive.Portal>
  )
}

function DropdownMenuGroup({ ...props }: MenuPrimitive.Group.Props) {
  return <MenuPrimitive.Group data-slot="dropdown-menu-group" {...props} />
}

function DropdownMenuLabel({
  className,
  inset,
  ...props
}: MenuPrimitive.GroupLabel.Props & {
  inset?: boolean
}) {
  return (
    <MenuPrimitive.GroupLabel
      data-slot="dropdown-menu-label"
      data-inset={inset}
      className={cn(
        "cn:px-1.5 cn:py-1 cn:text-xs cn:font-medium cn:text-muted-foreground cn:data-inset:pl-7",
        className
      )}
      {...props}
    />
  )
}

function DropdownMenuItem({
  className,
  inset,
  variant = "default",
  ...props
}: MenuPrimitive.Item.Props & {
  inset?: boolean
  variant?: "default" | "destructive"
}) {
  return (
    <MenuPrimitive.Item
      data-slot="dropdown-menu-item"
      data-inset={inset}
      data-variant={variant}
      className={cn(
        "cn:group/dropdown-menu-item cn:relative cn:flex cn:cursor-default cn:items-center cn:gap-1.5 cn:rounded-md cn:px-1.5 cn:py-1 cn:text-sm cn:outline-hidden cn:select-none cn:focus:bg-accent cn:focus:text-accent-foreground cn:not-data-[variant=destructive]:focus:**:text-accent-foreground cn:data-inset:pl-7 cn:data-[variant=destructive]:text-destructive cn:data-[variant=destructive]:focus:bg-destructive/10 cn:data-[variant=destructive]:focus:text-destructive cn:dark:data-[variant=destructive]:focus:bg-destructive/20 cn:data-disabled:pointer-events-none cn:data-disabled:opacity-50 cn:[&_svg]:pointer-events-none cn:[&_svg]:shrink-0 cn:[&_svg:not([class*=size-])]:size-4 cn:data-[variant=destructive]:*:[svg]:text-destructive",
        className
      )}
      {...props}
    />
  )
}

function DropdownMenuSub({ ...props }: MenuPrimitive.SubmenuRoot.Props) {
  return <MenuPrimitive.SubmenuRoot data-slot="dropdown-menu-sub" {...props} />
}

function DropdownMenuSubTrigger({
  className,
  inset,
  children,
  ...props
}: MenuPrimitive.SubmenuTrigger.Props & {
  inset?: boolean
}) {
  return (
    <MenuPrimitive.SubmenuTrigger
      data-slot="dropdown-menu-sub-trigger"
      data-inset={inset}
      className={cn(
        "cn:flex cn:cursor-default cn:items-center cn:gap-1.5 cn:rounded-md cn:px-1.5 cn:py-1 cn:text-sm cn:outline-hidden cn:select-none cn:focus:bg-accent cn:focus:text-accent-foreground cn:not-data-[variant=destructive]:focus:**:text-accent-foreground cn:data-inset:pl-7 cn:data-popup-open:bg-accent cn:data-popup-open:text-accent-foreground cn:data-open:bg-accent cn:data-open:text-accent-foreground cn:[&_svg]:pointer-events-none cn:[&_svg]:shrink-0 cn:[&_svg:not([class*=size-])]:size-4",
        className
      )}
      {...props}
    >
      {children}
      <ChevronRightIcon className="cn:ml-auto" />
    </MenuPrimitive.SubmenuTrigger>
  )
}

function DropdownMenuSubContent({
  align = "start",
  alignOffset = -3,
  side = "right",
  sideOffset = 0,
  className,
  ...props
}: React.ComponentProps<typeof DropdownMenuContent>) {
  return (
    <DropdownMenuContent
      data-slot="dropdown-menu-sub-content"
      className={cn("cn: cn: cn:w-auto cn:min-w-[96px] cn:rounded-lg cn:bg-popover cn:p-1 cn:text-popover-foreground cn:shadow-lg cn:ring-1 cn:ring-foreground/10 cn:duration-100 cn:data-[side=bottom]:slide-in-from-top-2 cn:data-[side=left]:slide-in-from-right-2 cn:data-[side=right]:slide-in-from-left-2 cn:data-[side=top]:slide-in-from-bottom-2 cn:data-open:animate-in cn:data-open:fade-in-0 cn:data-open:zoom-in-95 cn:data-closed:animate-out cn:data-closed:fade-out-0 cn:data-closed:zoom-out-95", className )}
      align={align}
      alignOffset={alignOffset}
      side={side}
      sideOffset={sideOffset}
      {...props}
    />
  )
}

function DropdownMenuCheckboxItem({
  className,
  children,
  checked,
  inset,
  ...props
}: MenuPrimitive.CheckboxItem.Props & {
  inset?: boolean
}) {
  return (
    <MenuPrimitive.CheckboxItem
      data-slot="dropdown-menu-checkbox-item"
      data-inset={inset}
      className={cn(
        "cn:relative cn:flex cn:cursor-default cn:items-center cn:gap-1.5 cn:rounded-md cn:py-1 cn:pr-8 cn:pl-1.5 cn:text-sm cn:outline-hidden cn:select-none cn:focus:bg-accent cn:focus:text-accent-foreground cn:focus:**:text-accent-foreground cn:data-inset:pl-7 cn:data-disabled:pointer-events-none cn:data-disabled:opacity-50 cn:[&_svg]:pointer-events-none cn:[&_svg]:shrink-0 cn:[&_svg:not([class*=size-])]:size-4",
        className
      )}
      checked={checked}
      {...props}
    >
      <span
        className="cn:pointer-events-none cn:absolute cn:right-2 cn:flex cn:items-center cn:justify-center"
        data-slot="dropdown-menu-checkbox-item-indicator"
      >
        <MenuPrimitive.CheckboxItemIndicator>
          <CheckIcon
          />
        </MenuPrimitive.CheckboxItemIndicator>
      </span>
      {children}
    </MenuPrimitive.CheckboxItem>
  )
}

function DropdownMenuRadioGroup({ ...props }: MenuPrimitive.RadioGroup.Props) {
  return (
    <MenuPrimitive.RadioGroup
      data-slot="dropdown-menu-radio-group"
      {...props}
    />
  )
}

function DropdownMenuRadioItem({
  className,
  children,
  inset,
  ...props
}: MenuPrimitive.RadioItem.Props & {
  inset?: boolean
}) {
  return (
    <MenuPrimitive.RadioItem
      data-slot="dropdown-menu-radio-item"
      data-inset={inset}
      className={cn(
        "cn:relative cn:flex cn:cursor-default cn:items-center cn:gap-1.5 cn:rounded-md cn:py-1 cn:pr-8 cn:pl-1.5 cn:text-sm cn:outline-hidden cn:select-none cn:focus:bg-accent cn:focus:text-accent-foreground cn:focus:**:text-accent-foreground cn:data-inset:pl-7 cn:data-disabled:pointer-events-none cn:data-disabled:opacity-50 cn:[&_svg]:pointer-events-none cn:[&_svg]:shrink-0 cn:[&_svg:not([class*=size-])]:size-4",
        className
      )}
      {...props}
    >
      <span
        className="cn:pointer-events-none cn:absolute cn:right-2 cn:flex cn:items-center cn:justify-center"
        data-slot="dropdown-menu-radio-item-indicator"
      >
        <MenuPrimitive.RadioItemIndicator>
          <CheckIcon
          />
        </MenuPrimitive.RadioItemIndicator>
      </span>
      {children}
    </MenuPrimitive.RadioItem>
  )
}

function DropdownMenuSeparator({
  className,
  ...props
}: MenuPrimitive.Separator.Props) {
  return (
    <MenuPrimitive.Separator
      data-slot="dropdown-menu-separator"
      className={cn("cn:-mx-1 cn:my-1 cn:h-px cn:bg-border", className)}
      {...props}
    />
  )
}

function DropdownMenuShortcut({
  className,
  ...props
}: React.ComponentProps<"span">) {
  return (
    <span
      data-slot="dropdown-menu-shortcut"
      className={cn(
        "cn:ml-auto cn:text-xs cn:tracking-widest cn:text-muted-foreground cn:group-focus/dropdown-menu-item:text-accent-foreground",
        className
      )}
      {...props}
    />
  )
}

export {
  DropdownMenu,
  DropdownMenuPortal,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuItem,
  DropdownMenuCheckboxItem,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
}
