import { DragState } from '@/ui/DragView'
import { BundleData, ImageSource } from './assets'
import { Layer, Renderer, RenderItem } from './renderer'
import { ImageRect, imageScale, lineSubImage, subImage } from './vertexWriter'

export class CanvasRenderer extends Renderer {
  private context: CanvasRenderingContext2D

  constructor(canvas: HTMLCanvasElement, state: DragState) {
    super(canvas, state)
    const context = canvas.getContext('2d')
    if (!context) throw Error('Canvas 2D is not supported')
    this.context = context
  }

  private bundleImages: Record<string, ImageSource[]> = {}

  loadBundle(name: string, bundle: BundleData) {
    super.loadBundle(name, bundle)
    if (!this.bundleImages[name]) {
      this.bundleImages[name] = bundle.map((b) => b.image)
    }
  }

  private items: RenderItem[] = []

  setItems(items: RenderItem[]) {
    super.setItems(items)

    this.items = items.slice().sort((a, b) => a.layer - b.layer)
  }

  private patterns = new Map<ImageSource, CanvasPattern | string>()
  private pattern(image: ImageSource, repeat?: string) {
    let p: CanvasPattern | string | undefined = this.patterns.get(image)
    if (!p) {
      try {
        p = this.context.createPattern(image, repeat ?? null) ?? '#000'
      } catch (e) {
        p = '#000'
      }
      this.patterns.set(image, p)
    }
    return p
  }

  private colorCache = new Map<ImageSource, Record<string, ImageSource>>()
  private colored(image: ImageSource, color: NonNullable<RenderItem['color']>) {
    let cache: Record<string, ImageSource> = this.colorCache.get(image)!
    if (!cache) {
      cache = {}
      this.colorCache.set(image, cache)
    }
    const key = JSON.stringify(color)
    if (!cache[key]) {
      const canvas = new OffscreenCanvas(image.width, image.height)
      console.time('color ' + key)
      const ctx = canvas.getContext('2d')!
      if (color.desaturate) {
        ctx.filter = `grayscale(${(color.desaturate * 100).toFixed(2)}%)`
      }
      ctx.drawImage(image, 0, 0)
      ctx.filter = ''
      ctx.globalCompositeOperation = 'multiply'
      ctx.fillStyle = `rgb(${[color.r, color.g, color.b]
        .map((c) => (c * 255).toFixed(0))
        .join(',')})`
      ctx.fillRect(0, 0, image.width, image.height)
      ctx.globalAlpha = color.a
      ctx.globalCompositeOperation = 'destination-atop'
      ctx.drawImage(image, 0, 0)
      console.timeEnd('color ' + key)
      cache[key] = canvas
    }
    return cache[key]
  }

  render() {
    const ctx = this.context

    const { size, transform } = this.state

    ctx.imageSmoothingQuality = 'medium'

    ctx.setTransform(1, 0, 0, 1, 0, 0)
    ctx.clearRect(0, 0, size.width, size.height)
    ctx.setTransform(
      transform.scale,
      0,
      0,
      transform.scale,
      transform.x,
      transform.y
    )

    for (const item of this.items) {
      const imageRect = this.images[item.image]
      if (!imageRect) continue
      let image = this.bundleImages[imageRect.bundle]?.[imageRect.index]
      if (!image) continue
      let rect: ImageRect = imageRect

      if (item.color) {
        image = this.colored(image, item.color)
      }

      if (
        item.layer === Layer.JEWEL_RADIUS1 ||
        item.layer === Layer.JEWEL_RADIUS2
      ) {
        ctx.globalCompositeOperation = 'lighter'
      } else {
        ctx.globalCompositeOperation = 'source-over'
      }

      // here we draw
      if (item.type === 'background') {
        ctx.fillStyle = this.pattern(image, 'repeat')
        ctx.fillRect(
          -transform.x / transform.scale,
          -transform.y / transform.scale,
          size.width / transform.scale,
          size.height / transform.scale
        )
      } else if (item.type === 'image') {
        const scale = imageScale * (item.scale ?? 1)
        ctx.save()
        ctx.translate(item.x, item.y)
        if (item.angle) ctx.rotate(item.angle)
        const width = item.size ?? rect.w * scale
        const height = item.size ?? rect.h * scale
        const delta = item.centered ? -0.5 : 0
        ctx.drawImage(
          image,
          rect.x,
          rect.y,
          rect.w,
          rect.h,
          delta * width,
          delta * height,
          width,
          height
        )
        ctx.restore()
      } else if (item.type === 'twohalf') {
        const width = rect.w * imageScale
        const height = rect.h * imageScale
        ctx.save()
        ctx.translate(item.x, item.y)
        ctx.drawImage(
          image,
          rect.x,
          rect.y,
          rect.w,
          rect.h,
          -width / 2,
          -height,
          width,
          height
        )
        ctx.scale(1, -1)
        ctx.drawImage(
          image,
          rect.x,
          rect.y,
          rect.w,
          rect.h,
          -width / 2,
          -height,
          width,
          height
        )
        ctx.restore()
      } else if (item.type === 'line') {
        rect = subImage(rect, lineSubImage)
        const dx = item.x2 - item.x1
        const dy = item.y2 - item.y1
        const dist = Math.sqrt(dx * dx + dy * dy)
        ctx.save()
        ctx.transform(
          dx / dist,
          dy / dist,
          -dy / dist,
          dx / dist,
          item.x1,
          item.y1
        )
        const imWidth = rect.w * imageScale
        const imHeight = rect.h * imageScale
        const uScale = dist / imWidth
        for (let start = 0; start < uScale; start += 1) {
          const width = Math.min(1, uScale - start)
          ctx.drawImage(
            image,
            rect.x,
            rect.y,
            rect.w * width,
            rect.h,
            start * imWidth,
            -0.5 * imHeight,
            width * imWidth,
            imHeight
          )
        }
        ctx.restore()
      } else if (item.type === 'arc') {
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

        ctx.save()
        ctx.translate(item.x, item.y)
        ctx.beginPath()
        ctx.arc(0, 0, item.r + 50, a1, a2, false)
        ctx.arc(0, 0, item.r - 50, a2, a1, true)
        ctx.closePath()
        ctx.clip()

        const w = rect.w * imageScale
        const h = rect.h * imageScale
        ctx.rotate(Math.PI + a1)
        while (diff > 0) {
          ctx.drawImage(image, rect.x, rect.y, rect.w, rect.h, -w, -h, w, h)
          ctx.rotate(Math.PI / 2)
          diff -= Math.PI / 2
        }

        ctx.restore()
      }
    }
  }
}
