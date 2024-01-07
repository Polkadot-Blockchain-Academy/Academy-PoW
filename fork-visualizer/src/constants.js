export const ENABLE_GRAPH_VIEW = false
export const ENABLE_TEST_VIEW = false

// used for the info overlay for latest block number (not working)
export const ENABLE_BLOCK_COUNTER = false
// used for the info overlay for block info (not working)
export const ENABLE_BLOCK_LIST = false
export const ENABLE_MINIMAP = false

export const MAX_CHAIN_COUNT = 100
export const AUTO_SCROLL_DEFAULT = false

export const INITIAL_WS_ADDRESS = "ws://127.0.0.1:9944"

export const SEAL_TO_GROUP = {
    "0x00": "md5",
    "0x01": "sha3",
    "0x02": "keccak",
}

export const GROUP_TO_NODE_COLOR = {
    "genesis": "#00ff00",
    "md5": "#ff0000",
    "sha3": "#0000ff",
    "keccak": "#ff00ff",
}


// a lot of variables used for the map
export const NODE_WIDTH = 1500
export const NODE_HEIGHT = 500
export const DEFAULT_DIRECTION = 'LR'
export const DEFAULT_POSITION = { x: 0, y: 0 }
export const DEFAULT_EDGE_TYPE = 'smoothstep'
export const DEFAULT_CONNECTION_LINE_STYLE = { stroke: '#fff' }
export const DEFAULT_SNAP_GRID = [20, 20]
export const DEFAULT_VIEWPORT = { x: 0, y: 0, zoom: .4 }
export const DEFAULT_MIN_ZOOM = .2
