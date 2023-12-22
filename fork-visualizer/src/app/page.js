"use client";

import { ApiPromise, WsProvider } from "@polkadot/api";
import { useEffect, useState } from "react";
import HorizontalBlockList from "@/app/components/HorizontalBlockList";
import BlockTable from "@/app/components/BlockTable";
import {
    MAX_CHAIN_COUNT,
    WS_ADDRESSES,
    GROUP_TO_COLOR,
    SEAL_TO_GROUP,
    GROUP_TO_NODE_COLOR,
    ENABLE_GRAPH_VIEW,
    ENABLE_TEST_VIEW,
} from "@/constants";
import ForceGraph from "@/app/components/ForceGraph"
import Flow from "@/app/components/Flow"




export default function Home() {

    const [ latestBlock, setLatestBlock ] = useState();
    const [ data, setData ] = useState({
        blocks: [],
        headers: [],
        nodes: [],
        links: [],
        flowNodeMap: [[]],
        flowNodes: [],
        flowEdges: [],
    });
    const [ running, setRunning ] = useState(false);
    const [ wsProviders, setWsProviders ] = useState([])

    useEffect(() => {
        setWsProviders([...WS_ADDRESSES.map((ws_addr) => {
            console.log(`watching ${ws_addr}...`)

            return new WsProvider(ws_addr)
        })])
    }, [])

    const updateBlocks = (header) => {
        setLatestBlock(header.number.toHuman());

        setData(({
            blocks,
            headers,
            nodes,
            links,
            flowNodeMap,
            flowNodes,
            flowEdges
        }) => {
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

            let newBlocks = [...blocks];

            // If the block is not in the graph already, we add it.
            // Actually it would be better to use a set datastructure here than a list,
            // but IDK how convenient / idiomatic sets are in JS.
            if (!blocks.some((h) => h.hash === header.hash.toString())) {
                console.log("Block not found. Adding it to graph.")
                newBlocks = [
                    ...blocks,
                    {
                        hash: header.hash.toHuman(),
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
                ];
            }

            const styledBlocks = newBlocks.map((block) => {
                const duplicate = newBlocks.filter((b) => b.number === block.number && b.hash !== block.hash).length === 0
                return {
                    ...block,
                    style: duplicate ? "" : "bg-rose-900 text-white",
                    duplicate
                };
            });

            let newNodes = [...nodes];

            // If the node is not in the graph already, we add it.
            // Actually it would be better to use a set datastructure here than a list,
            // but IDK how convenient / idiomatic sets are in JS.
            if (!nodes.some((h) => h.id === header.hash.toString())) {
                console.log("Block not found. Adding it to graph.")
                newNodes = [
                    ...nodes,
                    {
                        id: header.hash.toString(),
                        group: group,
                        nodeColor: GROUP_TO_NODE_COLOR[group]
                    }
                ];
            }

            let newLinks = [...links];
            //TODO Check out how I used `.some` above instead of `.filter`.
            // Perhaps we should do that here as well.
            if (newNodes.filter((h) => h.id === header.parentHash.toString()).length > 0) {
                console.log("parent found");
                newLinks = [
                    ...links,
                    {
                        source: header.hash.toString(),
                        target: header.parentHash.toString(),
                    },
                ];
            } else {
                console.log("parent not found");
            }

            // Flow nodes

            let newFlowNodeMap = [...flowNodeMap]
            let newFlowNodes = [...flowNodes];


            function findParentLevel (pHex) {
                let parentLevel = 0

                for (let level = 0; level < newFlowNodeMap.length; level++) {
                    let blocks = newFlowNodeMap[level]

                    if (blocks.length === 0) return parentLevel

                    let foundParent = blocks.filter(b => b.id === pHex).length > 0
                    parentLevel = level
                    if (foundParent) return parentLevel
                }

                return parentLevel + 1
            }

            function addNode (data, newFlowNodeMap) {
                const newData = [...newFlowNodeMap]
                let parentLevel = findParentLevel(data.data.parentHash)

                if (parentLevel >= newFlowNodeMap.length) return [...newFlowNodeMap, [ data ]]

                const levelData = [...newFlowNodeMap[parentLevel]]

                const isFork = levelData.filter(block => block.data.number === data.data.number).length > 0

                if (isFork) {
                    console.log("DATA: ", data)
                    return [...newFlowNodeMap, [ data ]]
                }

                newData[parentLevel] = [...levelData, data]

                return newData
            }

            // If the node is not in the graph already, we add it.
            // Actually it would be better to use a set datastructure here than a list,
            // but IDK how convenient / idiomatic sets are in JS.
            if (!flowNodes.some((h) => h.id === header.hash.toString())) {
                console.log("Block not found. Adding it to graph.")

                const newNodeData = {
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
                    position: { x: 0, y: 0 },
                    sourcePosition: 'right',
                    targetPosition: 'left',
                    style: {
                        background: GROUP_TO_NODE_COLOR[group],
                        color: 'white',
                        width: 100,
                    },
                }

                newFlowNodeMap = [...addNode(newNodeData, newFlowNodeMap)]
            }

            newFlowNodeMap.map((blocks, level) => {
                blocks.map((block, index) => {
                    const parentLevel = findParentLevel(block.data.parentHash)
                    const parent = newFlowNodeMap[parentLevel]?.filter(b => b.id === block.data.parentHash) | undefined

                    block.position = {
                        x: parent !== 0 ? parent.data.x + 200 : index * 200,
                        y: (level * 100) - 50
                    }

                    block.data.x = parent !== 0 ? parent.data.x + 200 : index * 200,
                    block.data.y = (level * 100) - 50

                    newFlowNodes = [...newFlowNodes, block]
                })
            })

            let newEdges = [...flowEdges];
            //TODO Check out how I used `.some` above instead of `.filter`.
            // Perhaps we should do that here as well.
            if (newFlowNodes.filter((h) => h.id === header.parentHash.toHuman()).length > 0) {
                console.log("parent found");
                newEdges = [
                    ...flowEdges,
                    {
                        id: `${header.number.toHuman()}-${header.hash.toHuman()}-${header.parentHash.toHuman()}`,
                        key: `${header.number.toHuman()}-${header.hash.toHuman()}-${header.parentHash.toHuman()}`,
                        source: header.parentHash.toHuman(),
                        target: header.hash.toHuman(),
                        animated: true,
                        style: { stroke: '#fff' },
                    },
                ];
            } else {
                console.log("parent not found");
            }

            return {
                blocks: styledBlocks,
                headers: [...headers, header.toHuman()],
                nodes: newNodes,
                links: newLinks,
                flowNodeMap: newFlowNodeMap,
                flowNodes: newFlowNodes,
                flowEdges: newEdges,
            };
        });
    }

    async function main() {
        // We only display a couple, then unsubscribe
        let count = 0
        setRunning(true)
        setData({
            blocks: [],
            headers: [],
            nodes: [],
            links: [],
            flowNodeMap: [[]],
            flowNodes: [],
            flowEdges: [],
        })

        wsProviders.map(async (wsProvider) => {
            // Subscribe to the new headers on-chain. The callback is fired when new headers
            // are found, the call itself returns a promise with a subscription that can be
            // used to unsubscribe from the newHead subscription
            const api = await ApiPromise.create({ provider: wsProvider })
            const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
                updateBlocks(header)

                if (++count === MAX_CHAIN_COUNT) {
                    unsubscribe();
                    setRunning(false);
                }
            });
        })
    }

    return (
        <div className="flex flex-col justify-center overflow-y-scroll">
            <div className="flex items-center justify-center flex-grow h-screen gap-4 ">
                <div className="flex flex-col justify-center">
                    {!running && (
                        <button className="px-4 py-2 text-white bg-blue-600 rounded-full" onClick={() => main()}>start</button>
                    )}
                </div>

                { ENABLE_TEST_VIEW && data.blocks.length > 0 && (
                    <BlockTable nodes={ data.blocks } latestBlock={ latestBlock } />
                )}

                { ENABLE_GRAPH_VIEW && data.blocks.length > 0 && (
                    <ForceGraph data={ data } />
                )}
            </div>

            { ENABLE_TEST_VIEW && (
                <HorizontalBlockList nodes={ data.blocks } />
            )}

            <div className="flex w-auto h-screen">
                <Flow initialNodes={data.flowNodes} initialEdges={data.flowEdges} />
            </div>

        </div>
    );
}
