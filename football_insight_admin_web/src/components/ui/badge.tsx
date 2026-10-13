import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

// 项目徽章变体映射到 styles.css 中的既有样式：
// default = 计数胶囊(.count)，status = 状态胶囊(.status-pill)，
// 其余 = ui-badge 系列。
const badgeVariants = cva("", {
  variants: {
    variant: {
      default: "count",
      status: "status-pill",
      secondary: "ui-badge ui-badge-secondary",
      outline: "ui-badge ui-badge-outline",
      destructive: "ui-badge ui-badge-destructive",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

export type BadgeProps = React.ComponentProps<"div"> &
  VariantProps<typeof badgeVariants> & {
    asChild?: boolean;
  };

function Badge({ className, variant, asChild = false, ...props }: BadgeProps) {
  const Comp = asChild ? Slot : "div";

  return (
    <Comp
      data-slot="badge"
      className={cn(badgeVariants({ variant }), className)}
      {...props}
    />
  );
}

export { Badge, badgeVariants };
