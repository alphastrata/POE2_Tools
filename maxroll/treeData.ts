import Data from '@/data'
import {
  AtlasSubTree,
  ClusterJewelSize,
  PassiveSkillDefinition,
  PassiveTreeDefinition,
  PassiveTreeLegacyNode,
  PassiveTreeNode
} from '@/data/types'
import { StoreState } from '@/store'
import { Item } from '@/types/item'
import { cachedMethod } from '@/utils/cached'
import { statsExtend, statsScale } from './mods'
import { NodeState, RadiusJewel, TreeViewState, radiusJewel } from './tree'
import { calculateJewelStats, mutateTreeSkill } from './treeMutation'

export const attributeSkills = [
  'generic_attribute_strength',
  'generic_attribute_dexterity',
  'generic_attribute_intelligence_'
]

const clusterNodeBase = 131072

//const orbitRadii = [0, 82, 162, 335, 493, 662, 846]
//const skillsPerOrbit = [1, 6, 16, 16, 40, 72, 72]
export const orbitRadii = [0, 82, 162, 335, 493, 662, 846, 251, 1080, 1322]
const skillsPerOrbit = [1, 12, 24, 24, 72, 72, 72, 24, 72, 144]

function getOrbitAngle(size: number, index: number) {
  if (size === 16) {
    return [
      0, 30, 45, 60, 90, 120, 135, 150, 180, 210, 225, 240, 270, 300, 315, 330
    ][index]
  } else if (size === 40) {
    return [
      0, 10, 20, 30, 40, 45, 50, 60, 70, 80, 90, 100, 110, 120, 130, 135, 140,
      150, 160, 170, 180, 190, 200, 210, 220, 225, 230, 240, 250, 260, 270, 280,
      290, 300, 310, 315, 320, 330, 340, 350
    ][index]
  } else {
    return (360 * index) / size
  }
}

export type GroupData = {
  proxy?: boolean
  bg?: number
  x: number
  y: number
  ascendancy?: string
  nodes: number[]
}

export type NodeData = {
  group: GroupData
  x: number
  y: number
  radius: number
  arc: number
  connections: {
    id: number
    radius: number
  }[]
  clusterNode?: boolean
  skill_id: string
  skill: PassiveSkillDefinition
  immutable: boolean
  state: NodeState
  weaponSet: number
  active: boolean
  masteryIndex?: number
  jewel?: Item
}

type ClusterData = {
  nodes: number
  sockets: number
  notables: string[]
  skill: string
  mastery?: string
  stats: Record<string, number>
}

function clusterData(item: Item) {
  const base = Data.items[item.base]
  if (!base.cluster_jewel) return
  const data: ClusterData = {
    nodes: 0,
    sockets: 0,
    notables: [],
    skill: 'affliction_empty_node_',
    stats: {}
  }
  if (!item.unique && item.clusterJewel) {
    data.nodes = item.clusterJewel.nodes
    const skill = base.cluster_jewel.passive_skills.find(
      (sk) => sk.id === item.clusterJewel!.skill
    )
    if (skill && Data.passive_skills[skill.id]) {
      data.skill = skill.id
      data.mastery = skill.mastery
      statsExtend(data.stats, Data.passive_skills[skill.id].stats)
    }
    if (base.cluster_jewel.size === 'Large') data.sockets = 2
    if (base.cluster_jewel.size === 'Medium') data.sockets = 1
  }
  const stats = statsExtend({}, ...Object.values(item.stats))
  for (const notable of Data.cluster_jewel_notables) {
    if (stats[notable.jewel_stat]) data.notables.push(notable.id)
  }
  for (const [id, stat] of Object.entries(Data.affliction_stats)) {
    if (stats[id]) statsExtend(data.stats, { [stat]: stats[id] })
  }
  if (stats.local_jewel_expansion_passive_node_count) {
    data.nodes = stats.local_jewel_expansion_passive_node_count
  }
  if (stats.local_jewel_expansion_jewels_count_override) {
    data.sockets = stats.local_jewel_expansion_jewels_count_override
  }
  data.nodes = Math.max(data.nodes, data.sockets + data.notables.length)
  if (stats.local_unique_jewel_grants_x_empty_passives) {
    data.nodes += stats.local_unique_jewel_grants_x_empty_passives
  }
  if (stats['local_affliction_jewel_small_nodes_have_effect_+%']) {
    data.stats = statsScale(
      data.stats,
      1 + 0.01 * stats['local_affliction_jewel_small_nodes_have_effect_+%']
    )
  }
  return data
}

export class TreeDataProvider {
  skills: Record<string, PassiveSkillDefinition>
  atlas_sub_trees: Record<string, AtlasSubTree>
  tree: PassiveTreeDefinition<PassiveTreeNode | PassiveTreeLegacyNode>

  constructor(public state: TreeViewState, public items: StoreState['items']) {
    const tree = Data.trees[state.version]!
    this.skills =
      'passive_skills' in tree ? tree.passive_skills : Data.passive_skills
    this.atlas_sub_trees =
      'atlas_sub_trees' in tree ? tree.atlas_sub_trees : Data.atlas_sub_trees
    this.tree = 'atlas' in state ? tree.atlas_passive_tree : tree.passive_tree
  }

  get atlas() {
    return 'atlas' in this.state
  }

  private useAscendancy(ascendancy: string | undefined) {
    const { state } = this
    if ('atlas' in state) return true
    if (state.ascendancyOnly) {
      return ascendancy === state.ascendancy
    } else {
      return !ascendancy || ascendancy === state.ascendancy
    }
  }

  showAscendancy(ascendancy: string | undefined) {
    const { state } = this
    if ('atlas' in state) return true
    if (state.ascendancyOnly) {
      return ascendancy === state.ascendancy
    } else {
      return !ascendancy || ascendancy === state.ascendancy
    }
  }

  @cachedMethod
  private clusterNodes() {
    const nodes: Record<number, NodeData> = {}
    const groups: Record<number, GroupData> = {}

    const proxyFor = (id: number) => {
      const node = this.tree.nodes[id]
      if (!node) return
      for (const { id: eid } of node.connections) {
        const next = this.tree.nodes[eid]
        if (!next || next.parent === node.parent) continue
        const skill = 'skill_id' in next ? this.skills[next.skill_id] : next
        if (skill.is_proxy) return eid
      }
    }

    const downgradeSocket = (
      id: number,
      size: ClusterJewelSize
    ): PassiveTreeNode | PassiveTreeLegacyNode => {
      const node = this.tree.nodes[id]
      if (!('skill_id' in node)) return node
      const socketSize = Data.jewel_slots[node.skill_id]?.size
      if (!socketSize) return node
      if (socketSize === 'Small' && size !== 'Small') return node
      if (socketSize !== 'Large' && size === 'Large') return node
      const proxy = proxyFor(id)
      if (!proxy) return node
      const proxyNode = this.tree.nodes[proxy]
      const proxySocket = proxyNode.connections.find(
        ({ id }) => this.tree.nodes[id].parent === proxyNode.parent
      )
      if (!proxySocket) return node
      return downgradeSocket(proxySocket.id, size)
    }

    const createCluster = (id: number, proxy: number) => {
      const jewelId = this.state.jewels[id]
      const jewel = jewelId != null && this.items[jewelId]
      if (!jewel) return
      const jewelData = clusterData(jewel)
      if (!jewelData || !jewelData.nodes) return
      const clusterBase = Data.items[jewel.base]?.cluster_jewel
      if (!clusterBase) return

      const proxyNode = this.tree.nodes[proxy]
      const proxyGroup = this.tree.groups[proxyNode.parent]

      const radius =
        clusterBase.size === 'Large'
          ? 3
          : clusterBase.size === 'Medium'
          ? 2
          : jewelData.nodes > 1
          ? 1
          : 0

      const clusterGroup: GroupData = {
        proxy: true,
        x: proxyGroup.x,
        y: proxyGroup.y,
        bg: radius,
        nodes: []
      }
      groups[proxyNode.parent] = clusterGroup

      const startAngle = getOrbitAngle(
        skillsPerOrbit[proxyNode.radius],
        proxyNode.position
      )

      const baseNode = clusterNodeBase + proxyNode.parent * 16
      const indices = clusterBase.small_indices
        .slice(0, jewelData.nodes)
        .sort((a, b) => a - b)
      const socketIndices = clusterBase.socket_indices
        .filter((i) => indices.includes(i))
        .slice(0, jewelData.sockets)
      const notableIndices = clusterBase.notable_indices
        .filter(
          (i) => indices.includes(i) && !socketIndices.includes(i) && i !== 0
        )
        .slice(0, jewelData.notables.length)
      while (notableIndices.length < jewelData.notables.length) {
        const index = indices.find(
          (i) =>
            i !== 0 && !socketIndices.includes(i) && !notableIndices.includes(i)
        )
        if (index != null) notableIndices.push(index)
        else break
      }
      if (notableIndices.length < jewelData.notables.length) {
        notableIndices.push(0)
      }
      let notablePos = 0
      const smallSkill = {
        ...this.skills[jewelData.skill],
        stats: jewelData.stats
      }
      for (const index of indices) {
        let angle =
          (startAngle + (360 * index) / clusterBase.total_indices) % 360
        if (clusterBase.size === 'Medium') {
          if (index === 4 && !indices.includes(2))
            angle = (startAngle + 90) % 360
          if (index === 8 && !indices.includes(10))
            angle = (startAngle + 270) % 360
        } else if (clusterBase.size === 'Small') {
          if (proxyGroup.bg === 3) angle = (angle + 30) % 360
          if (proxyGroup.bg === 2) angle = (angle + 330) % 360
        }
        const arc = ((angle - 90) * Math.PI) / 180
        const nodeId = index ? baseNode + index : proxy
        const state = (this.state.nodes[nodeId] ?? 0) & NodeState.StateMask
        const node: NodeData = {
          group: clusterGroup,
          skill_id: jewelData.skill,
          skill: smallSkill,
          x: clusterGroup.x + orbitRadii[radius] * Math.cos(arc),
          y: clusterGroup.y + orbitRadii[radius] * Math.sin(arc),
          radius: orbitRadii[radius],
          arc,
          state,
          weaponSet:
            (this.state.nodes[nodeId] ?? 0) >> NodeState.WeaponSetShift,
          active:
            state === NodeState.Active ||
            state === NodeState.Remove ||
            state === NodeState.AddHistory,
          connections: [],
          immutable: false,
          clusterNode: true
        }
        nodes[nodeId] = node

        if (socketIndices.includes(index)) {
          const baseSocket = proxyGroup.nodes.find((id) => {
            const node = this.tree.nodes[id]
            return (
              angle ===
              getOrbitAngle(skillsPerOrbit[node.radius], node.position)
            )
          })
          if (baseSocket) {
            const socketNode = downgradeSocket(baseSocket, clusterBase.size)
            node.skill_id =
              'skill_id' in socketNode
                ? socketNode.skill_id
                : socketNode.ref_id ?? baseSocket.toString()
            node.skill =
              'skill_id' in socketNode
                ? this.skills[socketNode.skill_id]
                : socketNode
            const socketProxy = proxyFor(baseSocket)
            if (socketProxy != null && createCluster(nodeId, socketProxy)) {
              node.connections.push({ id: socketProxy, radius: 0 })
            }
            if (this.state.jewels[nodeId]) {
              node.jewel = this.items[this.state.jewels[nodeId]!]
            }
          }
        } else if (notableIndices.includes(index)) {
          const notable = jewelData.notables[notablePos++]
          node.skill_id = notable
          node.skill = this.skills[notable]
        }
      }

      nodes[proxy].connections.push({ id, radius: 0 })
      const nodeIndices = indices.map((idx) => (idx ? baseNode + idx : proxy))
      nodeIndices.push(proxy)
      for (let i = 0; i < indices.length; ++i) {
        nodes[nodeIndices[i]].connections.push({
          id: nodeIndices[i + 1],
          radius: 0
        })
        nodes[nodeIndices[i + 1]].connections.push({
          id: nodeIndices[i],
          radius: 0
        })
      }

      if (jewelData.mastery && this.skills[jewelData.mastery]) {
        nodes[baseNode + 15] = {
          group: clusterGroup,
          skill_id: jewelData.mastery,
          skill: this.skills[jewelData.mastery],
          x: clusterGroup.x,
          y: clusterGroup.y,
          state: 0,
          weaponSet: 0,
          active: false,
          radius: 0,
          arc: 0,
          connections: [],
          immutable: true
        }
      }

      return true
    }

    for (const id of Object.keys(this.state.jewels).map(Number)) {
      if (id >= clusterNodeBase) continue
      const proxy = proxyFor(id)
      if (proxy != null) createCluster(id, proxy)
    }

    return { nodes, groups }
  }

  @cachedMethod
  group(id: number) {
    const group = this.tree.groups[id]
    if (!group) return
    // Cluster groups are all built on top of existing groups
    if (group.proxy) return this.clusterNodes().groups[id]
    if (!this.useAscendancy(group.ascendancy)) return
    const result: GroupData = {
      bg: group.bg,
      proxy: group.proxy,
      x: group.x,
      y: group.y,
      ascendancy: group.ascendancy,
      nodes: group.nodes
    }
    const ascendancyRoot =
      group.ascendancy &&
      this.tree.nodes[this.tree.ascendancyRoot[group.ascendancy]]
    if (ascendancyRoot && 'charClass' in this.state) {
      let cx = 0
      let cy = 0
      if (ascendancyRoot.radius !== 9) {
        const arc =
          ((getOrbitAngle(
            skillsPerOrbit[ascendancyRoot.radius],
            ascendancyRoot.position
          ) -
            90) *
            Math.PI) /
          180
        const r = orbitRadii[9] - orbitRadii[ascendancyRoot.radius]
        cx += Math.cos(arc) * r
        cy += Math.sin(arc) * r
      }
      const rootGroup = this.tree.groups[ascendancyRoot.parent]
      result.x += cx - rootGroup.x
      result.y += cy - rootGroup.y
    }
    return result
  }

  @cachedMethod
  private keystoneStats() {
    const stats: Record<string, number> = {}
    for (const id of this.activeNodes()) {
      const node = this.node(id)!
      if (node.skill.is_keystone) {
        statsExtend(stats, node.skill.stats)
      } else if (node.skill.is_wormhole) {
        statsExtend(stats, {
          wormhole_count: 1
        })
      }
    }
    return stats
  }

  @cachedMethod
  totalStats() {
    const stats: Record<string, number> = {}
    const auraStats: Record<string, number> = {}
    const keystones: number[] = []
    let notables = 0
    for (const id of this.activeNodes()) {
      const skill = this.nodeSkill(id)!
      if (skill.disabled) continue
      if (!skill.is_keystone) {
        statsExtend(stats, skill.stats)
        if (skill.aura) statsExtend(auraStats, skill.aura.stats)
      } else {
        keystones.push(id)
      }
      if (skill.is_notable) {
        notables += 1
      }
    }
    if (this.atlas) {
      const bonus =
        this.keystoneStats()[
          'map_pack_size_+%_per_atlas_notable_passive_allocated'
        ]
      if (bonus) {
        statsExtend(stats, {
          'map_pack_size_+%': bonus * notables
        })
      }
    }
    return { stats, auraStats, keystones }
  }

  @cachedMethod
  activeMasteries(stat: string) {
    const indices: number[] = []
    for (const id of this.activeNodes()) {
      const node = this.node(id)!
      if (
        node.skill.mastery?.count_stat === stat &&
        node.masteryIndex != null
      ) {
        indices.push(node.masteryIndex)
      }
    }
    return indices
  }

  @cachedMethod
  private radiusJewels() {
    const jewels: RadiusJewel[] = []
    for (const [nid, itemId] of Object.entries(this.state.jewels)) {
      const item = itemId && this.items[itemId]
      if (!item) continue
      const id = Number(nid)
      const state = (this.state.nodes[id] ?? 0) & NodeState.StateMask
      if (
        state === NodeState.Active ||
        state === NodeState.Remove ||
        state === NodeState.AddHistory
      ) {
        const radius = radiusJewel(this, id)
        if (radius) jewels.push(radius)
      }
    }
    return jewels
  }

  @cachedMethod
  node(id: number) {
    if (id > clusterNodeBase) return this.clusterNodes().nodes[id]
    const node = this.tree.nodes[id]
    if (!node) return
    const group = this.group(node.parent)
    if (!group) return
    if (group.proxy) return this.clusterNodes().nodes[id]
    const skill = 'skill_id' in node ? this.skills[node.skill_id] : node
    if (!this.useAscendancy(skill.ascendancy)) return
    if (skill.is_anointment_only && !this.state.showHiddenNodes) return
    const arc =
      ((getOrbitAngle(skillsPerOrbit[node.radius], node.position) - 90) *
        Math.PI) /
      180
    const r = orbitRadii[node.radius]

    let state = (this.state.nodes[id] ?? 0) & NodeState.StateMask
    let immutable = false
    if ('atlas' in this.state) {
      if (this.tree.root_passives.includes(id)) {
        state = NodeState.Active
        immutable = true
      }
    } else if (skill.starting_node) {
      immutable = true
      state = skill.starting_node.includes(this.state.charClass)
        ? NodeState.Active
        : NodeState.None
    } else if (skill.is_ascendancy_root) {
      immutable = true
      state =
        skill.ascendancy === this.state.ascendancy
          ? NodeState.Active
          : NodeState.None
    } else if (skill.is_just_icon) {
      immutable = true
    }

    const result: NodeData = {
      group,
      skill_id:
        'skill_id' in node ? node.skill_id : node.ref_id ?? id.toString(),
      skill,
      x: group.x + r * Math.cos(arc),
      y: group.y + r * Math.sin(arc),
      radius: r,
      arc,
      state,
      weaponSet: (this.state.nodes[id] ?? 0) >> NodeState.WeaponSetShift,
      active:
        state === NodeState.Active ||
        state === NodeState.Remove ||
        state === NodeState.AddHistory,
      connections: node.connections,
      immutable
    }
    if (skill.mastery && result.active) {
      result.masteryIndex = this.state.masteries[id]
    }
    if (skill.is_jewel_socket && this.state.jewels[id]) {
      result.jewel = this.items[this.state.jewels[id]!]
    }
    return result
  }

  @cachedMethod
  nodeSkill(id: number) {
    const node = this.node(id)
    if (!node) return

    if (this.atlas) {
      if (!node.skill.is_keystone) {
        const keystones = this.keystoneStats()
        if (node.skill.is_notable) {
          if (keystones['display_notable_atlas_passives_grant_nothing']) {
            return { ...node.skill, stats: {}, aura: undefined }
          }
        } else if (!node.skill.is_keystone) {
          if (keystones['display_small_atlas_passives_grant_nothing']) {
            return { ...node.skill, stats: {}, aura: undefined }
          }
          const bonus = keystones['small_atlas_passive_effect_+%']
          if (bonus) {
            return {
              ...node.skill,
              stats: statsScale(node.skill.stats, 1 + 0.01 * bonus)
            }
          }
        }
      }
    } else if (
      !node.immutable &&
      !node.skill.is_jewel_socket &&
      !node.skill.mastery &&
      !node.clusterNode
    ) {
      let skill = node.skill
      if (skill.stats.display_passive_attribute_text && node.active) {
        const index = this.state.attributes[id]
        if (index != null) skill = this.skills[attributeSkills[index]] ?? skill
      }
      const jewels = this.radiusJewels()
        .filter((radius) => radius.contains(node))
        .map((radius) => radius.jewel)
      return mutateTreeSkill(id, node.active, skill, node.skill, jewels)
    } else if (node.skill.is_jewel_socket && node.jewel && node.active) {
      return {
        ...node.skill,
        stats: calculateJewelStats(this, id)
      }
    }

    return node.skill
  }

  *startNodes() {
    if ('atlas' in this.state) {
      yield* this.tree.root_passives
    } else {
      if (!this.state.ascendancyOnly) {
        yield this.tree.characterRoot[this.state.charClass]
      }
      if (this.state.ascendancy) {
        yield this.tree.ascendancyRoot[this.state.ascendancy]
      }
    }
  }

  *allNodes() {
    for (const id of Object.keys(this.tree.nodes).map(Number)) {
      if (this.node(id)) yield id
    }
    for (const id of Object.keys(this.clusterNodes().nodes)) {
      yield Number(id)
    }
  }
  *activeNodes() {
    yield* this.startNodes()
    for (const [id, state] of Object.entries(this.state.nodes)) {
      const value = (state ?? 0) & NodeState.StateMask
      if (
        state === NodeState.Active ||
        state === NodeState.Remove ||
        state === NodeState.AddHistory
      ) {
        const nid = Number(id)
        if (this.node(nid)) yield nid
      }
    }
  }

  *nodesInRadius(radius: RadiusJewel) {
    for (const id of Object.keys(this.tree.nodes).map(Number)) {
      if (id === radius.id) continue
      const node = this.node(id)
      if (node && radius.contains(node)) yield id
    }
  }
}
