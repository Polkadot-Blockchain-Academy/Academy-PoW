"use client";

import { ApiPromise, WsProvider } from "@polkadot/api";
import { useState } from "react";

const MAX_CHAIN_COUNT = 256;

// // Construct
const wsProvider = new WsProvider("wss://rpc.polkadot.io");

export default function Home() {
    const [latestBlock, setLatestBlock] = useState();
    const [data, setData] = useState({ nodes: [] });
    const [running, setRunning] = useState(false);

    async function main() {
        const api = await ApiPromise.create({ provider: wsProvider });

        // We only display a couple, then unsubscribe
        let count = 0;
        setRunning(true);
        setData({ nodes: [] });

        // Subscribe to the new headers on-chain. The callback is fired when new headers
        // are found, the call itself returns a promise with a subscription that can be
        // used to unsubscribe from the newHead subscription
        const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
            setLatestBlock(header.number.toHuman());

            setData(({ nodes }) => {
                const newNodes = [
                    ...nodes,
                    {
                        hash: header.hash.toHuman(),
                        parentHash: header.parentHash.toHuman(),
                        number: header.number.toHuman(),
                        stateRoot: header.stateRoot.toHuman(),
                        extrinsicsRoot: header.extrinsicsRoot.toHuman(),
                    },
                ];

                const styledNodes = newNodes.map((node, i) => {
                    return {
                        ...node,
                        style:
                            newNodes.filter((n) => n.number === node.number && n.hash !== node.hash)
                                .length === 0
                                ? ""
                                : "bg-red-600",
                    };
                });

                return {
                    nodes: styledNodes,
                };
            });

            if (++count === MAX_CHAIN_COUNT) {
                unsubscribe();
                setRunning(false);
            }
        });
    }

    const { nodes } = data;
    const nodeData = nodes?.map((node, index) => {
        return (
            <div key={index} className="col-span-1 m-4" id={node.hash}>
                <div className="grid grid-cols-3">
                    <span className={`col-span-3 text-xl ${node.style}`}>
                        number: {node.number}
                    </span>
                    <span className="col-span-3 text-sm">hash: {node.hash}</span>
                    <span className="col-span-3 text-sm">parent hash: {node.parentHash}</span>
                    <span className="col-span-3 text-sm">state root: {node.stateRoot}</span>
                    <span className="col-span-3 text-sm">
                        extrinsics root: {node.extrinsicsRoot}
                    </span>
                    <div className="h-10 col-span-3 overflow-y-scroll">
                        {node.mappedValidators &&
                            node.mappedValidators.map((v, j) => {
                                return (
                                    <div key={j}>
                                        <span>address: {v.address}</span>
                                        <span>balance: {v.balance}</span>
                                        <span>nonce: {v.nonce}</span>
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
            {latestBlock && <h1>latest block: {latestBlock} </h1>}
            {!running && (
                <div>
                    <button onClick={() => main()}>start</button>
                </div>
            )}

            <div className="grid items-stretch min-h-screen grid-cols-1 mx-6 overflow-y-scroll">
                {nodeData}
            </div>
        </>
    );
}
