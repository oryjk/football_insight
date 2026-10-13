import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap [&_svg]:pointer-events-none [&_svg]:shrink-0",
  {
    variants: {
      // Visuals are owned by the shared data-slot rules in foundation.css.
      variant: {
        primary: "",
        default: "", // compatibility alias for primary
        secondary: "",
        outline: "", // compatibility alias for secondary
        destructive: "",
        ghost: "",
        link: "",
      },
      size: {
        default: "",
        sm: "", // dense console actions share the 36px control height
        lg: "",
        icon: "",
        "icon-sm": "",
        inline: "",
        content: "", // existing disclosure / card-shaped interaction geometry
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

export type ButtonProps = React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean;
  };

function Button({
  className,
  variant,
  size,
  asChild = false,
  ...props
}: ButtonProps) {
  const Comp = asChild ? Slot : "button";

  return (
    <Comp
      {...props}
      data-slot="button"
      data-variant={variant ?? "default"}
      data-size={size ?? "default"}
      className={cn(buttonVariants({ variant, size, className }))}
    />
  );
}

export { Button, buttonVariants };
