import { fetchAsset } from '@/data/load'

const decoder = new TextDecoder()

type AtlasJson = Record<
  string,
  { x: number; y: number; w: number; h: number }
> & {
  __background?: boolean
}

export type ImageSource =
  | HTMLImageElement
  | HTMLCanvasElement
  | OffscreenCanvas
  | ImageBitmap

export type GlyphData = {
  image: string
  dx: number
  dy: number
  advance: number
}

export type BundleData = {
  image: ImageSource
  index: AtlasJson
  glyphs?: Record<string, Record<string, GlyphData>>
}[]

const bundleData: Record<string, Promise<BundleData>> = {}

async function loadImage(data: Uint8Array, options?: ImageBitmapOptions) {
  const blob = new Blob([data], { type: 'image/webp' })
  if (window.createImageBitmap) {
    return window.createImageBitmap(blob, options)
  } else {
    return new Promise<HTMLImageElement>((resolve, reject) => {
      const img = new Image()
      const url = URL.createObjectURL(blob)
      img.onload = () => {
        URL.revokeObjectURL(url)
        resolve(img)
      }
      img.onerror = () => {
        URL.revokeObjectURL(url)
        reject()
      }
      img.src = url
    })
  }
}

async function loadBundle(name: string) {
  const buffer = await fetchAsset(`${name}.bundle`).then((r) => r.arrayBuffer())
  const view = new DataView(buffer)
  const count = view.getUint32(0, true)
  const result: BundleData = []
  let pos = 4
  for (let i = 0; i < count; ++i) {
    const sizeImage = view.getUint32(pos, true)
    const sizeIndex = view.getUint32(pos + 4, true)
    pos += 8
    const imageData = new Uint8Array(buffer, pos, sizeImage)
    pos += sizeImage
    const index = JSON.parse(
      decoder.decode(new Uint8Array(buffer, pos, sizeIndex))
    )
    pos += sizeIndex
    result.push({
      image: await loadImage(imageData, {
        premultiplyAlpha: index.__background ? 'none' : 'premultiply'
      }),
      index
    })
  }
  return result
}

export function getBundle(name: string) {
  if (!bundleData[name]) bundleData[name] = loadBundle(name)
  return bundleData[name]
}

//const fontsLoaded = {};

function isFontLoaded(face: string) {
  const div = document.createElement('div')
  div.style.position = 'absolute'
  div.style.visibility = 'hidden'
  div.style.whiteSpace = 'nowrap'
  let prevWidth = 0
  document.body.appendChild(div)
  div.textContent = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'
  for (const fallback of [
    'serif',
    'sans-serif',
    'monospace',
    'cursive',
    'fantasy'
  ]) {
    div.style.fontFamily = `"${face}", ${fallback}`
    const width = div.offsetWidth
    if (prevWidth && width !== prevWidth) {
      document.body.removeChild(div)
      return false
    }
    prevWidth = width
  }
  document.body.removeChild(div)
  return true
}

export function loadFont(face: string, timeout: number = 2000) {
  return new Promise<string>(function (resolve, reject) {
    function check() {
      if (isFontLoaded(face)) {
        resolve(face)
        return
      }
      timeout -= 100
      if (timeout < 0) {
        reject(null)
      } else {
        setTimeout(check, 100)
      }
    }
    check()
  })
}

export async function renderFontBitmap(
  prefix: string,
  fontFace: string,
  size: number,
  charSet: string
): Promise<BundleData | undefined> {
  await loadFont(fontFace)

  //const rect = entry.index[data.background]
  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')!
  ctx.font = `${size}px ${fontFace}`
  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.lineWidth = 1
  ctx.strokeStyle = '#000'
  ctx.fillStyle = '#fff'

  const index: AtlasJson = {}
  const glyphs: Record<string, GlyphData> = {}

  const width = 256
  let x = 1
  let y = 1
  let rowHeight = 0
  for (const chr of charSet) {
    const size = ctx.measureText(chr)
    const chWidth =
      Math.ceil(size.actualBoundingBoxLeft) +
      Math.ceil(size.actualBoundingBoxRight)
    const chHeight =
      Math.ceil(size.actualBoundingBoxAscent) +
      Math.ceil(size.actualBoundingBoxDescent)
    if (x + chWidth >= width) {
      x = 0
      y += rowHeight + 2
      rowHeight = 0
    }

    const name = `${prefix}_${chr}`
    glyphs[chr] = {
      image: name,
      advance: size.width,
      dx: Math.ceil(size.actualBoundingBoxLeft),
      dy: Math.ceil(size.actualBoundingBoxAscent)
    }
    index[name] = {
      x,
      y,
      w: chWidth,
      h: chHeight
    }

    rowHeight = Math.max(rowHeight, chHeight)
    x += chWidth + 2
  }

  let height = 128
  y += rowHeight + 1
  while (height < y) {
    height *= 2
  }

  canvas.width = width
  canvas.height = height
  ctx.font = `${size}px ${fontFace}`
  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.lineWidth = 1
  ctx.strokeStyle = '#000'
  ctx.fillStyle = '#fff'

  for (const [chr, glyph] of Object.entries(glyphs)) {
    const img = index[glyph.image]
    const x = img.x + glyph.dx
    const y = img.y + glyph.dy
    ctx.strokeText(chr, x, y)
    ctx.fillText(chr, x, y)
  }

  const image = window.createImageBitmap
    ? await window.createImageBitmap(canvas)
    : canvas
  return [
    {
      image,
      index,
      glyphs: {
        [prefix]: glyphs
      }
    }
  ]
}

export const imagePath = (path: string) =>
  path.toLowerCase().replace(/\.(png|dds)$/i, '')
export const imagePaths = <T extends Record<string, string>>(data: T) =>
  Object.fromEntries(
    Object.entries(data).map(([key, value]) => [key, imagePath(value)])
  ) as T
export const skillIconPath = (path: string) => imagePath(`Art/2DArt/${path}`)
