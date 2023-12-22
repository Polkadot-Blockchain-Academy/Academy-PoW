export const ENABLE_GRAPH_VIEW = true
export const ENABLE_TEST_VIEW = false

export const MAX_CHAIN_COUNT = 256
export const AUTO_SCROLL_DEFAULT = false

export const WS_ADDRESSES = [
    "ws://localhost:9944",
    "ws://localhost:8844",
    "ws://localhost:7744",
]

export const SEAL_TO_GROUP = {
    "0x00": "md5",
    "0x01": "sha3",
    "0x02": "keccak",
}

export const GROUP_TO_COLOR = {
    "genesis": "bg-green-600",
    "md5": "bg-red-600",
    "sha3": "bg-blue-600",
    "keccak": "bg-purple-600",
}

export const GROUP_TO_NODE_COLOR = {
    "genesis": "#00ff00",
    "md5": "#ff0000",
    "sha3": "#0000ff",
    "keccak": "#ff00ff",
}
