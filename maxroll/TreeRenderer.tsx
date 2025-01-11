import { DragContext, DragState } from '@/ui/DragView'
import React from 'react'
import { loadTree, renderTree } from './renderItems'
import { Renderer } from './renderer'

import { ItemTooltip, PassiveSkillTooltip } from '@/tooltip'
import { ManualTooltip, TooltipAction } from '@/ui/tooltip'

import { TreeViewState } from '@/logic/tree'
import Store, { StoreState } from '@/store'
import { TreeDataProvider } from '../logic/treeData'
import { CanvasRenderer } from './canvasRenderer'
import { ErrorRenderer } from './errorRenderer'
import { WebGLRenderer } from './glRenderer'
import './viewer.scss'

function VirtualNode({
  data,
  id,
  onClose,
  touch,
  nodeActions
}: {
  data: TreeDataProvider
  id: number
  onClose: () => void
  touch: boolean
  nodeActions?: (id: number) => TooltipAction[] | undefined
}) {
  const node = data.node(id)
  const { transform } = React.useContext(DragContext)

  const actions = React.useMemo(() => nodeActions?.(id), [id, nodeActions])

  const ref = React.useRef<HTMLDivElement>(null)

  if (!node) return

  const style = {
    left: node.x * transform.scale + transform.x,
    top: node.y * transform.scale + transform.y,
    fontSize: 256 * transform.scale
  }

  return (
    <React.Fragment>
      <ManualTooltip
        nodeRef={ref}
        touch={touch}
        onClose={onClose}
        actions={actions}
        lockKey="alt"
        tooltip={
          <div className="poe2-CompositeTooltip">
            {node.jewel && <ItemTooltip item={node.jewel} />}
            <PassiveSkillTooltip
              skill={data.nodeSkill(id)!}
              atlas={data.atlas}
            />
          </div>
        }
      />
      <div className="poe2-virtual-node" style={style} ref={ref} />
    </React.Fragment>
  )
}

function useDelay(duration: number) {
  const [done, setDone] = React.useState(false)
  const durationRef = React.useRef(duration)
  React.useEffect(() => {
    setTimeout(() => {
      if (durationRef.current >= 0) setDone(true)
    }, durationRef.current)
    return () => {
      durationRef.current = -1
    }
  }, [])
  return done
}

export function TreeRenderer({
  state,
  canvasFallback,
  onHover,
  onClick,
  nodeActions
}: {
  state: TreeViewState
  canvasFallback?: boolean
  onHover?: (id: number | undefined) => void
  onClick?: (id: number, e: React.MouseEvent) => void
  nodeActions?: (id: number) => TooltipAction[] | undefined
}) {
  const [touch, setTouch] = React.useState(false)
  const dragState = React.useContext(DragContext)
  const ref = React.useRef<HTMLCanvasElement>(null)
  const [contextLost, setContextLost] = React.useState(false)
  const [display, setDisplay] = React.useState<TreeDataProvider>()
  const items = Store.useSelector((store) => store.items)

  React.useEffect(() => {
    if (!ref.current) return
    ref.current.addEventListener('webglcontextlost', () => setContextLost(true))
    ref.current.addEventListener('webglcontextrestored', () =>
      setContextLost(false)
    )
  }, [])

  const renderer = React.useRef<Renderer>()
  const dragStateRef = React.useRef<DragState>()
  const treeStateRef = React.useRef<{
    state: TreeViewState
    items: StoreState['items']
  }>()
  const displayRef = React.useRef<{
    data: TreeDataProvider
  }>()
  React.useLayoutEffect(() => {
    if (ref.current && !contextLost) {
      if (!renderer.current || renderer.current.canvas !== ref.current) {
        if (renderer.current) renderer.current.destroy()
        try {
          renderer.current = new WebGLRenderer(ref.current, dragState)
        } catch (e) {
          if (canvasFallback) {
            renderer.current = new CanvasRenderer(ref.current, dragState)
          } else {
            renderer.current = new ErrorRenderer(ref.current, dragState)
          }
        }
        dragStateRef.current = undefined
        treeStateRef.current = undefined
      }
      if (
        state !== treeStateRef.current?.state ||
        items !== treeStateRef.current?.items
      ) {
        const loader =
          renderer.current instanceof ErrorRenderer
            ? undefined
            : loadTree(state, renderer.current)
        treeStateRef.current = { state, items }
        if (loader) {
          setDisplay(undefined)
          loader.then(() => {
            if (
              treeStateRef.current?.state === state &&
              treeStateRef.current?.items === items
            )
              setDisplay(new TreeDataProvider(state, items))
          })
        } else {
          setDisplay(new TreeDataProvider(state, items))
        }
      }
      if (display && display !== displayRef.current?.data) {
        const items = renderTree(display)
        renderer.current.setItems(items)
        displayRef.current = {
          data: display
        }
      }
      if (dragState !== dragStateRef.current) {
        renderer.current.update(dragState)
        dragStateRef.current = dragState
      }
      renderer.current.render()
    } else if (renderer.current) {
      renderer.current.destroy()
      renderer.current = undefined
    }
  }, [state, display, contextLost, dragState, items, canvasFallback])
  React.useEffect(() => {
    return () => {
      treeStateRef.current = undefined
    }
  }, [])

  const onMouseMove = React.useCallback(
    (e: React.PointerEvent) => {
      const rect = ref.current!.getBoundingClientRect()
      const x = e.clientX - rect.left
      const y = e.clientY - rect.top
      const key = renderer.current?.locate(x, y)
      onHover?.(key)
      setTouch(e.pointerType !== 'mouse')
    },
    [onHover]
  )
  const onMouseLeave = React.useCallback(() => {
    onHover?.(undefined)
  }, [onHover])
  const onMouseUp = React.useCallback(
    (e: React.PointerEvent) => {
      const rect = ref.current!.getBoundingClientRect()
      const x = e.clientX - rect.left
      const y = e.clientY - rect.top
      const key = renderer.current?.locate(x, y)
      if (e.shiftKey && e.button === 2) return
      if (key) onClick?.(key, e)
    },
    [onClick]
  )
  const onContextMenu = React.useCallback((e: React.MouseEvent) => {
    if (!e.shiftKey) e.preventDefault()
  }, [])

  return (
    <div
      className="poe2-outer"
      onPointerMove={onMouseMove}
      onPointerLeave={onMouseLeave}
      onPointerUp={onMouseUp}
      onContextMenu={onContextMenu}>
      <canvas ref={ref} />
      {display ? (
        <div className="poe2-inner">
          {display.state.hover != null ? (
            <VirtualNode
              data={display}
              id={display.state.hover}
              onClose={onMouseLeave}
              touch={touch}
              nodeActions={nodeActions}
            />
          ) : undefined}
        </div>
      ) : (
        <div className="poe2-loading">
          <div className="poe2-text">Loading...</div>
        </div>
      )}
    </div>
  )
}
