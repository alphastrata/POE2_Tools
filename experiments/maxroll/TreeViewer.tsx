import Data from '@/data'
import {
  lightState,
  TreeEditingState,
  TreeProperties,
  treeToggleNode,
  treeValidateHistory,
  treeViewState
} from '@/logic/tree'
import { attributeSkills, NodeData, TreeDataProvider } from '@/logic/treeData'
import Store from '@/store'
import { Rect } from '@/types/common'
import { defaultPlayerClass } from '@/types/profile'
import {
  BaseDragView,
  DragContext,
  DragController,
  DragParameters,
  DragState,
  Transform
} from '@/ui/DragView'
import { ease, easeOut } from '@/ui/easing'
import Popup, { getEventRect } from '@/ui/popup'
import { TooltipAction } from '@/ui/tooltip'
import { PopupMenu } from '@/ui/utils/OverlayManager'
import { notNull } from '@/utils/validation'
import useResizeObserver from '@react-hook/resize-observer'
import React from 'react'
import { TreeRenderer } from './TreeRenderer'

export type TreeHandle = {
  focus: (target: { nodes: number[] } | { search: string }) => void
}

export type TreeProps = {
  tree: TreeProperties
  state: TreeEditingState
  setState: (
    value: TreeEditingState | ((prev: TreeEditingState) => TreeEditingState)
  ) => void
  onClick?: (id: number, e?: React.MouseEvent) => any
  onEditNode?: (id: number, node: NodeData) => void
}

function TreeController({
  tree,
  state,
  setState,
  onClick: onClickHandler,
  onEditNode,
  canvasFallback
}: TreeProps & {
  canvasFallback?: boolean
}) {
  const [menu, setMenu] = React.useState<{
    pos: Rect
    actions: TooltipAction[]
  }>()

  const items = Store.useSelector((store) => store.items)
  const viewState = React.useMemo(
    () => treeViewState(tree, state, items),
    [tree, state, items]
  )

  const mountedRef = React.useRef(true)
  React.useEffect(() => {
    mountedRef.current = true
    return () => {
      mountedRef.current = false
    }
  }, [])
  const nodeActions = React.useCallback(
    (id: number, e?: React.MouseEvent): TooltipAction[] | undefined => {
      if (tree.readOnly) return

      const viewState = treeViewState(tree, lightState(state), items)
      const data = new TreeDataProvider(viewState, items)

      const node = data.node(id)
      if (!node || node.immutable) return

      const actions: TooltipAction[] = []
      actions.push({
        text: node.state ? 'Unallocate' : 'Allocate',
        action: () =>
          setState((state) => treeToggleNode(tree, state, id, items))
      })

      if (!e?.button) {
        if (
          node.skill.is_jewel_socket &&
          (node.state || node.jewel || e?.ctrlKey)
        ) {
          if (onEditNode) {
            actions.push({
              text: node.jewel ? 'Edit Jewel' : 'Add Jewel',
              action: () => onEditNode(id, node)
            })
          }
          if (node.jewel) {
            actions.push({
              text: 'Remove Jewel',
              action: () =>
                setState((state) => {
                  state = { ...state }
                  state.jewels = { ...state.jewels }
                  delete state.jewels[id]

                  const cleanState: TreeEditingState = {
                    history: [],
                    activeSet: 0,
                    position: 0,
                    masteries: state.masteries,
                    jewels: state.jewels,
                    attributes: state.attributes
                  }
                  const position = Math.max(
                    0,
                    Math.min(state.history.length, state.position)
                  )
                  const history1 = treeValidateHistory(
                    tree,
                    cleanState,
                    state.history.slice(0, position),
                    items
                  )
                  cleanState.history = history1
                  cleanState.position = history1.length
                  const history2 = treeValidateHistory(
                    tree,
                    cleanState,
                    state.history.slice(position),
                    items
                  )
                  state.history = [...history1, ...history2]
                  state.position = history1.length
                  return state
                })
            })
          }
        }
        if (node.skill.stats.display_passive_attribute_text) {
          if (!node.state) actions.pop()
          attributeSkills.forEach((skill, index) => {
            actions.push({
              text: Data.passive_skills[skill].name,
              action: () =>
                setState((state) => {
                  if (node.state) {
                    state = { ...state }
                    state.attributes = { ...state.attributes }
                    state.attributes[id] = index
                  } else {
                    state = treeToggleNode(tree, state, id, items, index)
                  }
                  return state
                })
            })
          })
        }
      }
      if (e?.ctrlKey && actions.length > 1) {
        actions.splice(0, 1)
        actions.splice(1)
      }

      return actions.map((action) => ({
        text: action.text,
        action: () => mountedRef.current && action.action()
      }))
    },
    [tree, state, onEditNode, items, setState]
  )

  const onHover = React.useCallback(
    (id: number | undefined) =>
      setState((state) => {
        const hover = id && id > 0 ? id : undefined
        if (hover === state.hover) return state
        return { ...state, hover }
      }),
    [setState]
  )
  const onClick = React.useCallback(
    (id: number, e: React.MouseEvent) => {
      if (id > 0) {
        if (onClickHandler?.(id, e)) return
        const actions = nodeActions(id, e)
        if (actions?.length) {
          if (actions.length > 1) {
            const pos = getEventRect(e)
            setTimeout(
              () => mountedRef.current && setMenu({ pos, actions }),
              10
            )
          } else {
            actions[0].action()
          }
        }
      }
    },
    [onClickHandler, nodeActions]
  )

  return (
    <>
      <TreeRenderer
        state={viewState}
        onHover={onHover}
        onClick={onClick}
        nodeActions={nodeActions}
        canvasFallback={canvasFallback}
      />
      {menu && (
        <Popup alignPosition={menu.pos} onClose={() => setMenu(undefined)}>
          <PopupMenu
            onResolve={(index) => {
              menu.actions[index].action()
              setMenu(undefined)
            }}
            items={menu.actions.map((a) => a.text)}
          />
        </Popup>
      )}
    </>
  )
}

const treeDimensions = {
  left: -12000,
  top: -12000,
  right: 12000,
  bottom: 12000
}

const atlasTreeDimensions = {
  left: -4500,
  top: -6500,
  right: 4500,
  bottom: 2500
}

const wheelZoom = {
  delta: 1.1,
  duration: 0.15,
  timingFunction: ease
}

export const TreeViewer = React.forwardRef(function TreeViewer(
  props: TreeProps,
  ref: React.ForwardedRef<TreeHandle>
) {
  const dragRef = React.useRef<DragState>(null)
  const store = Store.useStore()
  React.useImperativeHandle(
    ref,
    () => ({
      focus(target) {
        if (!dragRef.current) return
        const items = store.getState().items
        const view = treeViewState(
          props.tree,
          {
            ...props.state,
            focus: 'nodes' in target ? target.nodes : undefined,
            search: 'search' in target ? target.search : undefined
          },
          items
        )
        const data = new TreeDataProvider(view, items)
        const nodes = view.highlight?.map((id) => data.node(id)).filter(notNull)
        if (!nodes?.length) return
        const x = nodes.map((node) => node.x)
        const y = nodes.map((node) => node.y)
        const padding = 128 / dragRef.current.transform.scale
        dragRef.current.controller.zoomToFit(
          {
            left: Math.min(...x) - padding,
            top: Math.min(...y) - padding,
            right: Math.max(...x) + padding,
            bottom: Math.max(...y) + padding
          },
          0.5,
          easeOut
        )
      }
    }),
    [props.tree, props.state, store]
  )
  return (
    <div className="poe2-TreeViewer">
      <BaseDragView
        outerClass="poe2-TreeScroll poe2-draggable"
        dimensions={
          'atlas' in props.tree ? atlasTreeDimensions : treeDimensions
        }
        minScale={0}
        maxScale={0.5}
        wheelZoom={wheelZoom}
        ref={dragRef}>
        <TreeController {...props} />
      </BaseDragView>
    </div>
  )
})

export const TreeAscendancyViewer = React.forwardRef(function TreeViewer(
  props: TreeProps,
  ref: React.ForwardedRef<TreeHandle>
) {
  React.useImperativeHandle(
    ref,
    () => ({
      focus(target) {
        // do nothing
      }
    }),
    []
  )

  const containerRef = React.useRef<HTMLDivElement>(null)
  const [containerSize, setContainerSize] = React.useState({
    width: 0,
    height: 0
  })
  React.useLayoutEffect(() => {
    if (containerRef.current) {
      setContainerSize({
        width: containerRef.current.clientWidth,
        height: containerRef.current.clientHeight
      })
    }
  }, [])
  useResizeObserver(containerRef, (entry) => {
    const { inlineSize: width, blockSize: height } = entry.contentBoxSize[0]
    setContainerSize({ width, height })
  })

  const treeSize = 3180
  const treePaddingTop = 100
  const treePaddingBottom = 20

  const transform = React.useMemo<Transform>(() => {
    const scale = Math.min(
      containerSize.width / treeSize,
      containerSize.height / (treeSize + treePaddingTop + treePaddingBottom)
    )
    return {
      scale,
      x: containerSize.width / 2,
      y:
        (containerSize.height -
          (treePaddingTop + treeSize + treePaddingBottom) * scale) /
          2 +
        (treePaddingTop + treeSize / 2) * scale
    }
  }, [treeSize, containerSize])

  const parameters = React.useMemo<DragParameters>(
    () => ({
      dimensions: {
        left: -treeSize / 2,
        top: -treeSize / 2,
        right: treeSize / 2,
        bottom: treeSize / 2
      },
      minScale: 1,
      maxScale: 1,
      cover: true,
      wheelZoom: {
        delta: 1
      }
    }),
    [treeSize]
  )

  const transformRef = React.useRef(transform)
  transformRef.current = transform
  const parameterRef = React.useRef(parameters)
  parameterRef.current = parameters

  const controller = React.useMemo(
    () =>
      new DragController(() => {}, containerRef, parameterRef, transformRef),
    []
  )

  const dragState = React.useMemo<DragState>(
    () => ({
      get container() {
        return containerRef.current ?? undefined
      },
      controller: controller,
      transform,
      size: containerSize
    }),
    [transform, containerSize, controller]
  )

  return (
    <div className="poe2-TreeViewer">
      <div className="poe2-TreeScroll" ref={containerRef}>
        <DragContext.Provider value={dragState}>
          <TreeController {...props} canvasFallback />
        </DragContext.Provider>
      </div>
    </div>
  )
})

export const TreePicker = React.forwardRef(function TreePicker(
  {
    atlas,
    ascendancy,
    search,
    onSelect
  }: {
    atlas?: boolean
    ascendancy?: string
    search?: string
    onSelect: (id: number, e?: React.MouseEvent) => void
  },
  ref: React.ForwardedRef<TreeHandle>
) {
  const props = React.useMemo<TreeProperties>(() => {
    if (atlas) {
      return {
        version: Data.defaultVersion,
        atlas: true,
        readOnly: true
      }
    } else if (ascendancy) {
      return {
        version: Data.defaultVersion,
        charClass: Data.ascendancyClass[ascendancy] ?? defaultPlayerClass,
        ascendancy,
        ascendancyOnly: true,
        readOnly: true
      }
    } else {
      return {
        version: Data.defaultVersion,
        charClass: defaultPlayerClass,
        readOnly: true
      }
    }
  }, [atlas, ascendancy])
  const [state, setState] = React.useState<TreeEditingState>(() => {
    return {
      history: [],
      activeSet: 0,
      position: 0,
      masteries: {},
      jewels: {},
      attributes: {},
      search
    }
  })
  React.useEffect(() => setState((state) => ({ ...state, search })), [search])
  if (!atlas && ascendancy) {
    return (
      <TreeAscendancyViewer
        tree={props}
        state={state}
        setState={setState}
        onClick={onSelect}
        ref={ref}
      />
    )
  } else {
    return (
      <TreeViewer
        tree={props}
        state={state}
        setState={setState}
        onClick={onSelect}
        ref={ref}
      />
    )
  }
})
