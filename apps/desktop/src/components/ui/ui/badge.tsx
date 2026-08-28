import { mergeProps } from "@base-ui/react/merge-props"
import { useRender } from "@base-ui/react/use-render"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/cn"

const badgeVariants = cva(
  "cn:group/badge cn:inline-flex cn:h-5 cn:w-fit cn:shrink-0 cn:items-center cn:justify-center cn:gap-1 cn:overflow-hidden cn:rounded-4xl cn:border cn:border-transparent cn:px-2 cn:py-0.5 cn:text-xs cn:font-medium cn:whitespace-nowrap cn:transition-all cn:focus-visible:border-ring cn:focus-visible:ring-[3px] cn:focus-visible:ring-ring/50 cn:has-data-[icon=inline-end]:pr-1.5 cn:has-data-[icon=inline-start]:pl-1.5 cn:aria-invalid:border-destructive cn:aria-invalid:ring-destructive/20 cn:dark:aria-invalid:ring-destructive/40 cn:[&>svg]:pointer-events-none cn:[&>svg]:size-3!",
  {
    variants: {
      variant: {
        default: "cn:bg-primary cn:text-primary-foreground cn:[a]:hover:bg-primary/80",
        secondary:
          "cn:bg-secondary cn:text-secondary-foreground cn:[a]:hover:bg-secondary/80",
        destructive:
          "cn:bg-destructive/10 cn:text-destructive cn:focus-visible:ring-destructive/20 cn:dark:bg-destructive/20 cn:dark:focus-visible:ring-destructive/40 cn:[a]:hover:bg-destructive/20",
        outline:
          "cn:border-border cn:text-foreground cn:[a]:hover:bg-muted cn:[a]:hover:text-muted-foreground",
        ghost:
          "cn:hover:bg-muted cn:hover:text-muted-foreground cn:dark:hover:bg-muted/50",
        link: "cn:text-primary cn:underline-offset-4 cn:hover:underline",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

function Badge({
  className,
  variant = "default",
  render,
  ...props
}: useRender.ComponentProps<"span"> & VariantProps<typeof badgeVariants>) {
  return useRender({
    defaultTagName: "span",
    props: mergeProps<"span">(
      {
        className: cn(badgeVariants({ variant }), className),
      },
      props
    ),
    render,
    state: {
      slot: "badge",
      variant,
    },
  })
}

export { Badge, badgeVariants }
