import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/cn"

const alertVariants = cva(
  "cn:group/alert cn:relative cn:grid cn:w-full cn:gap-0.5 cn:rounded-lg cn:border cn:px-2.5 cn:py-2 cn:text-left cn:text-sm cn:has-data-[slot=alert-action]:relative cn:has-data-[slot=alert-action]:pr-18 cn:has-[>svg]:grid-cols-[auto_1fr] cn:has-[>svg]:gap-x-2 cn:*:[svg]:row-span-2 cn:*:[svg]:translate-y-0.5 cn:*:[svg]:text-current cn:*:[svg:not([class*=size-])]:size-4",
  {
    variants: {
      variant: {
        default: "cn:bg-card cn:text-card-foreground",
        destructive:
          "cn:bg-card cn:text-destructive cn:*:data-[slot=alert-description]:text-destructive/90 cn:*:[svg]:text-current",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

function Alert({
  className,
  variant,
  ...props
}: React.ComponentProps<"div"> & VariantProps<typeof alertVariants>) {
  return (
    <div
      data-slot="alert"
      role="alert"
      className={cn(alertVariants({ variant }), className)}
      {...props}
    />
  )
}

function AlertTitle({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="alert-title"
      className={cn(
        "cn:font-medium cn:group-has-[>svg]/alert:col-start-2 cn:[&_a]:underline cn:[&_a]:underline-offset-3 cn:[&_a]:hover:text-foreground",
        className
      )}
      {...props}
    />
  )
}

function AlertDescription({
  className,
  ...props
}: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="alert-description"
      className={cn(
        "cn:text-sm cn:text-balance cn:text-muted-foreground cn:md:text-pretty cn:[&_a]:underline cn:[&_a]:underline-offset-3 cn:[&_a]:hover:text-foreground cn:[&_p:not(:last-child)]:mb-4",
        className
      )}
      {...props}
    />
  )
}

function AlertAction({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="alert-action"
      className={cn("cn:absolute cn:top-2 cn:right-2", className)}
      {...props}
    />
  )
}

export { Alert, AlertTitle, AlertDescription, AlertAction }
