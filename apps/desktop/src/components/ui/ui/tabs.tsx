"use client"

import { Tabs as TabsPrimitive } from "@base-ui/react/tabs"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/cn"

function Tabs({
  className,
  orientation = "horizontal",
  ...props
}: TabsPrimitive.Root.Props) {
  return (
    <TabsPrimitive.Root
      data-slot="tabs"
      data-orientation={orientation}
      className={cn(
        "cn:group/tabs cn:flex cn:gap-2 cn:data-horizontal:flex-col",
        className
      )}
      {...props}
    />
  )
}

const tabsListVariants = cva(
  "cn:group/tabs-list cn:inline-flex cn:w-fit cn:items-center cn:justify-center cn:rounded-lg cn:p-[3px] cn:text-muted-foreground cn:group-data-horizontal/tabs:h-8 cn:group-data-vertical/tabs:h-fit cn:group-data-vertical/tabs:flex-col cn:data-[variant=line]:rounded-none",
  {
    variants: {
      variant: {
        default: "cn:bg-muted",
        line: "cn:gap-1 cn:bg-transparent",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

function TabsList({
  className,
  variant = "default",
  ...props
}: TabsPrimitive.List.Props & VariantProps<typeof tabsListVariants>) {
  return (
    <TabsPrimitive.List
      data-slot="tabs-list"
      data-variant={variant}
      className={cn(tabsListVariants({ variant }), className)}
      {...props}
    />
  )
}

function TabsTrigger({ className, ...props }: TabsPrimitive.Tab.Props) {
  return (
    <TabsPrimitive.Tab
      data-slot="tabs-trigger"
      className={cn(
        "cn:relative cn:inline-flex cn:h-[calc(100%-1px)] cn:flex-1 cn:items-center cn:justify-center cn:gap-1.5 cn:rounded-md cn:border cn:border-transparent cn:px-1.5 cn:py-0.5 cn:text-sm cn:font-medium cn:whitespace-nowrap cn:text-foreground/60 cn:transition-all cn:group-data-vertical/tabs:w-full cn:group-data-vertical/tabs:justify-start cn:hover:text-foreground cn:focus-visible:border-ring cn:focus-visible:ring-[3px] cn:focus-visible:ring-ring/50 cn:focus-visible:outline-1 cn:focus-visible:outline-ring cn:disabled:pointer-events-none cn:disabled:opacity-50 cn:has-data-[icon=inline-end]:pr-1 cn:has-data-[icon=inline-start]:pl-1 cn:aria-disabled:pointer-events-none cn:aria-disabled:opacity-50 cn:dark:text-muted-foreground cn:dark:hover:text-foreground cn:group-data-[variant=default]/tabs-list:data-active:shadow-sm cn:group-data-[variant=line]/tabs-list:data-active:shadow-none cn:[&_svg]:pointer-events-none cn:[&_svg]:shrink-0 cn:[&_svg:not([class*=size-])]:size-4",
        "cn:group-data-[variant=line]/tabs-list:bg-transparent cn:group-data-[variant=line]/tabs-list:data-active:bg-transparent cn:dark:group-data-[variant=line]/tabs-list:data-active:border-transparent cn:dark:group-data-[variant=line]/tabs-list:data-active:bg-transparent",
        "cn:data-active:bg-background cn:data-active:text-foreground cn:dark:data-active:border-input cn:dark:data-active:bg-input/30 cn:dark:data-active:text-foreground",
        "cn:after:absolute cn:after:bg-foreground cn:after:opacity-0 cn:after:transition-opacity cn:group-data-horizontal/tabs:after:inset-x-0 cn:group-data-horizontal/tabs:after:bottom-[-5px] cn:group-data-horizontal/tabs:after:h-0.5 cn:group-data-vertical/tabs:after:inset-y-0 cn:group-data-vertical/tabs:after:-right-1 cn:group-data-vertical/tabs:after:w-0.5 cn:group-data-[variant=line]/tabs-list:data-active:after:opacity-100",
        className
      )}
      {...props}
    />
  )
}

function TabsContent({ className, ...props }: TabsPrimitive.Panel.Props) {
  return (
    <TabsPrimitive.Panel
      data-slot="tabs-content"
      className={cn("cn:flex-1 cn:text-sm cn:outline-none", className)}
      {...props}
    />
  )
}

export { Tabs, TabsList, TabsTrigger, TabsContent, tabsListVariants }
