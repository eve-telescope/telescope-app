import { describe, it, expect } from 'vitest'
import {
    normalizeAnnotationTag,
    parseAnnotationTags,
    serializeAnnotationTags,
    getAnnotationColor,
    getPresetAnnotationColor,
    PRESET_ANNOTATION_TAGS,
} from './annotations'

describe('normalizeAnnotationTag', () => {
    it('uppercases and trims', () => {
        expect(normalizeAnnotationTag('  hostile  ')).toBe('HOSTILE')
    })

    it('collapses whitespace', () => {
        expect(normalizeAnnotationTag('black  ops')).toBe('BLACK OPS')
    })

    it('handles empty string', () => {
        expect(normalizeAnnotationTag('')).toBe('')
    })
})

describe('parseAnnotationTags', () => {
    it('splits by pipe', () => {
        expect(parseAnnotationTags('HOSTILE | SPY')).toEqual(['HOSTILE', 'SPY'])
    })

    it('normalizes each tag', () => {
        expect(parseAnnotationTags('hostile | spy')).toEqual(['HOSTILE', 'SPY'])
    })

    it('filters empty segments', () => {
        expect(parseAnnotationTags('HOSTILE | | SPY')).toEqual([
            'HOSTILE',
            'SPY',
        ])
    })

    it('handles empty string', () => {
        expect(parseAnnotationTags('')).toEqual([])
    })
})

describe('serializeAnnotationTags', () => {
    it('joins with pipe delimiter', () => {
        expect(serializeAnnotationTags(['HOSTILE', 'SPY'])).toBe(
            'HOSTILE | SPY'
        )
    })

    it('deduplicates', () => {
        expect(serializeAnnotationTags(['HOSTILE', 'hostile'])).toBe('HOSTILE')
    })

    it('handles empty array', () => {
        expect(serializeAnnotationTags([])).toBe('')
    })
})

describe('getPresetAnnotationColor', () => {
    it('returns color for known preset', () => {
        expect(getPresetAnnotationColor('HOSTILE')).toBe('#FF3B3B')
    })

    it('returns null for unknown tag', () => {
        expect(getPresetAnnotationColor('CUSTOM')).toBeNull()
    })

    it('normalizes before lookup', () => {
        expect(getPresetAnnotationColor('hostile')).toBe('#FF3B3B')
    })
})

describe('getAnnotationColor', () => {
    it('returns first matching preset color', () => {
        expect(getAnnotationColor(['CUSTOM', 'HOSTILE'])).toBe('#FF3B3B')
    })

    it('returns fallback when no preset matches', () => {
        expect(getAnnotationColor(['CUSTOM'], '#000000')).toBe('#000000')
    })

    it('returns null with no match and no fallback', () => {
        expect(getAnnotationColor(['CUSTOM'])).toBeNull()
    })
})

describe('PRESET_ANNOTATION_TAGS', () => {
    it('has expected presets', () => {
        const tags = PRESET_ANNOTATION_TAGS.map((p) => p.tag)
        expect(tags).toContain('HOSTILE')
        expect(tags).toContain('FRIENDLY')
        expect(tags).toContain('SPY')
        expect(tags).toContain('NEUTRAL')
    })
})
