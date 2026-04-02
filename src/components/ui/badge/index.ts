import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export { default as Badge } from './Badge.vue'

export const badgeVariants = cva(
    'inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded border px-1.5 py-0.5 text-[9px] font-bold tracking-wide whitespace-nowrap transition-[color,box-shadow] [&>svg]:size-3 [&>svg]:pointer-events-none focus-visible:ring-[3px] focus-visible:ring-eve-cyan/25',
    {
        variants: {
            variant: {
                default:
                    'border-transparent bg-eve-cyan/18 text-eve-cyan [a&]:hover:bg-eve-cyan/25',
                secondary:
                    'border-transparent bg-eve-bg-3 text-eve-text-2 [a&]:hover:bg-eve-bg-hover [a&]:hover:text-eve-text-1',
                destructive:
                    'border-transparent bg-eve-red/18 text-eve-red [a&]:hover:bg-eve-red/25',
                outline:
                    'border-eve-border bg-transparent text-eve-text-2 [a&]:hover:bg-eve-bg-2 [a&]:hover:text-eve-text-1',
            },
        },
        defaultVariants: {
            variant: 'default',
        },
    }
)
export type BadgeVariants = VariantProps<typeof badgeVariants>
