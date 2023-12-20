"use client";

import { ApiPromise, WsProvider } from "@polkadot/api";
import { useState, useRef } from "react";
import { ForceGraph2D, ForceGraph3D } from "react-force-graph";

const MAX_CHAIN_COUNT = 256;

// use polkadot main
// comment out for local
// const wsProvider = new WsProvider("wss://rpc.polkadot.io");


function OtherGraph({ data }) {
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

export default function Home() {
    const wsProvider1 = new WsProvider("ws://100.109.138.126:8844");
    const wsProvider2 = new WsProvider("ws://100.109.138.126:7744");
    const wsProvider3 = new WsProvider("ws://100.109.138.126:6644");

    const [latestBlock, setLatestBlock] = useState();
    const [data, setData] = useState({ nodes: [], links: [] });
    const [running, setRunning] = useState(false);

    async function main() {
        start_watch(wsProvider1)
        start_watch(wsProvider2)
        start_watch(wsProvider3)
    }
    async function start_watch(wsProvider) {
        console.log("Starting...")

        const api = await ApiPromise.create({ provider: wsProvider });

        // We only display a couple, then unsubscribe
        let count = 0;
        setRunning(true);
        setData({ nodes: [], links: [] });

        // Subscribe to the new headers on-chain. The callback is fired when new headers
        // are found, the call itself returns a promise with a subscription that can be
        // used to unsubscribe from the newHead subscription
        const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
            setLatestBlock(header.number.toHuman());

            setData(({ nodes, links }) => {

                let group = "genesis";

                // The genesis block (number 0) does not have the normal PoW seal on it.
                // This avoids a crash when you re-start the node. There is likely a more
                // idiomatic way to do this is js.
                if (header.number != 0) {
                    const seal_data = header.digest.logs[0].toJSON().seal[1];
                    // Detect the PoW algorithm from the first byte of the seal.
                    // This corresponds to `pub struct SupportedHashes` in multi-pow/src/lib.rs
                    switch (seal_data.slice(0, 4)) {
                        case "0x00":
                            group = "md5";
                            break;
                        case "0x01":
                            group = "sha3";
                            break;
                        case "0x02":
                            group = "keccak";
                    }

                    console.log(`group: ${group}`);
                }

               let newNodes = [...nodes];

                // If the node is not in the graph already, we add it.
                // Actually it would be better to use a set datastructure here than a list,
                // but IDK how convenient / idiomatic sets are in JS.
                if (!nodes.some((h) => h.id === header.hash.toString())) {
                    console.log("Block not found. Adding it to graph.")
                    newNodes = [...nodes, { id: header.hash.toString(), group: group }];
                }

                let newLinks = [...links];
                //TODO Check out how I used `.some` above instead of `.filter`.
                // Perhaps we should do that here as well.
                if (newNodes.filter((h) => h.id === header.parentHash.toString()).length > 0) {
                    console.log("parent found");
                    newLinks = [
                        ...links,
                        { source: header.parentHash.toString(), target: header.hash.toString() },
                    ];
                } else {
                    console.log("parent not found");
                }

                return {
                    nodes: newNodes,
                    links: newLinks,
                };
            });

            if (++count === MAX_CHAIN_COUNT) {
                unsubscribe();
                setRunning(false);
            }
        });
    }

    return (
        <>
            {latestBlock && <h1>latest block: {latestBlock} </h1>}
            {!running && (
                <div>
                    <button onClick={() => main()}>Watch Chain</button>
                </div>
            )}

            {data && (
                <div className="flex items-stretch min-h-screen mx-6">
                    <OtherGraph data={data} />
                </div>
            )}
        </>
    );
}
