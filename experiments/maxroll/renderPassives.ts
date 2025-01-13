import Data from '@/data'
import { PassiveSkillDefinition } from '@/data/types'
import { NodeState, TreeViewStatePassives, radiusJewel } from '@/logic/tree'
import { TreeDataProvider, orbitRadii } from '@/logic/treeData'
import { PlayerClass } from '@/types/profile'
import {
  BundleData,
  getBundle,
  imagePath,
  imagePaths,
  loadFont,
  skillIconPath
} from './assets'
import { frameColor, lineState } from './renderItems'
import { Layer, RenderItem, Renderer } from './renderer'

function nodeColor(state: NodeState) {
  switch (state & NodeState.StateMask) {
    case NodeState.Add:
    case NodeState.RemoveHistory:
      return { r: 0.8, g: 0.8, b: 0.8, a: 1.0, desaturate: 0.2 }
    case NodeState.None:
      return { r: 0.6, g: 0.6, b: 0.6, a: 1.0, desaturate: 0.4 }
  }
}

const treeBackground = imagePath('Art/2DArt/UIImages/Common/Background2')

const frameNormal = imagePaths({
  normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenPassiveFrameNormal',
  active: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenPassiveFrameActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenPassiveFrameCanAllocate'
})
const frameAscendancy = imagePaths({
  normal:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenAscendancyFrameSmallNormal',
  active:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenAscendancyFrameSmallAllocated',
  canAllocate:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenAscendancyFrameSmallCanAllocate'
})
const frameNotable = imagePaths({
  normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenNotableFrameNormal',
  active: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenNotableFrameActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenNotableFrameCanAllocate'
})
const frameNotableAscendancy = imagePaths({
  normal:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenAscendancyFrameLargeNormal',
  active:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenAscendancyFrameLargeAllocated',
  canAllocate:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenAscendancyFrameLargeCanAllocate'
})
const frameKeystone = imagePaths({
  normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenKeystoneFrameNormal',
  active: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenKeystoneFrameActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenKeystoneFrameCanAllocate'
})
const frameJewel = imagePaths({
  normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelSocketNormal',
  active: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelSocketActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelSocketCanAllocate'
})
const frameJewelLarge = imagePaths({
  normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelSocketAltNormal',
  active: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelSocketAltActive',
  canAllocate:
    'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelSocketAltCanAllocate'
})
const frameJewelCircle = {
  Small: imagePaths({
    normal: 'Art/2DArt/UIImages/InGame/JewelSocketClusterAltNormal1Small',
    canAllocate:
      'Art/2DArt/UIImages/InGame/JewelSocketClusterAltCanAllocate1Small'
  }),
  Medium: imagePaths({
    normal: 'Art/2DArt/UIImages/InGame/JewelSocketClusterAltNormal1Medium',
    canAllocate:
      'Art/2DArt/UIImages/InGame/JewelSocketClusterAltCanAllocate1Medium'
  }),
  Large: imagePaths({
    normal: 'Art/2DArt/UIImages/InGame/JewelSocketClusterAltNormal1Large',
    canAllocate:
      'Art/2DArt/UIImages/InGame/JewelSocketClusterAltCanAllocate1Large'
  })
}

const lineTextures = imagePaths({
  normal: 'Art/2DArt/PassiveTree/PassiveSkillScreenCurvesNormalTogether',
  active: 'Art/2DArt/PassiveTree/PassiveSkillScreenCurvesActiveTogether',
  intermediate:
    'Art/2DArt/PassiveTree/PassiveSkillScreenCurvesIntermediateTogether'
})

const ascendancyLineTextures = imagePaths({
  normal:
    'Art/2DArt/PassiveTree/AscendancyPassiveSkillScreenCurvesNormalTogether',
  active:
    'Art/2DArt/PassiveTree/AscendancyPassiveSkillScreenCurvesActiveTogether',
  intermediate:
    'Art/2DArt/PassiveTree/AscendancyPassiveSkillScreenCurvesIntermediateTogether'
})

const startInactive = imagePath(
  'Art/2DArt/UIImages/InGame/PassiveTree/PassiveTreeMainCircle'
)
const startActive = imagePath(
  `Art/2DArt/UIImages/InGame/PassiveTree/PassiveTreeMainCircleActive2`
)
const startAscendancy = imagePath(
  `Art/2DArt/UIImages/InGame/Ascendancy/AscendancySelectFrame`
)

const ascendancyRoot = imagePath(
  'Art/2DArt/UIImages/InGame/PassiveSkillScreenAscendancyMiddle'
)

const jewelRadius = imagePaths({
  normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelCircle1',
  outer: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelCircle1',
  inverse: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenJewelCircle1inverse'
})
const jewelRadiusSpecial = [
  imagePaths({
    normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenVaalJewelCircle1',
    outer: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenVaalJewelCircle2'
  }),
  imagePaths({
    normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenKaruiJewelCircle1',
    outer: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenKaruiJewelCircle2'
  }),
  imagePaths({
    normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenMarakethJewelCircle1',
    outer: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenMarakethJewelCircle2'
  }),
  imagePaths({
    normal: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenTemplarJewelCircle1',
    outer: 'Art/2DArt/UIImages/InGame/PassiveSkillScreenTemplarJewelCircle2'
  }),
  imagePaths({
    normal:
      'Art/2DArt/UIImages/InGame/PassiveSkillScreenEternalEmpireJewelCircle1',
    outer:
      'Art/2DArt/UIImages/InGame/PassiveSkillScreenEternalEmpireJewelCircle2'
  })
]

function nodeSize(skill: PassiveSkillDefinition) {
  if (skill.is_ascendancy_root) {
    return 10
  } else if (skill.is_jewel_socket) {
    return 30
  } else if (skill.is_notable) {
    return 48
  } else if (skill.is_keystone) {
    return 84
  } else if (skill.is_just_icon) {
    return 0
  } else {
    return 26
  }
}

export function renderPassiveTree(data: TreeDataProvider) {
  const items: RenderItem[] = []
  const tree = data.state as TreeViewStatePassives

  if (!tree.ascendancyOnly && !tree.hideBackground) {
    items.push({
      layer: Layer.BACKGROUND,
      image: treeBackground,
      type: 'background'
    })
  }

  for (const id of data.allNodes()) {
    const node = data.node(id)!
    if (!data.showAscendancy(node.skill.ascendancy)) continue

    const skill = data.nodeSkill(id)!

    if (skill.mastery) {
      let active = node.state != 0
      // if (skill.is_just_icon) {
      //   active = node.connections.some(({ id }) => data.node(id)?.active)
      // }
      items.push({
        layer: Layer.MASTERY_BACKGROUND,
        image: imagePath(skill.mastery.active_image),
        type: 'image',
        x: node.x,
        y: node.y,
        centered: true,
        color: active ? undefined : { r: 1, g: 1, b: 1, a: 0.15 }
      })
    }

    if (skill.is_ascendancy_root) {
      items.push({
        layer: Layer.NODE_FRAME,
        image: ascendancyRoot,
        type: 'image',
        x: node.x,
        y: node.y,
        centered: true
      })
    } else if (skill.is_jewel_socket) {
      const jewelSlot = Data.jewel_slots[node.skill_id]
      const frames = jewelSlot?.size ? frameJewelLarge : frameJewel
      items.push({
        layer: Layer.NODE_FRAME,
        image: node.state ? frames.active : frames.normal,
        color: frameColor(
          node.state | (node.weaponSet << NodeState.WeaponSetShift)
        ),
        type: 'image',
        interactKey: id,
        x: node.x,
        y: node.y,
        centered: true
      })
      if (node.jewel) {
        const base = Data.items[node.jewel.base]
        const icon = jewelSlot?.size
          ? base?.visual_identity.socketed_image_large
          : base?.visual_identity.socketed_image
        if (icon) {
          items.push({
            layer: Layer.NODE_JEWEL,
            image: imagePath(icon),
            type: 'image',
            x: node.x,
            y: node.y,
            centered: true
          })
        }

        if (node.active) {
          const radius = radiusJewel(data, id)
          if (radius) {
            const tex =
              (radius.treeVersion &&
                jewelRadiusSpecial[radius.treeVersion - 1]) ||
              jewelRadius
            items.push({
              layer: Layer.JEWEL_RADIUS1,
              image: tex.normal,
              type: 'image',
              x: radius.x,
              y: radius.y,
              centered: true,
              size: radius.maxRadius * 2
            })
            items.push({
              layer: Layer.JEWEL_RADIUS2,
              image: tex.outer,
              type: 'image',
              x: radius.x,
              y: radius.y,
              centered: true,
              angle: 180,
              size: radius.maxRadius * 2
            })
            if (radius.minRadius) {
              items.push({
                layer: Layer.JEWEL_RADIUS1,
                image: jewelRadius.inverse,
                type: 'image',
                x: radius.x,
                y: radius.y,
                centered: true,
                size: radius.minRadius * 2.125
              })
              items.push({
                layer: Layer.JEWEL_RADIUS2,
                image: jewelRadius.inverse,
                type: 'image',
                x: radius.x,
                y: radius.y,
                centered: true,
                angle: 180,
                size: radius.minRadius * 2.125
              })
            }
          }
        }
      } else if (jewelSlot?.size) {
        const frames = frameJewelCircle[jewelSlot.size]
        items.push({
          layer: Layer.NODE_JEWEL,
          image: node.state ? frames.canAllocate : frames.normal,
          type: 'image',
          x: node.x,
          y: node.y,
          centered: true
        })
      }
      //TODO: jewel icon and radius go here
    } else if (skill.starting_node) {
      // do nothing
    } else if (skill.is_just_icon) {
      if (!skill.mastery) {
        items.push({
          layer: Layer.MASTERY_ICON,
          image: skillIconPath(skill.icon),
          type: 'image',
          x: node.x,
          y: node.y,
          centered: true
        })
      }
    } else {
      items.push({
        layer: Layer.NODE_ICON,
        image: skillIconPath(skill.icon),
        color: nodeColor(
          node.state | (node.weaponSet << NodeState.WeaponSetShift)
        ),
        type: 'image',
        x: node.x,
        y: node.y,
        centered: true,
        size: skill.is_notable ? 98 : skill.is_keystone ? 128 : 64
      })
      const frames = skill.is_notable
        ? skill.ascendancy
          ? frameNotableAscendancy
          : frameNotable
        : skill.is_keystone
        ? frameKeystone
        : skill.ascendancy
        ? frameAscendancy
        : frameNormal
      items.push({
        layer: Layer.NODE_FRAME,
        image: node.state ? frames.active : frames.normal,
        color: frameColor(
          node.state | (node.weaponSet << NodeState.WeaponSetShift)
        ),
        type: 'image',
        interactKey: id,
        x: node.x,
        y: node.y,
        centered: true
      })
    }

    for (const { id: id2, radius } of node.connections) {
      if (id2 <= id) continue
      const node2 = data.node(id2)
      if (!node2) continue
      const skill2 = node2.skill
      if (skill.ascendancy !== skill2.ascendancy) continue
      if (skill.starting_node || skill2.starting_node) continue
      if (skill.is_just_icon || skill2.is_just_icon) continue

      const textures = skill.ascendancy ? ascendancyLineTextures : lineTextures

      const state = lineState(
        node.state | (node.weaponSet << NodeState.WeaponSetShift),
        node2.state | (node2.weaponSet << NodeState.WeaponSetShift)
      )
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
            layer: Layer.LINE,
            image: state ? textures.active : textures.normal,
            color: frameColor(state),
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
          layer: Layer.LINE,
          image: state ? textures.active : textures.normal,
          color: frameColor(state),
          type: 'arc',
          x: node.group.x,
          y: node.group.y,
          r: node.radius,
          a1: node.arc,
          a2: node2.arc
        })
      } else {
        items.push({
          layer: Layer.LINE,
          image: state ? textures.active : textures.normal,
          color: frameColor(state),
          type: 'line',
          x1: node.x,
          y1: node.y,
          x2: node2.x,
          y2: node2.y
        })
      }
    }
  }

  if (tree.highlight) {
    for (const id of tree.highlight) {
      const node = data.node(id)
      if (!node || !data.showAscendancy(node.skill.ascendancy)) continue

      let size = node.skill.mastery
        ? 224
        : node.skill.is_keystone
        ? 256
        : node.skill.is_notable || node.skill.is_jewel_socket
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

  const chr = Data.characters[tree.charClass]
  const ascendancy = tree.ascendancy
    ? chr.ascendancies[tree.ascendancy]
    : undefined
  const illustration = ascendancy?.illustration ?? chr.illustration
  if (illustration) {
    items.push({
      layer: Layer.ARTWORK,
      image: imagePath(illustration.image),
      type: 'image',
      x: 0,
      y: 0,
      centered: true,
      scale: 2
    })

    if (ascendancy) {
      let [x, y, w, h] = ascendancy.flavour_rect
        .split(',')
        .map((p) => parseInt(p))
      //x >>= 1
      //y >>= 1
      items.push({
        layer: Layer.ASCENDANCY_TEXT,
        image: `ascendancy_${tree.ascendancy}`,
        type: 'image',
        x: x * 2,
        y: y * 2 + ascendancy.illustration!.y,
        centered: true
      })
    }
  }

  if (!tree.ascendancyOnly) {
    const start = data.tree.characterRoot[tree.charClass]
    const root = data.node(start)!

    items.push({
      layer: Layer.NODE_ICON,
      image: startActive,
      type: 'image',
      x: 0,
      y: 0,
      centered: true,
      angle: Math.atan2(root.y, root.x) + Math.PI / 2
    })
    items.push({
      layer: Layer.NODE_FRAME,
      image: startInactive,
      type: 'image',
      x: 0,
      y: 0,
      centered: true
    })
  } else {
    items.push({
      layer: Layer.STATIC_BACKGROUND,
      image: startAscendancy,
      type: 'image',
      x: 0,
      y: 0,
      centered: true,
      scale: 2.5
    })
  }

  return items
}

async function renderAscendancyText(
  charClass: PlayerClass,
  ascendancy: string
): Promise<BundleData | undefined> {
  const data = Data.characters[charClass]?.ascendancies[ascendancy]
  if (!data) return

  // const bundle = await getBundle(`passive_tree_${charClass.toLowerCase()}`)
  // const entry = bundle.find((e) => e.index[data.background])
  // if (!entry) return

  await loadFont('FontinItalic')

  let [, , w, h] = data.flavour_rect.split(',').map((p) => parseInt(p))
  //w >>= 1
  //h >>= 1

  //const rect = entry.index[data.background]
  const canvas = document.createElement('canvas')
  let sizex = 128
  let sizey = 128
  while (sizex < w) sizex *= 2
  while (sizey < h) sizey *= 2
  canvas.width = sizex
  canvas.height = sizey
  const ctx = canvas.getContext('2d')!
  ctx.font = '52px FontinItalic'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'alphabetic'
  ctx.lineWidth = 1
  ctx.strokeStyle = '#000'
  ctx.fillStyle = `rgb(${data.flavour_colour})`
  let x = w >> 1
  let y = 56
  if (['Deadeye', 'Pathfinder', 'Chronomancer'].includes(data.name)) {
    y = 230
  }

  for (const line of data.flavour_text.replace(/\r/g, '').split('\n')) {
    let out = ''
    for (const word of line.split(' ')) {
      let next = (out ? out + ' ' : '') + word
      if (ctx.measureText(next).width > w - 20) {
        ctx.strokeText(out, x, y)
        ctx.fillText(out, x, y)
        y += 56
        out = word
      } else {
        out = next
      }
    }
    if (out) {
      ctx.strokeText(out, x, y)
      ctx.fillText(out, x, y)
      y += 56
    }
  }
  h = y

  const image = window.createImageBitmap
    ? await window.createImageBitmap(canvas)
    : canvas
  return [
    {
      image,
      index: {
        [`ascendancy_${ascendancy}`]: {
          x: 0,
          y: 0,
          w: w,
          h: h
        }
      }
    }
  ]
}

export function loadPassiveTree(
  tree: TreeViewStatePassives,
  renderer: Renderer
) {
  const bundles = ['passive_tree_common']
  if (!tree.ascendancyOnly) bundles.push('passive_tree_icons')
  bundles.push(`passive_tree_${tree.charClass.toLowerCase()}`)
  if (
    bundles.every((name) => renderer.loaded.has(name)) &&
    (!tree.ascendancy || renderer.loaded.has(`ascendancy_${tree.ascendancy}`))
  )
    return undefined
  const promises: Promise<any>[] = bundles.map((name) =>
    getBundle(name).then((bundle) => renderer.loadBundle(name, bundle))
  )
  if (tree.ascendancy) {
    promises.push(
      renderAscendancyText(tree.charClass, tree.ascendancy).then((img) => {
        if (img) renderer.loadBundle(`ascendancy_${tree.ascendancy}`, img)
      })
    )
  }
  return Promise.all(promises)
}
