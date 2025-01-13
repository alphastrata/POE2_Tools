import Data from '@/data'
import {
  PassiveSkillDefinition,
  StatSemanticEntry,
  StatSemanticTreeBonus,
  StatSemanticTreeMutation,
  TreeNodeFilter
} from '@/data/types'
import { Item } from '@/types/item'
import { notNull } from '@/utils/validation'
import { statsExtend } from './mods'
import { radiusJewel } from './tree'
import { TreeDataProvider } from './treeData'
import { mutateTreeTimeless } from './treeTimeless'

function filterNode(
  filter: TreeNodeFilter,
  active: boolean,
  skill: PassiveSkillDefinition
) {
  if (filter.allocated != null) {
    if (filter.allocated !== active) return false
  }
  switch (filter.size) {
    case 'small':
      return !skill.is_notable && !skill.is_keystone
    case 'notable':
      return skill.is_notable && !skill.is_keystone
    case 'keystone':
      return skill.is_keystone
    case 'non_keystone':
      return !skill.is_keystone
    case 'tatoo':
      return false
    default:
      return true
  }
}

export function mutateTreeSkill(
  id: number,
  active: boolean,
  skill: PassiveSkillDefinition,
  originalSkill: PassiveSkillDefinition,
  jewels: Item[]
) {
  const added_stats: Record<string, number> = {}
  let node_disabled = false

  const timelessJewel = jewels.find(
    (jwl) => jwl.stats.unique?.local_is_alternate_tree_jewel
  )
  if (timelessJewel) {
    return mutateTreeTimeless(id, skill, originalSkill, timelessJewel)
  }

  function processMutation(op: StatSemanticTreeMutation, value: number) {
    if (!filterNode(op, active, skill)) return

    if (op.type === 'disable') {
      node_disabled = true
    } else if (op.type === 'stat') {
      added_stats[op.id] =
        (added_stats[op.id] ?? 0) + (op.value ?? value) * (op.factor ?? 1)
    } else if (op.type === 'convert' || op.type === 'add') {
      const scale =
        1 + 0.01 * (op.percent === 'value' ? value : op.percent ?? 0)
      if ('from' in op) {
        const amount = skill.stats[op.from]
        if (amount) {
          added_stats[op.to] = (added_stats[op.to] ?? 0) + amount * scale
          if (op.type === 'convert') {
            added_stats[op.from] = (added_stats[op.from] ?? 0) - amount
          }
        }
      } else {
        for (const [id, amount] of Object.entries(skill.stats)) {
          const to = op.map[id]
          if (!to) continue
          added_stats[to] = (added_stats[to] ?? 0) + amount * scale
          if (op.type === 'convert') {
            added_stats[id] = (added_stats[id] ?? 0) - amount
          }
        }
      }
    } else if (op.type === 'amplify') {
      const scale = 0.01 * (op.percent === 'value' ? value : op.percent)
      for (const [id, amount] of Object.entries(skill.stats)) {
        added_stats[id] = (added_stats[id] ?? 0) + amount * scale
      }
    }
  }

  function processStat(semantic: StatSemanticEntry, value: number) {
    if (semantic.tree_mutation) {
      if (Array.isArray(semantic.tree_mutation)) {
        for (const op of semantic.tree_mutation) processMutation(op, value)
      } else {
        processMutation(semantic.tree_mutation, value)
      }
    }
  }

  for (const jewel of jewels) {
    for (const group of Object.values(jewel.stats).filter(notNull)) {
      for (const [id, value] of Object.entries(group)) {
        const semantic = Data.semantics[id]
        if (!semantic) continue

        if (Array.isArray(semantic)) {
          for (const s of semantic) processStat(s, value)
        } else {
          processStat(semantic, value)
        }
      }
    }
  }

  if (node_disabled) {
    return {
      ...skill,
      stats: {},
      aura: undefined
    }
  }

  if (Object.keys(added_stats).length) {
    skill = { ...skill }
    skill.stats = statsExtend(added_stats, skill.stats)
  }
  return skill
}

export function calculateJewelStats(data: TreeDataProvider, id: number) {
  const node = data.node(id)!
  const jewel = node.jewel!

  const stats: Record<string, number> = {}

  const radius = radiusJewel(data, id)
  const nodes = radius && [...data.nodesInRadius(radius)]

  function processBonus(bonus: StatSemanticTreeBonus, jewelValue: number) {
    if (!nodes) return
    const total: Record<string, number> = {
      node_count: 0
    }
    for (const id of nodes) {
      const node = data.node(id)!
      if (!filterNode(bonus, node.active, node.skill)) continue
      const skill = data.nodeSkill(id)!
      statsExtend(total, skill.stats)
      total.node_count += 1
    }
    if (bonus.from === 'any') {
      statsExtend(stats, total)
    } else {
      const value = Array.isArray(bonus.from)
        ? bonus.from.reduce((s, id) => s + (total[id] ?? 0), 0)
        : total[bonus.from] ?? 0
      if (bonus.threshold != null) {
        if (
          value >= (bonus.threshold === 'value' ? jewelValue : bonus.threshold)
        ) {
          stats[bonus.to] = (stats[bonus.to] ?? 0) + (bonus.value ?? jewelValue)
        }
      } else if (bonus.divisor != null) {
        const divisor = bonus.divisor === 'value' ? jewelValue : bonus.divisor
        if (value >= divisor) {
          stats[bonus.to] =
            (stats[bonus.to] ?? 0) +
            (bonus.value ?? jewelValue) * Math.floor(value / divisor)
        }
      } else if (value) {
        stats[bonus.to] =
          (stats[bonus.to] ?? 0) + (bonus.value ?? jewelValue) * value
      }
    }
  }

  for (const group of Object.values(jewel.stats).filter(notNull)) {
    for (const [id, value] of Object.entries(group)) {
      const semantic = Data.semantics[id]
      if (
        !semantic ||
        Array.isArray(semantic) ||
        semantic.stat !== 'jewel_tree_transform'
      ) {
        stats[id] = (stats[id] ?? 0) + value
        continue
      }

      if (semantic.tree_bonus) {
        if (Array.isArray(semantic.tree_bonus)) {
          for (const bonus of semantic.tree_bonus) processBonus(bonus, value)
        } else {
          processBonus(semantic.tree_bonus, value)
        }
      }
    }
  }

  return stats
}
