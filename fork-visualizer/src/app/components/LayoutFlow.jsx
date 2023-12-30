"use client"

import React, { useCallback, useEffect, useState } from 'react';
import ReactFlow, {
    ConnectionLineType,
    useNodesState,
    useEdgesState,
    Controls,
} from 'reactflow';
import { ApiPromise, WsProvider } from "@polkadot/api";
import dagre from 'dagre';
import 'reactflow/dist/style.css';
import '@/index.css';

import {
    WS_ADDRESSES,
    GROUP_TO_COLOR,
    GROUP_TO_NODE_COLOR,
    SEAL_TO_GROUP,
    MAX_CHAIN_COUNT,

    NODE_WIDTH,
    NODE_HEIGHT,
    DEFAULT_DIRECTION,
    DEFAULT_EDGE_TYPE,
    DEFAULT_POSITION,
} from '@/constants';

const initialNodes = []
const initialEdges = []

const connectionLineStyle = { stroke: '#fff' };
const snapGrid = [20, 20];

const defaultViewport = { x: 0, y: 0, zoom: 10.5 };

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

const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedElements(
    initialNodes,
    initialEdges
);

const LayoutFlow = () => {
    const [nodes, setNodes, onNodesChange] = useNodesState(layoutedNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(layoutedEdges);
    const [ data, setData ] = useState({
        nodes: [],
        edges: []
    })


    const updateStuff = useCallback(
        (header) => {
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
                console.log(`group: ${group}`);
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
                            label: header.number.toHuman(),
                            x: 0,
                            y: 0,
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
                        },
                        position: DEFAULT_POSITION,
                        sourcePosition: 'right',
                        targetPosition: 'left',
                        style: {
                            background: GROUP_TO_NODE_COLOR[group],
                            color: 'white',
                            width: 100,
                        },
                    }
                }

                let newEdge = undefined
                if (n.filter((h) => h.id === header.parentHash.toHuman()).length > 0) {
                    newEdge = {
                        id: `${header.number.toHuman()}-${header.hash.toHuman()}-${header.parentHash.toHuman()}`,
                        key: `${header.number.toHuman()}-${header.hash.toHuman()}-${header.parentHash.toHuman()}`,
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













    useEffect(() => {
        // We only display a couple, then unsubscribe
        let count = 0

        WS_ADDRESSES.map(async (ws_addr) => {
            const wsProvider = new WsProvider(ws_addr)
            // Subscribe to the new headers on-chain. The callback is fired when new headers
            // are found, the call itself returns a promise with a subscription that can be
            // used to unsubscribe from the newHead subscription
            const api = await ApiPromise.create({ provider: wsProvider })
            const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
                console.log("adding new header...")

                updateStuff(header)

                if (++count === MAX_CHAIN_COUNT) {
                    unsubscribe();
                }
            });
        })
    }, [ updateStuff ])


    console.log(data)

    return (
        <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            connectionLineType={ConnectionLineType.SmoothStep}

            connectionLineStyle={connectionLineStyle}
            snapToGrid={true}
            snapGrid={snapGrid}
            defaultViewport={defaultViewport}
            fitView
            attributionPosition="bottom-left"
            style={{ background: '#1A192B' }}

            nodesDraggable={false}
            nodesConnectable={false}
            nodesFocusable={false}
            edgesFocusable={false}
            elementsSelectable={false}
            autoPanOnConnect={false}
            autoPanOnNodeDrag={false}
            panOnDrag={false}
            panOnScroll={true}
            panOnScrollSpeed={2}
            panOnScrollMode={"horizontal"}
        >
            <Controls />
        </ReactFlow>
    );
};

export default LayoutFlow;
