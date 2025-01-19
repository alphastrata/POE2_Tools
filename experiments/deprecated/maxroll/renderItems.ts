import { NodeState, TreeViewState } from '@/logic/tree'
import { TreeDataProvider } from '@/logic/treeData'
import { loadAtlasTree, renderAtlasTree } from './renderAtlas'
import { loadPassiveTree, renderPassiveTree } from './renderPassives'
import { Renderer } from './renderer'

export function frameColor(state: NodeState) {
  switch (state & NodeState.StateMask) {
    case NodeState.Add:
    case NodeState.AddHistory:
      return { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }
    case NodeState.Remove:
    case NodeState.RemoveHistory:
      return { r: 1.0, g: 0.125, b: 0.125, a: 1.0 }
  }
  switch (state >> NodeState.WeaponSetShift) {
    case 1:
      return { r: 0.0, g: 0.5, b: 0.0, a: 1.0 }
    case 2:
      return { r: 0.8, g: 0.3, b: 0.3, a: 1.0 }
    case 3:
      return { r: 0.8, g: 0.5, b: 0.0, a: 1.0 }
  }
}

export const InteractAscendancy = -1

export function lineState(node1: NodeState, node2: NodeState) {
  const set1 = node1 >> NodeState.WeaponSetShift
  const set2 = node2 >> NodeState.WeaponSetShift
  if (set1 && set2 && set1 !== set2) return NodeState.None
  const set = (set1 || set2) << NodeState.WeaponSetShift
  const state1 = node1 & NodeState.StateMask
  const state2 = node2 & NodeState.StateMask
  if (state1 === NodeState.Active && state2 === NodeState.Active) {
    return NodeState.Active | set
  } else if (
    (state1 === NodeState.Remove || state1 === NodeState.RemoveHistory) &&
    state2
  ) {
    return NodeState.Remove
  } else if (
    (state2 === NodeState.Remove || state2 === NodeState.RemoveHistory) &&
    state1
  ) {
    return NodeState.Remove
  } else if (
    (state1 === NodeState.Add || state1 === NodeState.AddHistory) &&
    state2
  ) {
    return NodeState.Add
  } else if (
    (state2 === NodeState.Add || state2 === NodeState.AddHistory) &&
    state1
  ) {
    return NodeState.Add
  } else {
    return NodeState.None
  }
}

export function renderTree(data: TreeDataProvider) {
  return data.atlas ? renderAtlasTree(data) : renderPassiveTree(data)
}

export function loadTree(tree: TreeViewState, renderer: Renderer) {
  return 'atlas' in tree
    ? loadAtlasTree(tree, renderer)
    : loadPassiveTree(tree, renderer)
}
