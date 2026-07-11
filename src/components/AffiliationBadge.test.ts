import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import AffiliationBadge from './AffiliationBadge.vue'

describe('AffiliationBadge', () => {
    it('renders corporation logo and name', () => {
        const wrapper = mount(AffiliationBadge, {
            props: {
                type: 'corporation' as const,
                entityId: 98000001,
                name: 'Test Corp',
            },
        })
        const img = wrapper.find('img')
        expect(img.exists()).toBe(true)
        expect(img.attributes('src')).toBe(
            'https://images.evetech.net/corporations/98000001/logo?size=32'
        )
        expect(wrapper.text()).toContain('Test Corp')
    })

    it('renders alliance logo from the alliance endpoint', () => {
        const wrapper = mount(AffiliationBadge, {
            props: {
                type: 'alliance' as const,
                entityId: 99000001,
                name: 'Test Alliance',
            },
        })
        expect(wrapper.find('img').attributes('src')).toBe(
            'https://images.evetech.net/alliances/99000001/logo?size=32'
        )
    })

    it('omits the logo when the entity id is missing', () => {
        const wrapper = mount(AffiliationBadge, {
            props: {
                type: 'alliance' as const,
                entityId: null,
                name: 'Unknown Alliance',
            },
        })
        expect(wrapper.find('img').exists()).toBe(false)
        expect(wrapper.text()).toContain('Unknown Alliance')
    })

    it('falls back to an em-dash when there is no name', () => {
        const wrapper = mount(AffiliationBadge, {
            props: { type: 'corporation' as const },
        })
        expect(wrapper.text()).toContain('—')
        expect(wrapper.find('img').exists()).toBe(false)
    })
})
