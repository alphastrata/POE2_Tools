import Data from '@/data'
import { AlternatePassiveSkill, PassiveSkillDefinition } from '@/data/types'
import { Item } from '@/types/item'
import { statsExtend } from './mods'

class RandomGenerator {
  private state = new Uint32Array([
    0x40336050, 0xcfa3723c, 0x3cac5f6f, 0x3793fdff
  ])
  constructor(...seed: number[]) {
    const state = this.state
    let index = 1
    for (const v of seed) {
      let round = state[index] ^ state[(index + 1) & 3] ^ state[(index + 3) & 3]
      round = Math.imul(round ^ (round >>> 27), 0x19660d)
      state[(index + 1) & 3] += round
      round = (round + v + index) | 0
      state[(index + 2) & 3] += round
      state[index] = round
      index = (index + 1) & 3
    }
    for (let i = 0; i < 5; ++i) {
      let round = state[index] ^ state[(index + 1) & 3] ^ state[(index + 3) & 3]
      round = Math.imul(round ^ (round >>> 27), 0x19660d)
      state[(index + 1) & 3] += round
      round = (round + index) | 0
      state[(index + 2) & 3] += round
      state[index] = round
      index = (index + 1) & 3
    }
    for (let i = 0; i < 4; ++i) {
      let round =
        (state[index] + state[(index + 1) & 3] + state[(index + 3) & 3]) | 0
      round = Math.imul(round ^ (round >>> 27), 0x5d588b65)
      state[(index + 1) & 3] ^= round
      round = (round - index) | 0
      state[(index + 2) & 3] ^= round
      state[index] = round
      index = (index + 1) & 3
    }
    for (let i = 0; i < 8; ++i) {
      this.next()
    }
  }

  private next() {
    const state = this.state
    let a = state[3]
    let b = (state[0] & 0x7fffffff) ^ state[1] ^ state[2]
    a ^= a << 1
    b ^= (b >>> 1) ^ a
    state[0] = state[1]
    state[1] = state[2]
    state[2] = a ^ (b << 10)
    state[3] = b
    state[1] ^= -(b & 1) & 0x8f7011ee
    state[2] ^= -(b & 1) & 0xfc78ff1f
  }

  private temper() {
    const state = this.state
    let a = state[3]
    let b = (state[0] + (state[2] >>> 8)) | 0
    a ^= b
    if (b & 1) a ^= 0x3793fdff
    return a
  }

  uint() {
    this.next()
    return this.temper() >>> 0
  }

  modulo(mod: number) {
    return this.uint() % mod
  }
  range(a: number, b: number) {
    return (this.uint() % (b - a + 1)) + a
  }
}

function rollStats(
  rng: RandomGenerator,
  stats: AlternatePassiveSkill['stats']
) {
  const result: Record<string, number> = {}
  for (const { id, min, max } of stats) {
    result[id] = max > min ? rng.range(min, max) : max
  }
  return result
}

function createSkill(rng: RandomGenerator, entry: AlternatePassiveSkill) {
  const skill: PassiveSkillDefinition = {
    name: entry.name,
    icon: entry.icon,
    alternate: Data.alternate_tree_versions[entry.version].id,
    stats: rollStats(rng, entry.stats)
  }
  if (entry.flavour_text) skill.flavour_text = entry.flavour_text
  if (entry.types & 4) skill.is_notable = true
  if (entry.types & 8) skill.is_keystone = true
  return skill
}

function skillType(skill: PassiveSkillDefinition) {
  if (skill.is_keystone) return 8
  if (skill.is_notable) return 4
  const keys = Object.keys(skill.stats)
  if (keys.length !== 1) return 2
  return keys[0] === 'base_strength' ||
    keys[0] === 'base_dexterity' ||
    keys[0] === 'base_intelligence'
    ? 1
    : 2
}

export function mutateTreeTimeless(
  id: number,
  skill: PassiveSkillDefinition,
  originalSkill: PassiveSkillDefinition,
  jewel: Item
): PassiveSkillDefinition {
  if (!jewel.stats.unique) return skill
  const {
    local_unique_jewel_alternate_tree_version: version,
    local_unique_jewel_alternate_tree_seed: seed,
    local_unique_jewel_alternate_tree_keystone: keystone,
    local_unique_jewel_alternate_tree_internal_revision: revision
  } = jewel.stats.unique

  const tree = Data.alternate_tree_versions[version]
  if (!tree) return skill
  const rng = new RandomGenerator(id, seed)

  if (skill.is_keystone) {
    const entry =
      Data.alternate_passive_skills.find(
        (sk) =>
          sk.version === version &&
          sk.keystone === keystone &&
          sk.revision === revision
      ) ??
      Data.alternate_passive_skills.find(
        (sk) => sk.version === version && sk.keystone === keystone
      )
    if (!entry) return skill
    return createSkill(rng, entry)
  }

  let replace: boolean
  const type = skillType(originalSkill)
  if (skill.is_notable) {
    const roll = rng.range(0, 100)
    replace =
      tree.replaceNotableWeight >= 100 || roll < tree.replaceNotableWeight
  } else {
    replace = type === 1 ? tree.replaceSmallAttributes : tree.replaceSmallNormal
  }

  let result: PassiveSkillDefinition
  let { randomMin, randomMax } = tree
  if (replace) {
    let candidate: AlternatePassiveSkill | undefined
    let weight = 0
    for (const entry of Data.alternate_passive_skills) {
      if (entry.version !== version) continue
      if (!(entry.types & type)) continue
      weight += entry.weight
      if (rng.modulo(weight) < entry.weight) {
        candidate = entry
      }
    }
    if (!candidate) {
      result = skill
    } else {
      result = createSkill(rng, candidate)
      randomMin = candidate.randomMin
      randomMax = candidate.randomMax
    }
  } else {
    result = skill
  }

  const random =
    randomMax > randomMin ? rng.range(randomMin, randomMax) : randomMax
  if (!random) return result

  if (result === skill) {
    result = {
      ...skill,
      stats: { ...skill.stats },
      baseStats: skill.stats
    }
  } else {
    result.baseStats = { ...result.stats }
  }

  const additions = Data.alternate_passive_additions.filter(
    (row) => row.version === version && row.types & type
  )
  const totalWeight = additions.reduce((sum, row) => sum + row.weight, 0)
  for (let i = 0; i < random; ++i) {
    let roll = rng.modulo(totalWeight)
    let candidate = additions[0]
    for (const row of additions) {
      candidate = row
      if (roll < row.weight) break
      roll -= row.weight
    }
    if (candidate) statsExtend(result.stats, rollStats(rng, candidate.stats))
  }

  return result
}
