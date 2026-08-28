"use client"

import * as React from "react"
import { Dialog as DialogPrimitive } from "@base-ui/react/dialog"

import { cn } from "@/lib/cn"
import { Button } from "@/components/ui/ui/button"
import { XIcon } from "lucide-react"

function Dialog({ ...props }: DialogPrimitive.Root.Props) {
  return <DialogPrimitive.Root data-slot="dialog" {...props} />
}

function DialogTrigger({ ...props }: DialogPrimitive.Trigger.Props) {
  return <DialogPrimitive.Trigger data-slot="dialog-trigger" {...props} />
}

function DialogPortal({ ...props }: DialogPrimitive.Portal.Props) {
  return <DialogPrimitive.Portal data-slot="dialog-portal" {...props} />
}

function DialogClose({ ...props }: DialogPrimitive.Close.Props) {
  return <DialogPrimitive.Close data-slot="dialog-close" {...props} />
}

function DialogOverlay({
  className,
  ...props
}: DialogPrimitive.Backdrop.Props) {
  return (
    <DialogPrimitive.Backdrop
      data-slot="dialog-overlay"
      className={cn(
        "cn:fixed cn:inset-0 cn:isolate cn:z-50 cn:bg-black/10 cn:duration-100 cn:supports-backdrop-filter:backdrop-blur-xs cn:data-open:animate-in cn:data-open:fade-in-0 cn:data-closed:animate-out cn:data-closed:fade-out-0",
        className
      )}
      {...props}
    />
  )
}

function DialogContent({
  className,
  children,
  showCloseButton = true,
  ...props
}: DialogPrimitive.Popup.Props & {
  showCloseButton?: boolean
}) {
  return (
    <DialogPortal>
      <DialogOverlay />
      <DialogPrimitive.Popup
        data-slot="dialog-content"
        className={cn(
          "cn:fixed cn:top-1/2 cn:left-1/2 cn:z-50 cn:grid cn:w-full cn:max-w-[calc(100%-2rem)] cn:-translate-x-1/2 cn:-translate-y-1/2 cn:gap-4 cn:rounded-xl cn:bg-popover cn:p-4 cn:text-sm cn:text-popover-foreground cn:ring-1 cn:ring-foreground/10 cn:duration-100 cn:outline-none cn:sm:max-w-sm cn:data-open:animate-in cn:data-open:fade-in-0 cn:data-open:zoom-in-95 cn:data-closed:animate-out cn:data-closed:fade-out-0 cn:data-closed:zoom-out-95",
          className
        )}
        {...props}
      >
        {children}
        {showCloseButton && (
          <DialogPrimitive.Close
            data-slot="dialog-close"
            render={
              <Button
                variant="ghost"
                className="cn:absolute cn:top-2 cn:right-2"
                size="icon-sm"
              />
            }
          >
            <XIcon
            />
            <span className="cn:sr-only">Close</span>
          </DialogPrimitive.Close>
        )}
      </DialogPrimitive.Popup>
    </DialogPortal>
  )
}

function DialogHeader({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="dialog-header"
      className={cn("cn:flex cn:flex-col cn:gap-2", className)}
      {...props}
    />
  )
}

function DialogFooter({
  className,
  showCloseButton = false,
  children,
  ...props
}: React.ComponentProps<"div"> & {
  showCloseButton?: boolean
}) {
  return (
    <div
      data-slot="dialog-footer"
      className={cn(
        "cn:-mx-4 cn:-mb-4 cn:flex cn:flex-col-reverse cn:gap-2 cn:rounded-b-xl cn:border-t cn:bg-muted/50 cn:p-4 cn:sm:flex-row cn:sm:justify-end",
        className
      )}
      {...props}
    >
      {children}
      {showCloseButton && (
        <DialogPrimitive.Close render={<Button variant="outline" />}>
          Close
        </DialogPrimitive.Close>
      )}
    </div>
  )
}

function DialogTitle({ className, ...props }: DialogPrimitive.Title.Props) {
  return (
    <DialogPrimitive.Title
      data-slot="dialog-title"
      className={cn(
        "cn: cn:text-base cn:leading-none cn:font-medium",
        className
      )}
      {...props}
    />
  )
}

function DialogDescription({
  className,
  ...props
}: DialogPrimitive.Description.Props) {
  return (
    <DialogPrimitive.Description
      data-slot="dialog-description"
      className={cn(
        "cn:text-sm cn:text-muted-foreground cn:*:[a]:underline cn:*:[a]:underline-offset-3 cn:*:[a]:hover:text-foreground",
        className
      )}
      {...props}
    />
  )
}

export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
}
