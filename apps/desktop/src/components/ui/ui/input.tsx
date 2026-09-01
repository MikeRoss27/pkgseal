import { type ComponentProps } from "react"

import { cn } from "@/lib/cn"

function Input({ className, type = "text", ...props }: ComponentProps<"input">) {
  return (
    <input
      type={type}
      data-slot="input"
      className={cn(
        "cn:flex cn:h-8 cn:w-full cn:rounded-lg cn:border cn:border-border cn:bg-background cn:px-2.5 cn:text-sm cn:text-foreground cn:outline-none cn:transition-all cn:placeholder:text-muted-foreground cn:focus-visible:border-ring cn:focus-visible:ring-3 cn:focus-visible:ring-ring/50 cn:disabled:pointer-events-none cn:disabled:opacity-50 cn:dark:border-input cn:dark:bg-input/30",
        className,
      )}
      {...props}
    />
  )
}

export { Input }
