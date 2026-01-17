import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center whitespace-nowrap font-medium transition-all duration-300 ease focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 cursor-pointer",
  {
    variants: {
      variant: {
        default:
          "bg-primary text-primary-foreground px-12 py-4 text-base font-medium tracking-button hover:bg-hover hover:-translate-y-[1px]",
        destructive:
          "bg-primary text-primary-foreground px-12 py-4 text-base font-medium tracking-button hover:bg-hover hover:-translate-y-[1px]",
        outline:
          "bg-transparent text-foreground px-[46px] py-[14px] border-2 border-border text-base font-medium hover:border-hover hover:text-hover",
        secondary:
          "bg-transparent text-foreground px-[46px] py-[14px] border-2 border-border text-base font-medium hover:border-hover hover:text-hover",
        ghost: "bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline bg-transparent",
      },
      size: {
        default: "px-12 py-4 text-base",
        sm: "px-4 py-2 text-sm",
        lg: "px-12 py-4 text-lg",
        icon: "h-10 w-10 p-0",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <button className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props} />
    );
  }
);
Button.displayName = "Button";

export { Button, buttonVariants };
