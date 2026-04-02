import { describe, it, expect } from 'vitest'
import { detectScanInputKind } from './scanInput'

describe('detectScanInputKind', () => {
    it('detects local scan (pilot names)', () => {
        const input = 'Pilot One\nPilot Two\nPilot Three'
        expect(detectScanInputKind(input)).toBe('local')
    })

    it('detects d-scan (tab-separated with IDs)', () => {
        const input =
            '28356\tRifter\tPilot Ship\t100 km\n' +
            '670\tCapsule\tPilot Pod\t50 km'
        expect(detectScanInputKind(input)).toBe('dscan')
    })

    it('detects local when mixed content is mostly names', () => {
        const input =
            'Pilot One\nPilot Two\nPilot Three\nPilot Four\n12345\tShip\tType\t10km'
        expect(detectScanInputKind(input)).toBe('local')
    })

    it('detects dscan when majority are tab-separated', () => {
        const input =
            '28356\tRifter\tShip\t100 km\n' +
            '670\tCapsule\tPod\t50 km\n' +
            '123\tAtron\tFrig\t200 km\n' +
            'Random Name'
        expect(detectScanInputKind(input)).toBe('dscan')
    })

    it('handles empty input', () => {
        expect(detectScanInputKind('')).toBe('local')
    })

    it('handles single pilot name', () => {
        expect(detectScanInputKind('Solo Pilot')).toBe('local')
    })
})
