import { Button as ButtonPrimitive } from "@base-ui/react/button"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/cn"

const buttonVariants = cva(
  "cn:group/button cn:inline-flex cn:shrink-0 cn:items-center cn:justify-center cn:rounded-lg cn:border cn:border-transparent cn:bg-clip-padding cn:text-sm cn:font-medium cn:whitespace-nowrap cn:transition-all cn:outline-none cn:select-none cn:focus-visible:border-ring cn:focus-visible:ring-3 cn:focus-visible:ring-ring/50 cn:active:not-aria-[haspopup]:translate-y-px cn:disabled:pointer-events-none cn:disabled:opacity-50 cn:aria-invalid:border-destructive cn:aria-invalid:ring-3 cn:aria-invalid:ring-destructive/20 cn:dark:aria-invalid:border-destructive/50 cn:dark:aria-invalid:ring-destructive/40 cn:[&_svg]:pointer-events-none cn:[&_svg]:shrink-0 cn:[&_svg:not([class*=size-])]:size-4",
  {
    variants: {
      variant: {
        default: "cn:bg-primary cn:text-primary-foreground cn:hover:bg-primary/80",
        outline:
          "cn:border-border cn:bg-background cn:hover:bg-muted cn:hover:text-foreground cn:aria-expanded:bg-muted cn:aria-expanded:text-foreground cn:dark:border-input cn:dark:bg-input/30 cn:dark:hover:bg-input/50",
        secondary:
          "cn:bg-secondary cn:text-secondary-foreground cn:hover:bg-[color-mix(in_oklch,var(--secondary),var(--foreground)_5%)] cn:aria-expanded:bg-secondary cn:aria-expanded:text-secondary-foreground",
        ghost:
          "cn:hover:bg-muted cn:hover:text-foreground cn:aria-expanded:bg-muted cn:aria-expanded:text-foreground cn:dark:hover:bg-muted/50",
        destructive:
          "cn:bg-destructive/10 cn:text-destructive cn:hover:bg-destructive/20 cn:focus-visible:border-destructive/40 cn:focus-visible:ring-destructive/20 cn:dark:bg-destructive/20 cn:dark:hover:bg-destructive/30 cn:dark:focus-visible:ring-destructive/40",
        link: "cn:text-primary cn:underline-offset-4 cn:hover:underline",
      },
      size: {
        default:
          "cn:h-8 cn:gap-1.5 cn:px-2.5 cn:has-data-[icon=inline-end]:pr-2 cn:has-data-[icon=inline-start]:pl-2",
        xs: "cn:h-6 cn:gap-1 cn:rounded-[min(var(--radius-md),10px)] cn:px-2 cn:text-xs cn:in-data-[slot=button-group]:rounded-lg cn:has-data-[icon=inline-end]:pr-1.5 cn:has-data-[icon=inline-start]:pl-1.5 cn:[&_svg:not([class*=size-])]:size-3",
        sm: "cn:h-7 cn:gap-1 cn:rounded-[min(var(--radius-md),12px)] cn:px-2.5 cn:text-[0.8rem] cn:in-data-[slot=button-group]:rounded-lg cn:has-data-[icon=inline-end]:pr-1.5 cn:has-data-[icon=inline-start]:pl-1.5 cn:[&_svg:not([class*=size-])]:size-3.5",
        lg: "cn:h-9 cn:gap-1.5 cn:px-2.5 cn:has-data-[icon=inline-end]:pr-2 cn:has-data-[icon=inline-start]:pl-2",
        icon: "cn:size-8",
        "icon-xs":
          "cn:size-6 cn:rounded-[min(var(--radius-md),10px)] cn:in-data-[slot=button-group]:rounded-lg cn:[&_svg:not([class*=size-])]:size-3",
        "icon-sm":
          "cn:size-7 cn:rounded-[min(var(--radius-md),12px)] cn:in-data-[slot=button-group]:rounded-lg",
        "icon-lg": "cn:size-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

function Button({
  className,
  variant = "default",
  size = "default",
  ...props
}: ButtonPrimitive.Props & VariantProps<typeof buttonVariants>) {
  return (
    <ButtonPrimitive
      data-slot="button"
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  )
}

export { Button, buttonVariants }
