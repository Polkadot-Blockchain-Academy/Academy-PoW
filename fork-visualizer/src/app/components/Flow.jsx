"use client"

import { useEffect } from "react"
import ReactFlow, { useNodesState, useEdgesState, Controls } from 'reactflow';
import 'reactflow/dist/style.css';


import '@/index.css';


const connectionLineStyle = { stroke: '#fff' };
const snapGrid = [20, 20];

const defaultViewport = { x: 0, y: 0, zoom: 10.5 };

const Flow = ({ initialNodes, initialEdges }) => {
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState([]);

    useEffect(() => {
        setNodes(initialNodes);
    }, [initialNodes, setNodes]);


    useEffect(() => {
        setEdges(initialEdges);
    }, [initialEdges, setEdges]);

    return (
        <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
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

export default Flow;
