import Data from '@/data'

type UniqueModTypeData = {
  name: string
  list: () => string[]
}

const modTypes = <K extends string>(src: Record<K, UniqueModTypeData>) => src

export const uniqueModTypes = modTypes({
  list: {
    name: 'Mods from list',
    list: () => []
  },
  clusterNotable: {
    name: 'Cluster Jewel Notables',
    list: () =>
      Object.keys(Data.mods).filter((id) => {
        const mod = Data.mods[id]
        return (
          mod.domain === 'affliction_jewel' &&
          (mod.generation_type === 'suffix' ||
            mod.generation_type === 'prefix') &&
          mod.adds_tags.includes('has_affliction_notable')
        )
      })
  },
  charm: {
    name: 'Charm Mods',
    list: () =>
      Object.keys(Data.mods).filter((id) => {
        const mod = Data.mods[id]
        return (
          mod.domain === 'animal_charm' &&
          (mod.generation_type === 'suffix' || mod.generation_type === 'prefix')
        )
      })
  },
  synthesis: {
    name: 'Synthesis Implicits',
    list: () =>
      Object.keys(Data.mods).filter(
        (id) =>
          id.match(/^SynthesisImplicit/) &&
          !id.match(/^SynthesisImplicit(.*)Jewel\d_*$/)
      )
  },
  synthesisJewel: {
    name: 'Synthesis Jewel Implicits',
    list: () =>
      Object.keys(Data.mods).filter((id) =>
        id.match(/^SynthesisImplicit(.*)Jewel\d_*$/)
      )
  },
  aura: {
    name: 'Aura Modifiers',
    list: () =>
      Object.keys(Data.mods).filter((id) => {
        const mod = Data.mods[id]
        return mod.groups.includes('AuraBonus')
      })
  }
})

export type UniqueModType = keyof typeof uniqueModTypes
