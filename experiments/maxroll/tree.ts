import Data from '@/data'
import Trans from '@/data/trans'
import {
  PassiveSkillDefinition,
  PassiveTreeLegacyNode,
  PassiveTreeNode
} from '@/data/types'
import { StoreState } from '@/store'
import {
  AtlasEmbed,
  PassivesEmbed,
  TreeAddRemoveAction,
  TreeHistoryStep,
  TreeNodeAction
} from '@/types/embed'
import { Item } from '@/types/item'
import { ItemId, PlayerClass } from '@/types/profile'
import { searchRegex } from '@/utils'
import { NodeData, TreeDataProvider } from './treeData'

export function nodeData(
  node: PassiveTreeNode | PassiveTreeLegacyNode,
  skills: Record<string, PassiveSkillDefinition>
) {
  return 'skill_id' in node ? skills[node.skill_id] : node
}

export type NodeCount = {
  normal: number
  weaponSet: number[]
  ascendancy: number
  subTrees: Record<string, number>
}

export const atlasNodeLimit: NodeCount = {
  normal: 30,
  weaponSet: [0, 0, 0],
  ascendancy: 0,
  subTrees: {
    Breach: 8,
    Expedition: 8,
    Delirium: 8,
    Ritual: 8,
    Boss: 10,
    PinnacleBoss: 6
  }
}
export const passiveNodeLimit: NodeCount = {
  normal: 123,
  weaponSet: [24, 24, 0],
  ascendancy: 8,
  subTrees: {}
}

export type TreeProperties = {
  version: number
  readOnly?: boolean
  showHiddenNodes?: boolean
  hideBackground?: boolean
  limit?: NodeCount
} & (
  | {
      charClass: PlayerClass
      ascendancy?: string
      ascendancyOnly?: boolean
    }
  | {
      atlas: true
    }
)

export type TreeEditingState = {
  history: TreeHistoryStep[]
  activeSet: number
  position: number
  startPosition?: number
  masteries: Record<number, number>
  jewels: Record<number, ItemId>
  attributes: Record<number, number>
  hover?: number
  recording?: boolean
  editHistory?: boolean
  search?: string
  focus?: number[]
}

export const enum NodeState {
  None = 0,
  Active = 1,
  Add = 2,
  Remove = 3,
  AddHistory = 4,
  RemoveHistory = 5,
  StateMask = 0x0f,
  WeaponSetShift = 4
}

type TreeViewStateBase = {
  version: number
  showHiddenNodes?: boolean
  hideBackground?: boolean
  nodes: {
    [id in number]?: NodeState
  }
  activeSet: number
  masteries: Record<number, number>
  jewels: Record<number, ItemId>
  attributes: Record<number, number>
  ascendancyOnly?: boolean
  hover?: number
  highlight?: number[]
  limit?: NodeCount
  count: NodeCount
}

export type TreeViewStatePassives = TreeViewStateBase & {
  charClass: PlayerClass
  ascendancy?: string
}

export type TreeViewStateAtlas = TreeViewStateBase & {
  atlas: true
  league?: Boolean
}

export type TreeViewState = TreeViewStatePassives | TreeViewStateAtlas

export function checkLimit(count: NodeCount, limit: NodeCount) {
  return (
    count.normal <= limit.normal &&
    count.ascendancy <= limit.ascendancy &&
    count.weaponSet.every((w, i) => w <= limit.weaponSet[i]) &&
    Object.keys(count.subTrees).every(
      (id) => count.subTrees[id] <= (limit.subTrees[id] ?? 0)
    )
  )
}

function treeConnectedNodes(
  data: TreeDataProvider,
  visited: Map<number, number>,
  queue: number[],
  set: number,
  ...exclude: number[]
) {
  for (const id of queue) {
    visited.set(id, set)
  }
  for (let pos = 0; pos < queue.length; ++pos) {
    const id = queue[pos]
    const node = data.node(id)!
    for (const { id: nid } of node.connections) {
      if (visited.has(nid) || exclude.includes(nid)) continue
      const next = data.node(nid)
      if (!next || !next.state) continue
      if (node.weaponSet && node.weaponSet !== next.weaponSet) continue
      if (node.skill.is_just_icon) continue // no going through those
      if (next.skill.ascendancy && !node.skill.ascendancy) continue // can't go backwards into scion ascendancy
      visited.set(nid, next.weaponSet)
      queue.push(nid)
    }
  }
  return visited
}

export class RadiusJewel {
  private r1: number
  private r2: number
  constructor(
    public id: number,
    public jewel: Item,
    public x: number,
    public y: number,
    public minRadius: number | undefined,
    public maxRadius: number,
    public treeVersion: number | undefined
  ) {
    this.r1 = minRadius ? minRadius * minRadius : 0
    this.r2 = maxRadius * maxRadius
  }
  contains(node: NodeData) {
    if (
      node.clusterNode ||
      node.skill.is_jewel_socket ||
      node.immutable ||
      node.skill.is_multiple_choice_option ||
      node.skill.ascendancy
    )
      return false
    const dx = node.x - this.x
    const dy = node.y - this.y
    const d2 = dx * dx + dy * dy
    return d2 >= this.r1 && d2 <= this.r2
  }
}

export function radiusJewel(
  data: TreeDataProvider,
  id: number,
  leapOnly?: boolean
) {
  const node = data.node(id)
  if (!node || node.clusterNode || !node.jewel) return
  if (!node.jewel.unique) return

  const {
    local_jewel_variable_ring_radius_value: ringRadius,
    local_unique_jewel_disconnected_passives_can_be_allocated_around_keystone_hash:
      escapeNode,
    local_unique_jewel_nearby_disconnected_passives_can_be_allocated: leap,
    local_unique_jewel_alternate_tree_version: treeVersion
  } = node.jewel.stats.unique ?? {}

  const radius = Data.uniques[node.jewel.unique]?.jewel_radius

  if (escapeNode && radius) {
    const onode = data.node(escapeNode)
    if (onode) {
      return new RadiusJewel(
        id,
        node.jewel,
        onode.x,
        onode.y,
        144,
        Data.jewelRadii[radius - 1].radius,
        treeVersion
      )
    }
    return
  }

  if (leapOnly && !leap) return

  if (ringRadius) {
    return new RadiusJewel(
      id,
      node.jewel,
      node.x,
      node.y,
      Data.jewelRadii[ringRadius - 1].ringInner,
      Data.jewelRadii[ringRadius - 1].ringOuter,
      treeVersion
    )
  } else if (radius) {
    return new RadiusJewel(
      id,
      node.jewel,
      node.x,
      node.y,
      undefined,
      Data.jewelRadii[radius - 1].radius,
      treeVersion
    )
  }
}

function treeLeapJewels(data: TreeDataProvider) {
  const leaps: RadiusJewel[] = []
  for (const id of Object.keys(data.state.jewels).map(Number)) {
    const leap = radiusJewel(data, id, true)
    if (leap) leaps.push(leap)
  }
  return leaps
}

function hasLeap(
  leaps: RadiusJewel[],
  connected: Map<number, number>,
  node: NodeData,
  set: number
) {
  for (const leap of leaps) {
    const con = connected.get(leap.id)
    if (con == null) continue
    if (con && con !== set) continue
    if (leap.contains(node)) return true
  }
  return false
}

function treeAllocatePath(data: TreeDataProvider, target: number) {
  const targetNode = data.node(target)
  // any non-empty state can't be allocated
  if (!targetNode || targetNode.state) return

  // ascendancy nodes can't go into alt sets
  const activeSet = targetNode.skill.ascendancy ? 0 : data.state.activeSet

  const connected = new Map<number, number>()
  treeConnectedNodes(data, connected, [...data.startNodes()], 0)
  if (
    !targetNode.skill.is_jewel_socket &&
    !targetNode.skill.is_multiple_choice_option &&
    !targetNode.skill.ascendancy
  ) {
    if (hasLeap(treeLeapJewels(data), connected, targetNode, activeSet)) {
      return { add: [target] }
    }
  }

  const visited = new Map<number, number>()
  visited.set(target, 0)
  const queue: number[] = [target]
  for (let pos = 0; pos < queue.length; ++pos) {
    let id = queue[pos]
    const node = data.node(id)!
    if (connected.has(id)) {
      const add: TreeNodeAction[] = []
      const remove: number[] = []
      while (id) {
        const node = data.node(id)!
        if (!node.state) add.push(activeSet ? { id, set: activeSet } : id)
        const next = visited.get(id)!
        if (
          next &&
          node.skill.is_multiple_choice &&
          data.node(next)!.skill.is_multiple_choice_option
        ) {
          // deallocate other options
          for (const { id: oid } of node.connections) {
            if (oid === next) continue
            const onode = data.node(oid)
            if (onode?.skill.is_multiple_choice_option && onode.state) {
              remove.push(oid)
            }
          }
        }
        id = next
      }
      return { add, remove: remove.length ? remove : undefined }
    }
    for (const { id: nid } of node.connections) {
      if (visited.has(nid)) continue
      const next = data.node(nid)
      if (!next) continue
      if (
        (next.skill.is_ascendancy_root || next.skill.starting_node) &&
        !next.state
      )
        continue // can't path through inactive root
      if (node.skill.ascendancy !== next.skill.ascendancy && !next.state)
        continue // can't automatically path through scion ascendancy nodes
      if (next.skill.is_just_icon) continue // no going through those
      if (!next.skill.ascendancy && node.skill.ascendancy) continue // can't go backwards into scion ascendancy
      if (
        next.skill.is_multiple_choice &&
        !node.skill.is_multiple_choice_option &&
        !next.state
      )
        continue // can't path through unallocated multiple choice nodes (need to select an option first)
      if (next.weaponSet && next.weaponSet !== activeSet) continue // can't connect to nodes allocated in another weapon set
      visited.set(nid, id)
      queue.push(nid)
    }
  }
}

function treeDisconnectedNodes(data: TreeDataProvider, ...remove: number[]) {
  const connected = new Map<number, number>()
  treeConnectedNodes(data, connected, [...data.startNodes()], 0, ...remove)
  const leaps = treeLeapJewels(data)
  return Object.keys(data.state.nodes)
    .map(Number)
    .filter((id) => {
      if (!data.state.nodes[id] || connected.has(id)) return false
      if (remove.includes(id)) return true
      if (!leaps.length) return true
      const node = data.node(id)
      return (
        !node ||
        !hasLeap(
          leaps,
          connected,
          node,
          data.state.nodes[id]! >> NodeState.WeaponSetShift
        )
      )
    })
}

function treeToggleAction(
  data: TreeDataProvider,
  id: number
): TreeAddRemoveAction | undefined {
  // We check state.nodes even though it has 'wrong' states for starting node,
  // because it will result in a call to treeAllocatePath, which correctly returns
  // nothing for already active nodes.
  if (data.state.nodes[id]) {
    const node = data.node(id)
    if (!node) return
    const removing = [id]
    if (node.skill.is_multiple_choice_option) {
      // Also remove parent
      removing.push(node.connections[0].id)
    }
    return {
      remove: treeDisconnectedNodes(data, ...removing)
    }
  } else {
    return treeAllocatePath(data, id)
  }
}

function historyNodes(tree: TreeProperties, steps: TreeHistoryStep[]) {
  const nodes = new Map<number, number>()
  for (const step of steps) {
    if (typeof step === 'number') {
      nodes.set(step, 0)
    } else if ('id' in step) {
      nodes.set(step.id, step.set)
    } else {
      if (step.add) {
        for (const id of step.add) {
          if (typeof id === 'number') {
            nodes.set(id, 0)
          } else {
            nodes.set(id.id, id.set)
          }
        }
      }
      if (step.remove) {
        for (const id of step.remove) nodes.delete(id)
      }
    }
  }

  // We expect that multiple choice nodes do not appear on cluster jewels,
  // so we do not need to construct a TreeDataProvider.
  const treeData = Data.trees[tree.version]!
  const treeNodes = (
    'atlas' in tree ? treeData.atlas_passive_tree : treeData.passive_tree
  ).nodes
  const treeSkills =
    'passive_skills' in treeData ? treeData.passive_skills : Data.passive_skills
  for (const [id, set] of nodes) {
    const node = treeNodes[id]
    if (!node) continue
    const skill = nodeData(node, treeSkills)
    if (skill.is_multiple_choice_option) {
      // we assume that options only have one connection
      nodes.set(node.connections[0].id, set)
    }
  }

  return nodes
}

function countNodes(tree: TreeProperties, nodes: Map<number, number>) {
  // Ascendancy and multiple choice nodes only appear in the base tree,
  // so we do not need to construct a TreeDataProvider.
  const treeData = Data.trees[tree.version]!
  const treeNodes = (
    'atlas' in tree ? treeData.atlas_passive_tree : treeData.passive_tree
  ).nodes
  const treeSkills =
    'passive_skills' in treeData ? treeData.passive_skills : Data.passive_skills
  let normal = 0
  let weaponSet = [0, 0, 0]
  let ascendancy = 0
  let points = 0
  let weaponPoints = 0
  const subTrees: Record<string, number> = {}
  for (const [id, set] of nodes) {
    const node = treeNodes[id]
    if (!node) {
      // Must be a cluster jewel node
      normal += 1
      if (set) weaponSet[set - 1] += 1
      continue
    }
    const skill = nodeData(node, treeSkills)
    if (skill.is_multiple_choice_option) continue
    if (skill.ascendancy) {
      ascendancy += 1
    } else if (skill.atlas_sub_tree) {
      subTrees[skill.atlas_sub_tree] = (subTrees[skill.atlas_sub_tree] ?? 0) + 1
    } else {
      normal += 1
      if (set) weaponSet[set - 1] += 1
    }
    if (skill.skill_points) {
      points += skill.skill_points
    }
    if (skill.weapon_points) {
      weaponPoints += skill.weapon_points
    }
  }
  return { normal, weaponSet, ascendancy, points, weaponPoints, subTrees }
}

export function treeNodeCount(tree: TreeProperties, src: TreeEditingState) {
  const position = Math.min(src.history.length, Math.max(0, src.position))
  const nodes = historyNodes(tree, src.history.slice(0, position))
  return countNodes(tree, nodes)
}

export function treeViewState(
  tree: TreeProperties,
  src: TreeEditingState,
  items: StoreState['items']
) {
  const position = Math.min(src.history.length, Math.max(0, src.position))
  const nodes = historyNodes(tree, src.history.slice(0, position))
  const count = countNodes(tree, nodes)

  const base: TreeViewStateBase = {
    version: tree.version,
    showHiddenNodes: tree.showHiddenNodes,
    hideBackground: tree.hideBackground,
    nodes: {},
    activeSet: src.activeSet,
    masteries: src.masteries,
    jewels: src.jewels,
    attributes: src.attributes,
    hover: src.hover,
    count: {
      normal: count.normal,
      weaponSet: count.weaponSet,
      ascendancy: count.ascendancy,
      subTrees: count.subTrees
    }
  }

  if (tree.limit) {
    base.limit = { ...tree.limit }
    base.limit.normal += count.points
    base.limit.weaponSet = tree.limit.weaponSet.map(
      (v) => v + count.weaponPoints
    )
  }

  const state: TreeViewState =
    'atlas' in tree
      ? {
          ...base,
          atlas: true
        }
      : {
          ...base,
          charClass: tree.charClass,
          ascendancy: tree.ascendancy,
          ascendancyOnly: tree.ascendancyOnly
        }

  if (state.hover && !tree.readOnly) {
    for (const [id, set] of nodes) {
      state.nodes[id] = NodeState.Active | (set << NodeState.WeaponSetShift)
    }

    const data = new TreeDataProvider(state, items)
    const path = treeToggleAction(data, state.hover)

    let valid = true
    if (tree.limit) {
      if (path?.add) {
        for (const id of path.add) {
          if (typeof id === 'number') {
            nodes.set(id, 0)
          } else {
            nodes.set(id.id, id.set)
          }
        }
      }
      if (path?.remove) {
        for (const id of path.remove) nodes.delete(id)
      }
      const count = countNodes(tree, nodes)
      valid = checkLimit(count, {
        ...tree.limit,
        normal: tree.limit.normal + count.points,
        weaponSet: tree.limit.weaponSet.map((w) => w + count.weaponPoints)
      })
    }

    if (valid) {
      if (path?.add) {
        for (const id of path.add) {
          if (typeof id === 'number') {
            state.nodes[id] = NodeState.Add
          } else {
            state.nodes[id.id] =
              NodeState.Add | (id.set << NodeState.WeaponSetShift)
          }
        }
      }
      if (path?.remove) {
        for (const id of path.remove) {
          state.nodes[id] =
            ((state.nodes[id] ?? 0) & ~NodeState.StateMask) | NodeState.Remove
        }
      }
    } else if (path?.add?.length) {
      for (const id of path.add) {
        if (typeof id === 'number') {
          state.nodes[id] = NodeState.RemoveHistory
        } else {
          state.nodes[id.id] =
            NodeState.RemoveHistory | (id.set << NodeState.WeaponSetShift)
        }
      }
    }

    if (!src.focus && !src.search?.length) {
      const radius = radiusJewel(data, state.hover)
      if (radius) {
        state.highlight = [...data.nodesInRadius(radius)]
      }
    }
  } else if (src.startPosition != null && src.startPosition < position) {
    const startPosition = Math.min(position, Math.max(0, src.startPosition))
    const startNodes = historyNodes(tree, src.history.slice(0, startPosition))
    for (const [id, set] of nodes) {
      state.nodes[id] =
        (startNodes.delete(id) ? NodeState.Active : NodeState.AddHistory) |
        (set << NodeState.WeaponSetShift)
    }
    for (const [id, set] of startNodes) {
      state.nodes[id] =
        NodeState.RemoveHistory | (set << NodeState.WeaponSetShift)
    }
  } else {
    for (const [id, set] of nodes) {
      state.nodes[id] = NodeState.Active | (set << NodeState.WeaponSetShift)
    }
  }

  if (src.focus) {
    state.highlight = src.focus
  } else if (src.search?.length) {
    const search = searchRegex(src.search, true)
    if (!search) return state

    const data = new TreeDataProvider(state, items)
    state.highlight = [...data.allNodes()].filter((id) => {
      const node = data.nodeSkill(id)!

      if (node.is_ascendancy_root || node.starting_node || node.is_just_icon)
        return false

      if (node.name.match(search)) return true

      const lines = Trans[data.atlas ? 'atlas' : 'passive_skill'].formatStats(
        node.stats
      )
      if (lines.some((line) => line.match(search))) return true

      if (node.aura) {
        const lines = Trans.passive_skill_aura.formatStats(node.aura.stats)
        if (lines.some((line) => line.match(search))) return true
      }
      return false
    })
  }

  return state
}

export function lightState(state: TreeEditingState) {
  return {
    ...state,
    // Hide hover and start position to avoid highlighting nodes
    startPosition: undefined,
    hover: undefined,
    // Hide search to avoid unnecessary work
    focus: undefined,
    search: undefined
  }
}

export function treeValidateHistory(
  tree: TreeProperties,
  src: TreeEditingState,
  steps: TreeHistoryStep[],
  items: StoreState['items']
) {
  const view = treeViewState(tree, lightState(src), items)
  const data = new TreeDataProvider(view, items)

  const connected = new Map<number, number>()
  treeConnectedNodes(data, connected, [...data.startNodes()], 0)
  const leaps = treeLeapJewels(data)

  const count = {
    ...view.count,
    weaponSet: view.count.weaponSet.slice(),
    subTrees: { ...view.count.subTrees }
  }

  function addNode(id: number, set: number, check: boolean) {
    const node = data.node(id)
    if (!node || node.immutable || node.state) return false
    let connect = true
    if (node.skill.is_multiple_choice_option) {
      // We have to add parent first
      // If it fails due to already being active, we must've been
      // trying to allocate two options at once.
      if (!addNode(node.connections[0].id, set, check)) return false
      // But now we don't need to check anything!
    } else if (check) {
      if (
        !node.connections.some(({ id: pid }) => {
          if (!connected.has(pid)) return false
          const prev = data.node(pid)
          if (!prev || !prev.state) return false
          if (prev.weaponSet && prev.weaponSet !== set) return false
          if (node.skill.ascendancy && !prev.skill.ascendancy) return false // can't go backwards into scion ascendancy
          return true
        })
      ) {
        if (!hasLeap(leaps, connected, node, set)) {
          return false
        }
        connect = false
      }
    }
    // Modify both to keep them in sync
    node.state = NodeState.Active
    node.weaponSet = set
    view.nodes[id] = NodeState.Active | (set << NodeState.WeaponSetShift)
    if (connect) {
      // Run the whole function in case we connected to another path
      treeConnectedNodes(data, connected, [id], set)
    }
    // Update count
    if (!node.skill.is_multiple_choice) {
      if (node.skill.ascendancy) {
        count.ascendancy += 1
      } else if (node.skill.atlas_sub_tree) {
        count.subTrees[node.skill.atlas_sub_tree] =
          (count.subTrees[node.skill.atlas_sub_tree] ?? 0) + 1
      } else {
        count.normal += 1
        if (set) count.weaponSet[set - 1] += 1
      }
    }
    // Update limits
    if (view.limit) {
      if (node.skill.skill_points) {
        view.limit.normal += node.skill.skill_points
      }
      if (node.skill.weapon_points) {
        for (let i = 0; i < 3; ++i) {
          view.limit.weaponSet[i] += node.skill.weapon_points
        }
      }
    }
    return true
  }
  function removeNode(id: number) {
    const node = data.node(id)
    if (!node || node.immutable || !node.state) return false
    if (node.skill.is_multiple_choice_option) {
      // We have to remove parent first
      // Don't check for success - we have to remove the node anyway
      removeNode(node.connections[0].id)
    }
    // Update count
    if (!node.skill.is_multiple_choice) {
      if (node.skill.ascendancy) {
        count.ascendancy -= 1
      } else if (node.skill.atlas_sub_tree) {
        count.subTrees[node.skill.atlas_sub_tree] =
          (count.subTrees[node.skill.atlas_sub_tree] ?? 0) - 1
      } else {
        count.normal -= 1
        if (node.weaponSet) count.weaponSet[node.weaponSet - 1] -= 1
      }
    }
    // Modify both to keep them in sync
    node.state = 0
    node.weaponSet = 0
    view.nodes[id] = 0
    // Update limits
    if (view.limit && node.skill.skill_points) {
      view.limit.normal -= node.skill.skill_points
    }
    if (view.limit && node.skill.weapon_points) {
      for (let i = 0; i < 3; ++i) {
        view.limit.weaponSet[i] -= node.skill.weapon_points
      }
    }
    return true
  }

  const result: TreeHistoryStep[] = []
  for (const step of steps) {
    if (typeof step === 'number') {
      // Try to add the node
      if (addNode(step, 0, true)) {
        if (view.limit && !checkLimit(count, view.limit)) return result
        result.push(step)
      }
    } else if ('id' in step) {
      // Try to add the node
      if (addNode(step.id, step.set, true)) {
        if (view.limit && !checkLimit(count, view.limit)) return result
        result.push(step)
      }
    } else {
      const remove: number[] = []
      // First, remove all nodes to be removed
      if (step.remove) {
        for (const id of step.remove) {
          if (removeNode(id)) {
            remove.push(id)
          }
        }
      }
      // Then, add nodes without checking connections
      const add = new Map<number, number>()
      if (step.add) {
        for (const id of step.add) {
          if (typeof id === 'number') {
            if (addNode(id, 0, false)) {
              add.set(id, 0)
            }
          } else {
            if (addNode(id.id, id.set, false)) {
              add.set(id.id, id.set)
            }
          }
        }
      }
      // Now, find disconnected nodes and remove them
      connected.clear()
      treeConnectedNodes(data, connected, [...data.startNodes()], 0)
      for (const id of Object.keys(data.state.nodes).map(Number)) {
        if (!data.state.nodes[id] || connected.has(id)) continue
        const node = data.node(id)
        if (!node) continue // What happened?
        if (
          hasLeap(
            leaps,
            connected,
            node,
            data.state.nodes[id]! >> NodeState.WeaponSetShift
          )
        )
          continue

        node.state = 0
        node.weaponSet = 0
        view.nodes[id] = 0
        if (!add.delete(id)) {
          remove.push(id)
        }
      }
      if (view.limit && !checkLimit(count, view.limit)) return result
      if (remove.length || add.size) {
        const res: TreeAddRemoveAction = {}
        if (add.size)
          res.add = [...add].map(([id, set]) => (set ? { id, set } : id))
        if (remove.length) res.remove = remove
        result.push(res)
      }
    }
  }
  return result
}

function sameHistory(lhs: TreeHistoryStep[], rhs: TreeHistoryStep[]) {
  if (lhs.length !== rhs.length) return false
  return lhs.every((lv, index) => {
    const rv = rhs[index]
    if (lv === rv) return true
    if (typeof lv === 'object' && typeof rv === 'object') {
      if ('id' in lv) {
        if (!('id' in rv)) return false
        return lv.id === rv.id && lv.set === rv.set
      } else {
        if ('id' in rv) return false
        if ((lv.add?.length ?? 0) !== (rv.add?.length ?? 0)) return false
        if (
          lv.add?.length &&
          !lv.add.every((lid, idx) => {
            const rid = rv.add![idx]
            if (lid === rid) return true
            if (typeof lid === 'object' && typeof rid === 'object') {
              return lid.id === rid.id && lid.set === rid.set
            }
          })
        )
          return false
        if ((lv.remove?.length ?? 0) !== (rv.remove?.length ?? 0)) return false
        if (
          lv.remove?.length &&
          lv.remove.some((lid, idx) => lid !== rv.remove![idx])
        )
          return false
        return true
      }
    }
    return false
  })
}

function treeModifyHistory(
  tree: TreeProperties,
  src: TreeEditingState,
  add: TreeNodeAction[] | undefined,
  remove: number[] | undefined,
  items: StoreState['items']
) {
  const position = Math.min(src.history.length, Math.max(0, src.position))
  const history = src.history.slice(0, position)
  const remaining = src.history.slice(position)
  if (src.recording) {
    const top = history[history.length - 1]
    if (top && typeof top === 'object' && !('id' in top)) {
      // Current history step is a recording step. Augment it with new nodes.
      let topAdd = top.add?.slice() ?? []
      let topRemove = top.remove?.slice() ?? []
      if (add) {
        const adding = new Map(
          add.map((a) => (typeof a === 'number' ? [a, 0] : [a.id, a.set]))
        )
        topRemove = topRemove.filter((id) => !adding.delete(id))
        topAdd.push(...[...adding].map(([id, set]) => (set ? { id, set } : id)))
      }
      if (remove) {
        const removing = new Set(remove)
        topAdd = topAdd.filter(
          (id) => !removing.delete(typeof id === 'number' ? id : id.id)
        )
        topRemove.push(...removing)
      }
      const res: TreeAddRemoveAction = {}
      if (topAdd.length) res.add = topAdd
      if (topRemove.length) res.remove = topRemove
      history[history.length - 1] = res
    } else {
      // Insert a recording step at current position
      const res: TreeAddRemoveAction = {}
      if (add?.length) res.add = add
      if (remove?.length) res.remove = remove
      history.push(res)
    }
  } else if (add?.length && remove?.length) {
    // Special case: if both add and remove are specified, then we must be
    // replacing a multiple choice option. If the node was allocated in the previous
    // state, then we replace it. Otherwise, we add a respec step.
    const top = history[history.length - 1]
    if (add.length === 1 && remove.length === 1 && top === remove[0]) {
      history[history.length - 1] = add[0]
    } else {
      history.push({ add, remove })
    }
  } else {
    if (remove) {
      for (const id of remove) {
        // Find last step where the node was added
        for (let pos = history.length - 1; pos >= 0; --pos) {
          const step = history[pos]
          if (step === id) {
            history.splice(pos, 1)
            break
          } else if (typeof step === 'object' && 'id' in step) {
            if (step.id === id) {
              history.splice(pos, 1)
              break
            }
          } else if (typeof step === 'object' && step.add) {
            const index = step.add.findIndex((v) =>
              typeof v === 'number' ? v === id : v.id === id
            )
            if (index >= 0) {
              const newStep: TreeAddRemoveAction = { ...step }
              newStep.add = step.add.slice()
              newStep.add.splice(index, 1)
              if (!newStep.add.length) delete newStep.add
              if (!newStep.add && !newStep.remove?.length) {
                // Nothing left in the step, remove it.
                history.splice(pos, 1)
              } else {
                history[pos] = newStep
              }
              break
            }
          }
        }
      }
    }
    if (add) {
      // Simply add the nodes to the end of history
      history.push(...add)
    }
  }
  const result: TreeEditingState = {
    ...src,
    history: [],
    position: 0,
    startPosition: undefined
  }
  // Check that we didn't break the progression by removing something
  const checkHistory = treeValidateHistory(tree, result, history, items)
  if (sameHistory(history, checkHistory)) {
    // All good, replace history
    result.history = history
  } else {
    // Check if we exceeded the limit
    if (tree.limit) {
      const view = treeViewState(
        tree,
        lightState({
          ...result,
          history,
          position: history.length
        }),
        items
      )
      if (!checkLimit(view.count, view.limit!)) {
        // Simply return the original state
        return src
      }
    }
    // Revert history and add new respec step
    result.history = src.history.slice(0, position)
    const res: TreeAddRemoveAction = {}
    if (add?.length) res.add = add
    if (remove?.length) res.remove = remove
    result.history.push(res)
  }
  result.position = result.history.length
  if (remaining.length && src.editHistory) {
    // Check remaining history for consistency
    result.history.push(...treeValidateHistory(tree, result, remaining, items))
  }
  return result
}

export function treeToggleNode(
  tree: TreeProperties,
  src: TreeEditingState,
  id: number,
  items: StoreState['items'],
  attribute?: number
) {
  const view = treeViewState(tree, lightState(src), items)
  const data = new TreeDataProvider(view, items)
  const action = treeToggleAction(data, id)
  if (!action) return src
  let { add, remove } = action
  if (!add?.length && !remove?.length) return src

  // First, filter nodes to make sure we're actually adding and removing
  // (sanity check). Also exclude multiple choice parents, as we do not
  // keep them in the state.
  add = add?.filter((a) => {
    const id = typeof a === 'number' ? a : a.id
    const node = data.node(id)
    if (!node) return false
    return !node.state && !node.skill.is_multiple_choice
  })
  remove = remove?.filter((id) => {
    const node = data.node(id)
    if (!node) return true
    return node.state && !node.skill.is_multiple_choice
  })

  src = treeModifyHistory(tree, src, add, remove, items)
  if (add?.length && attribute != null) {
    src = { ...src }
    src.attributes = { ...src.attributes }
    for (const a of add) {
      const id = typeof a === 'number' ? a : a.id
      src.attributes[id] = attribute
    }
  }

  return src
}

const isRespec = (step: TreeHistoryStep): step is TreeAddRemoveAction =>
  typeof step === 'object' && !('id' in step)

function isConnected(
  data: TreeDataProvider,
  step: TreeHistoryStep,
  prev: TreeHistoryStep
) {
  if (isRespec(step) || isRespec(prev)) return false
  const node = data.node(typeof step === 'number' ? step : step.id)
  if (!node) return false
  if (typeof prev === 'number') {
    return node.connections.find((n) => n.id === prev)
  }
  if (prev.set !== (typeof step === 'number' ? 0 : step.set)) return false
  return node.connections.find((n) => n.id === prev.id)
}

export function historyNextPosition(
  tree: TreeProperties,
  state: TreeEditingState,
  position: number,
  items: StoreState['items']
) {
  if (position <= 0 || isRespec(state.history[position - 1])) return position
  const data = new TreeDataProvider(
    treeViewState(tree, lightState(state), items),
    items
  )
  while (position < state.history.length) {
    if (
      !isConnected(data, state.history[position], state.history[position - 1])
    )
      break
    position += 1
  }
  return position
}

export function historyPrevPosition(
  tree: TreeProperties,
  state: TreeEditingState,
  position: number,
  items: StoreState['items']
) {
  if (position <= 0 || isRespec(state.history[position - 1])) return position
  const data = new TreeDataProvider(
    treeViewState(tree, lightState(state), items),
    items
  )
  while (position >= 2) {
    if (
      !isConnected(
        data,
        state.history[position - 2],
        state.history[position - 1]
      )
    )
      break
    position -= 1
  }
  if (position === 1) return 0
  return position
}

export function treeToggleRecording(
  tree: TreeProperties,
  state: TreeEditingState
) {
  if (tree.readOnly) return state
  if (state.recording) {
    return {
      ...state,
      recording: false
    }
  } else {
    const position = Math.max(0, Math.min(state.history.length, state.position))
    const history = state.history.slice()
    history.splice(position, 0, {})
    return {
      ...state,
      history,
      position: position + 1,
      startPosition: undefined,
      recording: true
    }
  }
}

export function validateTreeEmbed(
  embed: PassivesEmbed,
  items: StoreState['items']
): PassivesEmbed
export function validateTreeEmbed(
  embed: AtlasEmbed,
  items: StoreState['items']
): AtlasEmbed
export function validateTreeEmbed(
  embed: PassivesEmbed | AtlasEmbed,
  items: StoreState['items']
): PassivesEmbed | AtlasEmbed
export function validateTreeEmbed(
  embed: PassivesEmbed | AtlasEmbed,
  items: StoreState['items']
) {
  const props: TreeProperties =
    embed.type === 'atlas'
      ? {
          version: embed.version,
          atlas: true,
          limit: atlasNodeLimit
        }
      : {
          version: embed.version,
          charClass: embed.charClass,
          ascendancy: embed.ascendancy,
          limit: passiveNodeLimit
        }

  const variants = embed.variants.map((variant) => {
    const state: TreeEditingState = {
      history: [],
      position: 0,
      activeSet: 0,
      masteries: variant.masteries,
      jewels: variant.jewels,
      attributes: variant.attributes
    }
    const history = treeValidateHistory(props, state, variant.history, items)
    return {
      ...variant,
      history
    }
  })
  if (
    variants.some((v, i) => !sameHistory(v.history, embed.variants[i].history))
  ) {
    return { ...embed, variants }
  }
  return embed
}

function treePointsFromLevel(level: number, bandits?: string) {
  let points = Math.max(0, level - 1)
  if (level >= 8) points += 1 // The Dweller of the Deep
  if (level >= 11) points += 1 // The Marooned Mariner
  if (level >= 17 && bandits === 'all') points += 2 // Deal With The Bandits
  if (level >= 17) points += 1 // The Way Forward
  if (level >= 23) points += 1 // Victario's Secrets
  if (level >= 27) points += 1 // Piety's Pets
  if (level >= 31) points += 1 // An Indomitable Spirit
  if (level >= 37) points += 1 // In Service to Science
  if (level >= 40) points += 1 // Kitava's Torments
  if (level >= 42) points += 1 // The Father of War
  if (level >= 44) points += 1 // The Puppet Mistress
  if (level >= 44) points += 1 // The Cloven One
  if (level >= 48) points += 1 // The Master of a Million Faces
  if (level >= 48) points += 1 // Queen of Despair
  if (level >= 48) points += 1 // Kishara's Star
  if (level >= 52) points += 1 // Love is Dead
  if (level >= 52) points += 1 // The Gemling Legion
  if (level >= 60) points += 1 // Reflection of Terror
  if (level >= 60) points += 1 // Queen of the Sands
  if (level >= 60) points += 1 // The Ruler of Highgate
  if (level >= 61) points += 1 // Vilenta's Vengeance
  if (level >= 62) points += 2 // An End to Hunger
  return points
}

export function treeWithLevel(
  embed: PassivesEmbed,
  active: number | undefined,
  level: number | undefined,
  bandits?: string
) {
  embed = { ...embed, active: active ?? embed.active }
  if (!embed.variants[embed.active]) embed.active = 0
  const variant = embed.variants[embed.active]
  if (!variant || level == null) return embed

  const treeData = Data.trees[embed.version]!
  const treeNodes = treeData.passive_tree.nodes
  const treeSkills =
    'passive_skills' in treeData ? treeData.passive_skills : Data.passive_skills

  let points = 0
  let limit = treePointsFromLevel(level, bandits)
  embed.activePos = 0

  function countNode(id: number, factor = 1) {
    const node = treeNodes[id]
    const skill = nodeData(node, treeSkills)
    if (skill.is_multiple_choice) return
    if (skill.skill_points) limit += skill.skill_points * factor
    if (!skill.ascendancy) points += factor
  }

  const nodes = new Set<number>()
  for (const step of variant.history) {
    embed.activePos += 1

    if (isRespec(step)) {
      if (step.remove) {
        for (const id of step.remove) {
          if (nodes.delete(id)) countNode(id, -1)
        }
      }
      if (step.add) {
        for (const sub of step.add) {
          const id = typeof sub === 'number' ? sub : sub.id
          if (!nodes.has(id)) {
            nodes.add(id)
            countNode(id)
          }
        }
      }
    } else {
      const id = typeof step === 'number' ? step : step.id
      if (!nodes.has(id)) {
        nodes.add(id)
        countNode(id)
      }
    }

    if (points >= limit) break
  }

  return embed
}
