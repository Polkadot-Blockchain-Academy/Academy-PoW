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
    INITIAL_WS_ADDRESS,
} from '@/constants';

import CustomBlockNode from '@/app/components/CustomBlockNode';
import NodeWsInput from '@/app/components/NodeWsInput'
import NodeWsController from '@/app/components/NodeWsController'

const nodeTypes = {
    custom: CustomBlockNode,
};

const initialWsAddress = process.env.NEXT_PUBLIC_INITIAL_WS_ADDRESS || INITIAL_WS_ADDRESS

const getLayoutedElements = (nodes, edges, direction = DEFAULT_DIRECTION) => {
    const dagreGraph = new dagre.graphlib.Graph();
    dagreGraph.setDefaultEdgeLabel(() => ({}));

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

    // I am using setData, but not data right now.
    // There is probably a better way, just have not had time to make it better
    // eslint-disable-next-line
    const [ data, setData ] = useState({
        nodes: [],
        edges: [],
    })
    const [ wsAddresses, setWsAddresses ] = useState([initialWsAddress])

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
            // The genesis block (number 0) does not have the normal PoW seal on it.
            // This avoids a crash when you re-start the block. There is likely a more
            // idiomatic way to do this is js.
            if (header.number != 0) {
                const seal_data = header.digest.logs[0].toJSON().preRuntime[1];
                const seal = seal_data.slice(0, 4)

                group = SEAL_TO_GROUP[seal] ?? "genesis"
            }

            let authorAccount = undefined
            if (group != "genesis") {
                const apiAt = await api.at(header.hash)
                const aAccount = await apiAt.query.blockAuthor.author()

                authorAccount = aAccount.toHuman()
            }

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
                            digestLogs: header.digest.logs.map((d) => d.toHuman().Seal),
                            group,
                            duplicate: false,
                            authorAccount,
                            reportingNode,
                        },
                        position: DEFAULT_POSITION,
                        sourcePosition: 'right',
                        targetPosition: 'left',
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
                    edges: newLayoutedEdges,
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

            <NodeWsInput
                addNode={addNode}
            >
                { wsAddresses && (
                    <div className="flex flex-col text-white">
                        <table className="content-center justify-center text-center table-auto">
                            <thead>
                                <tr>
                                    <th>Remove</th>
                                    <th>Websocket Address</th>
                                    <th>Subscribe</th>
                                    <th>Best Block Hash</th>
                                </tr>
                            </thead>
                            <tbody>
                                { wsAddresses.map((wsAddress, index) => {
                                    return (
                                        <NodeWsController
                                            key={index}
                                            wsAddress={wsAddress}
                                            updateStuff={updateStuff}
                                            removeNode={removeNode}
                                        />
                                    )
                                })}
                            </tbody>
                        </table>
                    </div>
                )}
            </NodeWsInput>
        </>
    );
};

export default LayoutFlow;
