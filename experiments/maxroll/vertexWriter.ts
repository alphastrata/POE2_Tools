import { RenderItem, Renderer } from './renderer'

type ShaderType = 'background' | 'image' | 'circle'

export function itemShader(type: RenderItem['type']): ShaderType {
  switch (type) {
    case 'background':
      return 'background'
    case 'circle':
      return 'circle'
    default:
      return 'image'
  }
}

export const imageScale = 2

function encodeColor(color: RenderItem['color']) {
  const r = Math.min(255, Math.max(0, Math.round((color?.r ?? 1) * 128)))
  const g = Math.min(255, Math.max(0, Math.round((color?.g ?? 1) * 128)))
  const b = Math.min(255, Math.max(0, Math.round((color?.b ?? 1) * 128)))
  const a = Math.min(255, Math.max(0, Math.round((color?.a ?? 1) * 255)))
  return r | (g << 8) | (b << 16) | (a << 24)
}

const rectUV = [
  [0, 0],
  [1, 0],
  [0, 1],
  [1, 0],
  [1, 1],
  [0, 1]
]
const twoSideXYUV = [
  [-1, -1, 0, 0, 0],
  [1, -1, 1, 0, 0],
  [-1, 0, 0, 1, 0],
  [1, -1, 1, 0, 0],
  [1, 0, 1, 1, 0],
  [-1, 0, 0, 1, 0],

  [-1, 0, 0, 1, 1],
  [1, 0, 1, 1, 1],
  [1, 1, 1, 0, 1],
  [-1, 0, 0, 1, 1],
  [1, 1, 1, 0, 1],
  [-1, 1, 0, 0, 1]
]

type VertexImage = {
  x: number
  y: number
  u: number
  v: number
  color: number
  desaturate: number
}
const vertexImageSize = 24

export const lineSubImage = {
  x: 0,
  y: 14,
  w: 717,
  h: 16
}
export const arcSubImage = {
  x: 48,
  y: 48,
  w: 669,
  h: 669
}

export type ImageRect = {
  u0: number
  v0: number
  u1: number
  v1: number
  x: number
  y: number
  w: number
  h: number
}
export type SubImage = {
  x: number
  y: number
  w: number
  h: number
}

export function subImage(image: ImageRect, sub: SubImage) {
  return {
    x: image.x + sub.x,
    y: image.y + sub.y,
    w: sub.w,
    h: sub.h,
    u0: image.u0 + (image.u1 - image.u0) * (sub.x / image.w),
    v0: image.v0 + (image.v1 - image.v0) * (sub.y / image.h),
    u1: image.u0 + (image.u1 - image.u0) * ((sub.x + sub.w) / image.w),
    v1: image.v0 + (image.v1 - image.v0) * ((sub.y + sub.h) / image.h)
  }
}

export function writeVerticesImage(items: RenderItem[], renderer: Renderer) {
  const vertices: VertexImage[] = []

  function writeItem(item: RenderItem) {
    const color = encodeColor(item.color)
    const desaturate = item.color?.desaturate ?? 0
    let image: ImageRect = renderer.images[item.image]
    if (item.type === 'image') {
      const scale = imageScale * (item.scale ?? 1)
      const width = item.size ?? image.w * scale
      const height = item.size ?? image.h * scale
      const cos = item.angle ? Math.cos(item.angle) : 1
      const sin = item.angle ? Math.sin(item.angle) : 0
      const delta = item.centered ? -0.5 : 0
      for (const [u, v] of rectUV) {
        const x = (u + delta) * width
        const y = (v + delta) * height
        vertices.push({
          x: item.x + cos * x - sin * y,
          y: item.y + sin * x + cos * y,
          u: image.u0 + (image.u1 - image.u0) * u,
          v: image.v0 + (image.v1 - image.v0) * v,
          color,
          desaturate
        })
      }
    } else if (item.type === 'twohalf') {
      const width = (image.w * imageScale) / 2
      const height = image.h * imageScale
      for (const [x, y, u, v] of twoSideXYUV) {
        vertices.push({
          x: item.x + x * width,
          y: item.y + y * height,
          u: image.u0 + (image.u1 - image.u0) * u,
          v: image.v0 + (image.v1 - image.v0) * v,
          color,
          desaturate
        })
      }
    } else if (item.type === 'line') {
      image = subImage(image, lineSubImage)
      const dx = item.x2 - item.x1
      const dy = item.y2 - item.y1
      const dist = Math.sqrt(dx * dx + dy * dy)
      const px = (-dy / dist) * (image.h * imageScale)
      const py = (dx / dist) * (image.h * imageScale)
      const uScale = dist / (image.w * imageScale)
      for (let start = 0; start < uScale; start += 1) {
        const width = Math.min(1, uScale - start)
        for (const [u, v] of rectUV) {
          const x = (start + u * width) / uScale
          vertices.push({
            x: item.x1 + dx * x + px * (v - 0.5),
            y: item.y1 + dy * x + py * (v - 0.5),
            u: image.u0 + (image.u1 - image.u0) * u * width,
            v: image.v0 + (image.v1 - image.v0) * v,
            color,
            desaturate
          })
        }
      }
    } else if (item.type === 'arc') {
      image = subImage(image, arcSubImage)
      let { a1, a2 } = item
      let diff = a2 - a1
      while (diff <= -Math.PI) diff += Math.PI * 2
      while (diff > Math.PI) diff -= Math.PI * 2
      if (diff < 0) {
        const t = a1
        a1 = a2
        a2 = t
        diff = -diff
      }
      const maxStep = 0.3
      const r1 = item.r + 50
      const r2 = item.r - 50
      const r1t = r1 / image.w / imageScale
      const r2t = r2 / image.h / imageScale
      // we flip the arc texture to hide the cuts
      let flip = false
      function tcoords(x: number, y: number) {
        if (flip) {
          return {
            u: image.u1 + (image.u0 - image.u1) * y,
            v: image.v1 + (image.v0 - image.v1) * x
          }
        } else {
          return {
            u: image.u1 + (image.u0 - image.u1) * x,
            v: image.v1 + (image.v0 - image.v1) * y
          }
        }
      }
      while (diff > 1e-3) {
        const arc = Math.min(diff, Math.PI / 2)
        const steps = Math.ceil(arc / maxStep)
        const vtop: VertexImage[] = []
        const vbot: VertexImage[] = []
        for (let i = 0; i <= steps; ++i) {
          const da = (arc * i) / steps
          const wcos = Math.cos(a1 + da)
          const wsin = Math.sin(a1 + da)
          const tcos = Math.cos(da)
          const tsin = Math.sin(da)
          vtop.push({
            x: item.x + r1 * wcos,
            y: item.y + r1 * wsin,
            ...tcoords(r1t * tcos, r1t * tsin),
            color,
            desaturate
          })
          vbot.push({
            x: item.x + r2 * wcos,
            y: item.y + r2 * wsin,
            ...tcoords(r2t * tcos, r2t * tsin),
            color,
            desaturate
          })
        }
        for (let i = 0; i < steps; ++i) {
          vertices.push(
            vtop[i],
            vtop[i + 1],
            vbot[i],
            vtop[i + 1],
            vbot[i + 1],
            vbot[i]
          )
        }
        diff -= arc
        a1 += arc
        flip = !flip
      }
    }
  }

  for (const item of items) {
    writeItem(item)
  }

  const buffer = new ArrayBuffer(vertices.length * vertexImageSize)
  const f32 = new Float32Array(buffer)
  const u32 = new Uint32Array(buffer)
  for (let i = 0; i < vertices.length; ++i) {
    const pos = (i * vertexImageSize) >> 2
    const { x, y, u, v, color, desaturate } = vertices[i]
    f32[pos] = x
    f32[pos + 1] = y
    f32[pos + 2] = u
    f32[pos + 3] = v
    u32[pos + 4] = color
    f32[pos + 5] = desaturate
  }
  return buffer
}

type VertexBackground = {
  x: number
  y: number
  scale: number
}
const vertexBackgroundSize = 12

export function writeVerticesBackground(
  items: RenderItem[],
  renderer: Renderer
) {
  const vertices: VertexBackground[] = []
  for (const item of items) {
    const image = renderer.images[item.image]
    if (item.type === 'background') {
      for (const [u, v] of rectUV) {
        vertices.push({
          x: u,
          y: v,
          scale: image.w * imageScale
        })
      }
    }
  }
  const buffer = new ArrayBuffer(vertices.length * vertexBackgroundSize)
  const f32 = new Float32Array(buffer)
  for (let i = 0; i < vertices.length; ++i) {
    const pos = (i * vertexBackgroundSize) >> 2
    const { x, y, scale } = vertices[i]
    f32[pos] = x
    f32[pos + 1] = y
    f32[pos + 2] = scale
  }
  return buffer
}

type VertexCircle = {
  x: number
  y: number
  cx: number
  cy: number
  r: number
  color: number
}
const vertexCircleSize = 24

export function writeVerticesCircle(items: RenderItem[], renderer: Renderer) {
  const vertices: VertexCircle[] = []
  for (const item of items) {
    const color = encodeColor(item.color)
    if (item.type === 'circle') {
      const padding = 2 / 0.05
      for (const [u, v] of rectUV) {
        vertices.push({
          x: item.x + (item.r + padding) * (u * 2 - 1),
          y: item.y + (item.r + padding) * (v * 2 - 1),
          cx: item.x,
          cy: item.y,
          r: item.r,
          color
        })
      }
    }
  }
  const buffer = new ArrayBuffer(vertices.length * vertexCircleSize)
  const f32 = new Float32Array(buffer)
  const u32 = new Uint32Array(buffer)
  for (let i = 0; i < vertices.length; ++i) {
    const pos = (i * vertexCircleSize) >> 2
    const { x, y, cx, cy, r, color } = vertices[i]
    f32[pos] = x
    f32[pos + 1] = y
    f32[pos + 2] = cx
    f32[pos + 3] = cy
    f32[pos + 4] = r
    u32[pos + 5] = color
  }
  return buffer
}
