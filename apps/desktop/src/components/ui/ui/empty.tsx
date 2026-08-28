import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/cn"

function Empty({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="empty"
      className={cn(
        "cn:flex cn:w-full cn:min-w-0 cn:flex-1 cn:flex-col cn:items-center cn:justify-center cn:gap-4 cn:rounded-xl cn:border-dashed cn:p-6 cn:text-center cn:text-balance",
        className
      )}
      {...props}
    />
  )
}

function EmptyHeader({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="empty-header"
      className={cn("cn:flex cn:max-w-sm cn:flex-col cn:items-center cn:gap-2", className)}
      {...props}
    />
  )
}

const emptyMediaVariants = cva(
  "cn:mb-2 cn:flex cn:shrink-0 cn:items-center cn:justify-center cn:[&_svg]:pointer-events-none cn:[&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "cn:bg-transparent",
        icon: "cn:flex cn:size-8 cn:shrink-0 cn:items-center cn:justify-center cn:rounded-lg cn:bg-muted cn:text-foreground cn:[&_svg:not([class*=size-])]:size-4",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

function EmptyMedia({
  className,
  variant = "default",
  ...props
}: React.ComponentProps<"div"> & VariantProps<typeof emptyMediaVariants>) {
  return (
    <div
      data-slot="empty-icon"
      data-variant={variant}
      className={cn(emptyMediaVariants({ variant, className }))}
      {...props}
    />
  )
}

function EmptyTitle({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="empty-title"
      className={cn(
        "cn: cn:text-sm cn:font-medium cn:tracking-tight",
        className
      )}
      {...props}
    />
  )
}

function EmptyDescription({ className, ...props }: React.ComponentProps<"p">) {
  return (
    <div
      data-slot="empty-description"
      className={cn(
        "cn:text-sm/relaxed cn:text-muted-foreground cn:[&>a]:underline cn:[&>a]:underline-offset-4 cn:[&>a:hover]:text-primary",
        className
      )}
      {...props}
    />
  )
}

function EmptyContent({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="empty-content"
      className={cn(
        "cn:flex cn:w-full cn:max-w-sm cn:min-w-0 cn:flex-col cn:items-center cn:gap-2.5 cn:text-sm cn:text-balance",
        className
      )}
      {...props}
    />
  )
}

export {
  Empty,
  EmptyHeader,
  EmptyTitle,
  EmptyDescription,
  EmptyContent,
  EmptyMedia,
}
