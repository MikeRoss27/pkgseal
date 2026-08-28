import * as React from "react"

import { cn } from "@/lib/cn"

function Card({
  className,
  size = "default",
  ...props
}: React.ComponentProps<"div"> & { size?: "default" | "sm" }) {
  return (
    <div
      data-slot="card"
      data-size={size}
      className={cn(
        "cn:group/card cn:flex cn:flex-col cn:gap-(--card-spacing) cn:overflow-hidden cn:rounded-xl cn:bg-card cn:py-(--card-spacing) cn:text-sm cn:text-card-foreground cn:ring-1 cn:ring-foreground/10 cn:[--card-spacing:--spacing(4)] cn:has-data-[slot=card-footer]:pb-0 cn:has-[>img:first-child]:pt-0 cn:data-[size=sm]:[--card-spacing:--spacing(3)] cn:data-[size=sm]:has-data-[slot=card-footer]:pb-0 cn:*:[img:first-child]:rounded-t-xl cn:*:[img:last-child]:rounded-b-xl",
        className
      )}
      {...props}
    />
  )
}

function CardHeader({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-header"
      className={cn(
        "cn:group/card-header cn:@container/card-header cn:grid cn:auto-rows-min cn:items-start cn:gap-1 cn:rounded-t-xl cn:px-(--card-spacing) cn:has-data-[slot=card-action]:grid-cols-[1fr_auto] cn:has-data-[slot=card-description]:grid-rows-[auto_auto] cn:[.border-b]:pb-(--card-spacing)",
        className
      )}
      {...props}
    />
  )
}

function CardTitle({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-title"
      className={cn(
        "cn: cn:text-base cn:leading-snug cn:font-medium cn:group-data-[size=sm]/card:text-sm",
        className
      )}
      {...props}
    />
  )
}

function CardDescription({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-description"
      className={cn("cn:text-sm cn:text-muted-foreground", className)}
      {...props}
    />
  )
}

function CardAction({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-action"
      className={cn(
        "cn:col-start-2 cn:row-span-2 cn:row-start-1 cn:self-start cn:justify-self-end",
        className
      )}
      {...props}
    />
  )
}

function CardContent({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-content"
      className={cn("cn:px-(--card-spacing)", className)}
      {...props}
    />
  )
}

function CardFooter({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-footer"
      className={cn(
        "cn:flex cn:items-center cn:rounded-b-xl cn:border-t cn:bg-muted/50 cn:p-(--card-spacing)",
        className
      )}
      {...props}
    />
  )
}

export {
  Card,
  CardHeader,
  CardFooter,
  CardTitle,
  CardAction,
  CardDescription,
  CardContent,
}
