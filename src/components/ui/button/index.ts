import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export { default as Button } from './Button.vue'

export const buttonVariants = cva(
    "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-eve-cyan-dim focus-visible:ring-eve-cyan-dim/50 focus-visible:ring-[3px] aria-invalid:ring-eve-red/20 aria-invalid:border-eve-red",
    {
        variants: {
            variant: {
                default:
                    'border border-eve-border bg-eve-bg-3 text-eve-text-1 hover:bg-eve-bg-hover',
                destructive:
                    'bg-eve-red/60 text-eve-text-1 hover:bg-eve-red/70 focus-visible:ring-eve-red/40',
                outline:
                    'border border-eve-border bg-transparent shadow-xs hover:bg-eve-bg-hover hover:text-eve-text-1',
                secondary:
                    'bg-eve-bg-2 text-eve-text-2 hover:bg-eve-bg-3 hover:text-eve-text-1',
                ghost: 'hover:bg-eve-bg-hover hover:text-eve-text-1',
                link: 'text-eve-cyan underline-offset-4 hover:underline',
            },
            size: {
                default: 'h-9 px-4 py-2 has-[>svg]:px-3',
                sm: 'h-8 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5',
                lg: 'h-10 rounded-md px-6 has-[>svg]:px-4',
                icon: 'size-9',
                'icon-sm': 'size-8',
                'icon-lg': 'size-10',
            },
        },
        defaultVariants: {
            variant: 'default',
            size: 'default',
        },
    }
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
