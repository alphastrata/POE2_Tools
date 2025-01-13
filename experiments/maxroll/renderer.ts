import { DragState } from '@/ui/DragView'
import { BundleData, GlyphData } from './assets'
import { imageScale } from './vertexWriter'

export const enum Layer {
  BACKGROUND,
  STATIC_BACKGROUND, // ascendancy tree frame
  ATLAS_BACKGROUND, // atlas background, additive
  ATLAS_ILLUSTRATION, // atlas sub-trees, additive
  MASTERY_BACKGROUND,
  MASTERY_ICON,
  MASTERY_HIGHLIGHT,
  ARTWORK, // character art
  ASCENDANCY_TEXT,
  LINE,
  NODE_ICON,
  NODE_FRAME,
  NODE_JEWEL,
  NODE_HIGHLIGHT,
  JEWEL_RADIUS1, // additive
  JEWEL_RADIUS2 // additive
}

export type RenderItem = {
  layer: number
  image: string
  color?: { r: number; g: number; b: number; a: number; desaturate?: number }
} & (
  | {
      type: 'background'
    }
  | {
      type: 'image'
      centered: boolean
      size?: number
      scale?: number
      angle?: number
      x: number
      y: number
      interactKey?: number
    }
  | {
      type: 'twohalf'
      x: number
      y: number
    }
  | {
      type: 'line'
      x1: number
      y1: number
      x2: number
      y2: number
    }
  | {
      type: 'arc'
      x: number
      y: number
      r: number
      a1: number
      a2: number
    }
  | {
      type: 'circle'
      x: number
      y: number
      r: number
    }
  | {
      type: 'text'
      text: string
      x: number
      y: number
      centered?: boolean
    }
)

export abstract class Renderer {
  loaded = new Set<string>()
  images: Record<
    string,
    {
      bundle: string
      index: number
      u0: number
      v0: number
      u1: number
      v1: number
      x: number
      y: number
      w: number
      h: number
    }
  > = {}
  fontData: Record<string, Record<string, GlyphData>> = {}

  loadBundle(name: string, bundle: BundleData) {
    this.loaded.add(name)
    bundle.forEach((data, index) => {
      const { width, height } = data.image
      for (const [key, image] of Object.entries(data.index)) {
        if (typeof image !== 'object') continue
        this.images[key] = {
          bundle: name,
          index,
          x: image.x,
          y: image.y,
          w: image.w,
          h: image.h,
          u0: image.x / width,
          v0: image.y / height,
          u1: (image.x + image.w) / width,
          v1: (image.y + image.h) / height
        }
      }
      if (data.glyphs) {
        Object.assign(this.fontData, data.glyphs)
      }
    })
  }

  constructor(
    public readonly canvas: HTMLCanvasElement,
    protected state: DragState
  ) {}

  update(state: DragState) {
    this.state = state
    const pixelRatio = window.devicePixelRatio ?? 1
    const width = Math.round(state.size.width * pixelRatio)
    const height = Math.round(state.size.height * pixelRatio)
    if (width < 10 || height < 10) return
    if (width !== this.canvas.width || height !== this.canvas.height) {
      if (width !== this.canvas.width) this.canvas.width = width
      if (height !== this.canvas.height) this.canvas.height = height
      this.canvas.style.width = `${state.size.width}px`
      this.canvas.style.height = `${state.size.height}px`
      this.resize()
    }
  }
  protected resize() {}

  destroy() {}

  private interactive: {
    layer: number
    x: number
    y: number
    r2: number
    key: number
  }[] = []

  setItems(items: RenderItem[]) {
    this.interactive.length = 0
    for (const item of items) {
      if (item.type !== 'image' || item.interactKey == null) continue
      const image = this.images[item.image]
      if (!image) continue
      this.interactive.push({
        layer: item.layer,
        x: item.x,
        y: item.y,
        r2: Math.pow((Math.min(image.w, image.h) * imageScale) / 2, 2),
        key: item.interactKey
      })
    }
    this.interactive.sort((a, b) => b.layer - a.layer)
  }
  abstract render(): void

  locate(x: number, y: number) {
    const { transform } = this.state
    x = (x - transform.x) / transform.scale
    y = (y - transform.y) / transform.scale
    for (const item of this.interactive) {
      const dx = item.x - x
      const dy = item.y - y
      if (dx * dx + dy * dy < item.r2) {
        return item.key
      }
    }
  }

  writeText(item: RenderItem & { type: 'text' }) {
    const glyphs: GlyphData[] = []
    const fontData = this.fontData[item.image]
    if (!fontData) return []

    let width = 0
    for (const chr of item.text) {
      const glyph = fontData[chr]
      if (glyph) {
        glyphs.push(glyph)
        width += glyph.advance * imageScale
      }
    }

    let x = item.x
    let y = item.y
    if (item.centered) x -= width / 2

    const items: RenderItem[] = []
    for (const glyph of glyphs) {
      items.push({
        type: 'image',
        image: glyph.image,
        layer: item.layer,
        x: x - glyph.dx * imageScale,
        y: y - glyph.dy * imageScale,
        centered: false,
        color: item.color
      })
      x += glyph.advance * imageScale
    }
    return items
  }
}
