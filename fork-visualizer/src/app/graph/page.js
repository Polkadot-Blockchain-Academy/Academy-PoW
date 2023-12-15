"use client";

import { ApiPromise, WsProvider } from "@polkadot/api";
import { useState, useRef } from "react";
import { ForceGraph2D, ForceGraph3D } from "react-force-graph";

const MAX_CHAIN_COUNT = 256;

// use polkadot main
// comment out for local
// const wsProvider = new WsProvider("wss://rpc.polkadot.io");

const wsProvider = new WsProvider("ws://localhost:9944");

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
    const [latestBlock, setLatestBlock] = useState();
    const [data, setData] = useState({ nodes: [], links: [] });
    const [running, setRunning] = useState(false);

    async function main() {
        console.log("Starting...")

        // for polkadot main
        const api = await ApiPromise.create({ provider: wsProvider });

        // for local
        // const api = await ApiPromise.create();

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
                const seals = header.digest.logs[0].toHuman().Seal

                // TODO: pull the correct group from the seal
                const group = seals[1].slice(196,204)
                console.log(group, seals[1])

                const newNodes = [...nodes, { id: header.hash.toString(), group: group }];

                let newLinks = [...links];
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
