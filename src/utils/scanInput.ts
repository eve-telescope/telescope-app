export type ScanInputKind = 'local' | 'dscan'

/**
 * Splits a pasted local-scan block into trimmed, non-empty pilot names —
 * the shared definition of "one pilot per line" used for both the pilot
 * count and the cross-fade retain set.
 */
export function splitPilotNames(text: string): string[] {
    return text
        .split('\n')
        .map((name) => name.trim())
        .filter(Boolean)
}

const DSCAN_LINE_PATTERN = /^\d+\t.+\t.+(?:\t.+)?$/

export function detectScanInputKind(text: string): ScanInputKind {
    const lines = text
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean)

    if (lines.length === 0) {
        return 'local'
    }

    const dscanMatches = lines.filter((line) =>
        DSCAN_LINE_PATTERN.test(line)
    ).length
    return dscanMatches > 0 &&
        dscanMatches >= Math.max(1, Math.floor(lines.length / 2))
        ? 'dscan'
        : 'local'
}
