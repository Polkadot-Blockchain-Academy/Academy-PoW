"use client";


export default function BlockList({ nodes }) {
    const nodeData = nodes?.map((node, index) => {
        return (
            <div key={index} className="col-span-1 m-4" id={node.hash}>
                <div className={`grid grid-cols-3 ${ node.style }`}>
                    <span className={`col-span-3 text-xl ${node.groupColor}`}>
                        number: {node.number}
                    </span>
                    <span className="col-span-3 text-sm">hash: {node.hash}</span>
                    <span className="col-span-3 text-sm">parent hash: {node.parentHash}</span>
                    <span className="col-span-3 text-sm">state root: {node.stateRoot}</span>
                    <span className="col-span-3 text-sm">
                        extrinsics root: {node.extrinsicsRoot}
                    </span>
                    <div className="h-12 col-span-3 overflow-y-scroll">
                        {node.digestLogs &&
                            node.digestLogs.map((d, j) => {
                                return (
                                    <div key={`${index}-${j}`}>
                                        <span>seal1: {d[0]}</span>
                                        <span>seal2: {d[1]}</span>
                                    </div>
                                );
                            })}
                    </div>
                </div>
            </div>
        );
    });

    return (
        <>
            { nodeData && (
                <div className="grid items-stretch min-h-screen grid-cols-1 mx-6 overflow-y-scroll">
                    {nodeData}
                </div>
            )}
        </>
    );
}

