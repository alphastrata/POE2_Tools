import { PassiveSkillDefinition } from '@/data/types'
import { NodeState, TreeViewStateAtlas } from '@/logic/tree'
import { orbitRadii, TreeDataProvider } from '@/logic/treeData'
import {
  getBundle,
  imagePath,
  imagePaths,
  renderFontBitmap,
  skillIconPath
} from './assets'
import { frameColor, lineState } from './renderItems'
import { Layer, Renderer, RenderItem } from './renderer'

function nodeColor(state: NodeState) {
  switch (state & NodeState.StateMask) {
    case NodeState.Add:
    case NodeState.RemoveHistory:
      return { r: 0.8, g: 0.8, b: 0.8, a: 1.0 }
    case undefined:
      return { r: 0.6, g: 0.6, b: 0.6, a: 1.0 }
  }
}

const treeBackground = imagePath('Art/2DArt/UIImages/Common/Background2')

const atlasBackground = imagePath(
  'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveBackground'
)

const rootBackground = imagePath(
  'Art/2DArt/UIImages/InGame/AtlasScreen/GeneralSubTreeBG'
)

const pointBackground = imagePath(
  'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassivePointBackground'
)

const frameNormal = imagePaths({
  normal:
    'Art/2DArt/UIImages/InGame/AtlasScreen/PassiveSkillScreenPassiveFrameNormal',
  active:
    'Art/2DArt/UIImages/InGame/AtlasScreen/PassiveSkillScreenPassiveFrameActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/AtlasScreen/PassiveSkillScreenPassiveFrameCanAllocate'
})
const frameNotable = imagePaths({
  normal:
    'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveSkillScreenNotableFrameNormal',
  active:
    'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveSkillScreenNotableFrameActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveSkillScreenNotableFrameCanAllocate'
})
const frameKeystone = imagePaths({
  normal:
    'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveSkillScreenKeystoneFrameNormal',
  active:
    'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveSkillScreenKeystoneFrameActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveSkillScreenKeystoneFrameCanAllocate'
})
const frameWormhole = imagePaths({
  normal: 'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasWormholeFrameNormal',
  active: 'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasWormholeFrameActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasWormholeFrameCanAllocate'
})
const iconWormhole = imagePaths({
  active:
    'Art/2DArt/SkillIcons/passives/AtlasTrees/AtlasPassiveWormholeActiveIcon',
  default:
    'Art/2DArt/SkillIcons/passives/AtlasTrees/AtlasPassiveWormholeDefaultIcon'
})

const lineTextures = imagePaths({
  normal:
    'Art/2DArt/PassiveTree/AtlasPassiveSkillScreenCurvesNormalBlueTogether',
  active:
    'Art/2DArt/PassiveTree/AtlasPassiveSkillScreenCurvesActiveBlueTogether',
  intermediate:
    'Art/2DArt/PassiveTree/AtlasPassiveSkillScreenCurvesIntermediateBlueTogether',
  ornament:
    'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveSkillScreenOrnament1'
})

const startNode = imagePath(
  'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveSkillScreenStart'
)
const masteryHighlight = imagePath(
  'Art/2DArt/UIImages/InGame/AtlasScreen/AtlasPassiveMasteryIconHighlight'
)

function nodeSize(skill: PassiveSkillDefinition) {
  if (skill.is_notable) {
    return 48
  } else if (skill.is_keystone) {
    return 84
  } else if (skill.is_just_icon) {
    return 0
  } else {
    return 26
  }
}

export function renderAtlasTree(data: TreeDataProvider) {
  const items: RenderItem[] = []
  const tree = data.state as TreeViewStateAtlas

  if (!tree.hideBackground) {
    items.push({
      layer: Layer.BACKGROUND,
      image: treeBackground,
      type: 'background'
    })
  }

  items.push({
    layer: Layer.ATLAS_BACKGROUND,
    image: atlasBackground,
    type: 'image',
    x: -100,
    y: -550,
    centered: true,
    scale: 2.5
  })

  for (const id of data.allNodes()) {
    const node = data.node(id)!
    const skill = data.nodeSkill(id)!

    if (skill.is_just_icon) {
      items.push({
        layer: Layer.MASTERY_BACKGROUND,
        image: skillIconPath(skill.icon),
        type: 'image',
        x: node.x,
        y: node.y,
        centered: true
      })
      const active = node.group.nodes.find((id) => {
        const node = data.node(id)
        return node && node.skill.is_notable && node.active
      })
      if (active != null) {
        items.push({
          layer: Layer.MASTERY_HIGHLIGHT,
          image: masteryHighlight,
          type: 'image',
          x: node.x,
          y: node.y,
          centered: true
        })
      }
    } else {
      const subTree = skill.atlas_sub_tree
        ? data.atlas_sub_trees[skill.atlas_sub_tree]
        : undefined
      if (data.tree.root_passives.includes(id) && subTree) {
        items.push({
          layer: Layer.NODE_FRAME,
          image: imagePath(subTree.start_icon),
          color: frameColor(node.state),
          type: 'image',
          x: node.x,
          y: node.y,
          centered: true
        })
      } else {
        items.push({
          layer: Layer.NODE_ICON,
          image: skillIconPath(skill.icon),
          color: nodeColor(node.state),
          type: 'image',
          x: node.x,
          y: node.y,
          centered: true,
          size: skill.is_notable ? 98 : skill.is_keystone ? 128 : 64
        })
        const frames = skill.is_notable
          ? frameNotable
          : skill.is_keystone
          ? frameKeystone
          : frameNormal
        items.push({
          layer: Layer.NODE_FRAME,
          image: node.state ? frames.active : frames.normal,
          color: frameColor(node.state),
          type: 'image',
          interactKey: data.tree.root_passives.includes(id) ? undefined : id,
          x: node.x,
          y: node.y,
          centered: true
        })
      }
    }

    for (const { id: id2, radius } of node.connections) {
      if (id2 <= id) continue
      const node2 = data.node(id2)
      if (!node2) continue
      const skill2 = node2.skill
      if (skill.is_just_icon || skill2.is_just_icon) continue

      const state = lineState(node.state, node2.state)
      const color = frameColor(state)
      const image = state ? lineTextures.active : lineTextures.normal
      const layer = Layer.LINE

      if (radius && orbitRadii[Math.abs(radius)]) {
        const r = orbitRadii[Math.abs(radius)]
        const dx = node2.x - node.x
        const dy = node2.y - node.y
        const dist = Math.sqrt(dx * dx + dy * dy)
        if (dist < r * 2) {
          const perp = Math.sqrt(r * r - (dist * dist) / 4) * Math.sign(radius)
          const cx = node.x + dx / 2 + perp * (dy / dist)
          const cy = node.y + dy / 2 - perp * (dx / dist)
          items.push({
            layer,
            image,
            color,
            type: 'arc',
            x: cx,
            y: cy,
            r: r,
            a1: Math.atan2(node.y - cy, node.x - cx),
            a2: Math.atan2(node2.y - cy, node2.x - cx)
          })
        }
      } else if (
        node.group === node2.group &&
        node.radius === node2.radius &&
        !radius
      ) {
        items.push({
          layer,
          image,
          color,
          type: 'arc',
          x: node.group.x,
          y: node.group.y,
          r: node.radius,
          a1: node.arc,
          a2: node2.arc
        })
      } else {
        items.push({
          layer,
          image,
          color,
          type: 'line',
          x1: node.x,
          y1: node.y,
          x2: node2.x,
          y2: node2.y
        })
      }
    }
  }

  for (const id of data.tree.root_passives) {
    const node = data.node(id)!
    const skill = data.nodeSkill(id)!
    const subTree = skill.atlas_sub_tree
      ? data.atlas_sub_trees[skill.atlas_sub_tree]
      : undefined
    items.push({
      layer: Layer.ATLAS_ILLUSTRATION,
      image: subTree ? imagePath(subTree.background) : rootBackground,
      type: 'image',
      x: node.x + (subTree?.x ?? 0),
      y: node.y + (subTree?.y ?? 0),
      centered: true
    })
    const limit = skill.atlas_sub_tree
      ? data.state.limit?.subTrees[skill.atlas_sub_tree]
      : data.state.limit?.normal
    const count = skill.atlas_sub_tree
      ? data.state.count.subTrees[skill.atlas_sub_tree] ?? 0
      : data.state.count.normal
    if (limit) {
      items.push({
        layer: Layer.NODE_FRAME,
        image: pointBackground,
        type: 'image',
        x: node.x + (subTree?.x1 ?? 0),
        y: node.y + (subTree?.y1 ?? 740),
        centered: true
      })
      items.push({
        layer: Layer.NODE_JEWEL,
        image: 'atlas_font',
        type: 'text',
        text: `${count}/${limit}`,
        x: node.x + (subTree?.x1 ?? 0),
        y: node.y + (subTree?.y1 ?? 740) + 24,
        centered: true
      })
    }
  }

  if (tree.highlight) {
    for (const id of tree.highlight) {
      const node = data.node(id)
      if (!node) continue

      let size =
        node.skill.is_keystone || node.skill.is_wormhole
          ? 256
          : node.skill.is_notable
          ? 192
          : 128

      items.push({
        layer: Layer.NODE_HIGHLIGHT,
        image: treeBackground, // neutral image that's always loaded
        type: 'circle',
        x: node.x,
        y: node.y,
        r: size / 2,
        color: { r: 0, g: 1, b: 0, a: 1 }
      })
    }
  }

  return items
}

export function loadAtlasTree(tree: TreeViewStateAtlas, renderer: Renderer) {
  const bundles = ['atlas_passive_tree']
  if (
    bundles.every((name) => renderer.loaded.has(name)) &&
    renderer.loaded.has('atlas_font')
  )
    return undefined
  const promises: Promise<any>[] = bundles.map((name) =>
    getBundle(name).then((bundle) => renderer.loadBundle(name, bundle))
  )
  promises.push(
    renderFontBitmap('atlas_font', 'FontinSmallCaps', 36, '0123456789/').then(
      (img) => {
        if (img) renderer.loadBundle('atlas_font', img)
      }
    )
  )
  return Promise.all(promises)
}
