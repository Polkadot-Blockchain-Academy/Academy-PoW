"use client"

import dagre from 'dagre';
import React, { useCallback, useState } from 'react';
import ReactFlow, {
    ConnectionLineType,
    useNodesState,
    useEdgesState,
    Controls,
    MiniMap,
} from 'reactflow';

import 'reactflow/dist/style.css';
import '@/index.css';

import {
    ENABLE_BLOCK_COUNTER,
    ENABLE_BLOCK_LIST,

    GROUP_TO_COLOR,
    GROUP_TO_NODE_COLOR,
    SEAL_TO_GROUP,

    NODE_WIDTH,
    NODE_HEIGHT,
    DEFAULT_DIRECTION,
    DEFAULT_EDGE_TYPE,
    DEFAULT_POSITION,
    DEFAULT_CONNECTION_LINE_STYLE,
    DEFAULT_SNAP_GRID,
    DEFAULT_VIEWPORT,
    ENABLE_MINIMAP,
    DEFAULT_MIN_ZOOM,

    INITIAL_WS_ADDRESSES,
} from '@/constants';

import CustomBlockNode from '@/app/components/CustomBlockNode';
import BlockCounter from '@/app/components/BlockCounter'
import BlockTracker from '@/app/components/BlockTracker';
import NodeTracker, { NodeState } from '@/app/components/NodeTracker';

const nodeTypes = {
    custom: CustomBlockNode,
};


const dagreGraph = new dagre.graphlib.Graph();
dagreGraph.setDefaultEdgeLabel(() => ({}));


const getLayoutedElements = (nodes, edges, direction = DEFAULT_DIRECTION) => {
    const isHorizontal = direction === 'LR';
    dagreGraph.setGraph({ rankdir: direction });

    nodes.forEach((node) => {
        dagreGraph.setNode(node.id, { width: NODE_WIDTH, height: NODE_HEIGHT });
    });

    edges.forEach((edge) => {
        dagreGraph.setEdge(edge.source, edge.target);
    });

    dagre.layout(dagreGraph);

    nodes.forEach((node) => {
        const nodeWithPosition = dagreGraph.node(node.id);
        node.targetPosition = isHorizontal ? 'left' : 'top';
        node.sourcePosition = isHorizontal ? 'right' : 'bottom';

        // We are shifting the dagre node position (anchor=center center) to the top left
        // so it matches the React Flow node anchor point (top left).
        node.position = {
            x: nodeWithPosition.x - NODE_WIDTH / 2,
            y: nodeWithPosition.y - NODE_HEIGHT / 2,
        };

        return node;
    });

    return { nodes, edges };
};

const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedElements([], []);

const LayoutFlow = () => {
    const [nodes, setNodes, onNodesChange] = useNodesState(layoutedNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(layoutedEdges);
    const [ latestBlockNumber, setLatestBlockNumber ] = useState(0)
    const [ data, setData ] = useState({
        nodes: [],
        edges: []
    })
    const [ wsAddresses, setWsAddresses ] = useState([...INITIAL_WS_ADDRESSES])

    const removeNode = (nodeAddress) => {
        const newAddresses = [...wsAddresses]

        const index = newAddresses.indexOf(nodeAddress);
        if (index > -1) {
            newAddresses.splice(index, 1)
        }

        setWsAddresses([...newAddresses])
    }

    const updateStuff = useCallback(
        async (header, api, reportingNode) => {
            let group = "genesis";
            let groupColor = GROUP_TO_COLOR[group]
            // The genesis block (number 0) does not have the normal PoW seal on it.
            // This avoids a crash when you re-start the block. There is likely a more
            // idiomatic way to do this is js.
            if (header.number != 0) {
                const seal_data = header.digest.logs[0].toJSON().seal[1];
                const seal = seal_data.slice(0, 4)

                group = SEAL_TO_GROUP[seal] ?? "genesis"
                groupColor = GROUP_TO_COLOR[group]
            }


            let authorAccount = undefined
            if (group != "genesis") {
                const apiAt = await api.at(header.hash)
                const aAccount = await apiAt.query.blockAuthor.author()

                authorAccount = aAccount.toHuman()
            }

            // update block counter
            setLatestBlockNumber(oldBlockNumber => {
                const newNumber = header.number.toHuman()
                if (newNumber > oldBlockNumber) return newNumber

                return oldBlockNumber
            })

            setData(({ nodes: n, edges: e }) => {
                // If the node is not in the graph already, we add it.
                // Actually it would be better to use a set datastructure here than a list,
                // but IDK how convenient / idiomatic sets are in JS.
                let newNodeData = undefined
                if (!n.some((h) => h.id === header.hash.toString())) {
                    console.log("Block not found. Adding it to graph.")

                    newNodeData = {
                        id: header.hash.toHuman(),
                        key: header.hash.toHuman(),
                        data: {
                            label: header.hash.toHuman(),
                            parentHash: header.parentHash.toHuman(),
                            number: header.number.toHuman(),
                            stateRoot: header.stateRoot.toHuman(),
                            extrinsicsRoot: header.extrinsicsRoot.toHuman(),
                            digestLogs: header.digest.logs.map((d) => {
                                return d.toHuman().Seal
                            }),
                            group: group,
                            groupColor: groupColor,
                            duplicate: false,
                            authorAccount: authorAccount,
                            reportingNode: reportingNode
                        },
                        position: DEFAULT_POSITION,
                        sourcePosition: 'right',
                        targetPosition: 'left',
                        // style: {
                        //     background: GROUP_TO_NODE_COLOR[group],
                        //     color: 'white',
                        //     width: 5000
                        // },
                        type: 'custom',
                    }
                }

                let newEdge = undefined
                const edgeId = `${header.number.toHuman()}-${header.hash.toHuman()}-${header.parentHash.toHuman()}`
                const edgeExists = e.some((_edge) => _edge.id === edgeId)
                const parentExists = n.filter((h) => h.id === header.parentHash.toHuman()).length > 0

                if (!edgeExists && parentExists) {
                    newEdge = {
                        id: edgeId,
                        source: header.parentHash.toHuman(),
                        target: header.hash.toHuman(),
                        animated: true,
                        type: DEFAULT_EDGE_TYPE,
                        style: { stroke: '#fff' },
                    }
                }

                const newNodes = newNodeData ? [...n, newNodeData] : [...n]
                const newEdges = newEdge ? [...e, newEdge] : [...e]

                const { nodes: newLayoutedNodes, edges: newLayoutedEdges } = getLayoutedElements(
                    newNodes,
                    newEdges
                );

                setNodes([...newLayoutedNodes])
                setEdges([...newLayoutedEdges])

                return {
                    nodes: newLayoutedNodes,
                    edges: newLayoutedEdges
                }
            });

        },
        [ setNodes, setEdges ]
    );

    const addNode = async (nodeAddress) => {
        if (nodeAddress !== "") setWsAddresses([...wsAddresses, nodeAddress])
    }

    return (
        <>
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                connectionLineType={ConnectionLineType.SmoothStep}
                fitView

                nodeTypes={nodeTypes}

                connectionLineStyle={DEFAULT_CONNECTION_LINE_STYLE}
                snapToGrid={true}
                snapGrid={DEFAULT_SNAP_GRID}
                defaultViewport={DEFAULT_VIEWPORT}
                attributionPosition="bottom-left"
                style={{ background: '#1A192B' }}

                minZoom={DEFAULT_MIN_ZOOM}

                nodesDraggable={false}
                nodesConnectable={false}
                nodesFocusable={false}
                edgesFocusable={false}
                elementsSelectable={false}
                autoPanOnConnect={false}
                autoPanOnNodeDrag={false}
                panOnDrag={true}
                panOnScroll={true}
                panOnScrollSpeed={2}
                panOnScrollMode={"horizontal"}
            >
                { ENABLE_MINIMAP && (
                    <MiniMap
                        nodeStrokeColor={n => GROUP_TO_NODE_COLOR[n.data.group]}
                        nodeColor={n => GROUP_TO_NODE_COLOR[n.data.group]}
                        zoomable={true}
                        pannable={true}
                    />
                )}
                <Controls />
            </ReactFlow>
            { ENABLE_BLOCK_COUNTER && (
                <BlockCounter latestBlockNumber={ latestBlockNumber } />
            )}

            { ENABLE_BLOCK_LIST && (
                <BlockTracker blocks={ data.nodes }/>
            )}

            <NodeTracker
                addNode={addNode}
            >
                { wsAddresses && (
                    <div className="flex flex-col text-white">
                        { wsAddresses.map((wsAddress, index) => {
                            return (
                                <NodeState
                                    key={index}
                                    wsAddress={wsAddress}
                                    updateStuff={updateStuff}
                                    removeNode={removeNode}
                                />
                            )
                        })}
                    </div>
                )}
            </NodeTracker>
        </>
    );
};

export default LayoutFlow;
