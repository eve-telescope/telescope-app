import type { PilotFlags } from '../types'

export function getFlagLabels(flags: PilotFlags): string[] {
    const labels: string[] = []

    if (flags.is_super) labels.push('SUPER')
    else if (flags.is_capital) labels.push('CAPITAL')

    if (flags.is_blops) labels.push('BLACK OPS')
    if (flags.is_recon) labels.push('RECON')
    if (flags.is_cyno) labels.push('CYNO')

    if (flags.is_solo) labels.push('SOLO')

    return labels
}
