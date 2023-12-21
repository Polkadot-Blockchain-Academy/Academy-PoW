"use client"

import { useRef } from "react";
import { ForceGraph2D } from "react-force-graph";


export default function ForceGraph({ data }) {
    const fgRef = useRef();

    return (
        <div className="flex-grow">
            <ForceGraph2D
                graphData={data}
                nodeLabel="id"
                nodeAutoColorBy="group"
                linkDirectionalArrowLength={3.5}
                linkDirectionalArrowRelPos={1}
                ref={fgRef}
                enableNodeDrag={true}
                cooldownTicks={100}
                onEngineStop={() => fgRef.current.zoomToFit(500)}
            />
        </div>
    );
}
