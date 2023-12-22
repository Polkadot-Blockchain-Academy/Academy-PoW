"use client";

import { ApiPromise, WsProvider } from "@polkadot/api";
import { useEffect, useState } from "react";
import HorizontalBlockList from "@/app/components/HorizontalBlockList";
import BlockTable from "@/app/components/BlockTable";
import { MAX_CHAIN_COUNT, WS_ADDRESS, GROUP_TO_COLOR, SEAL_TO_GROUP } from "@/constants";


const wsProvider = new WsProvider(WS_ADDRESS)


export default function Home() {

    const [ latestBlock, setLatestBlock ] = useState();
    const [ data, setData ] = useState({ nodes: [] });
    const [ running, setRunning ] = useState(false);

    const [ headers, setHeaders ] = useState([])
    const [ blocks, setBlocks ] = useState([])


    useEffect(() => {

    }, [headers])

    const updateHeaders = (header, oldHeaders) => {

        const headerInfo = header.toHuman()

        let group = "genesis";
        let groupColor = GROUP_TO_COLOR[group]

        // The genesis block (number 0) does not have the normal PoW seal on it.
        // This avoids a crash when you re-start the node. There is likely a more
        // idiomatic way to do this is js.
        if (header.number != 0) {
            const seal_data = header.digest.logs[0].toJSON().seal[1];
            const seal = seal_data.slice(0, 4)

            group = SEAL_TO_GROUP[seal] ?? "genesis"
            groupColor = GROUP_TO_COLOR[group]

            // Detect the PoW algorithm from the first byte of the seal.
            // This corresponds to `pub struct SupportedHashes` in multi-pow/src/lib.rs
            // switch (seal_data.slice(0, 4)) {
            //     case "0x00":
            //         group = "md5";
            //         groupColor = GROUP_TO_COLOR[group]
            //         break;
            //     case "0x01":
            //         group = "sha3";
            //         groupColor = GROUP_TO_COLOR[group]
            //         break;
            //     case "0x02":
            //         group = "keccak";
            //         groupColor = GROUP_TO_COLOR[group]
            // }

            console.log(`group: ${group}`);
        }


        const newNodes = [
            ...data.nodes,
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
                groupColor: groupColor
            },
        ];

        const styledNodes = newNodes.map((node, i) => {
            return {
                ...node,
                style:
                    newNodes.filter((n) => n.number === node.number && n.hash !== node.hash)
                        .length === 0
                        ? ""
                        : "bg-rose-900 text-white",
            };
        });

        setData({ nodes: styledNodes })

        return [ ...oldHeaders, header ]
    }

    async function main() {
        const api = await ApiPromise.create({ provider: wsProvider })


        // We only display a couple, then unsubscribe
        let count = 0
        setRunning(true)
        setData({ nodes: [] })
        setBlocks([])

        // Subscribe to the new headers on-chain. The callback is fired when new headers
        // are found, the call itself returns a promise with a subscription that can be
        // used to unsubscribe from the newHead subscription
        const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
            setHeaders(updateHeaders(header, headers))

            setLatestBlock(header.number.toHuman());

            // setData(({ nodes }) => {

            //     let group = "genesis";
            //     let groupColor = "bg-green-600"

            //     // The genesis block (number 0) does not have the normal PoW seal on it.
            //     // This avoids a crash when you re-start the node. There is likely a more
            //     // idiomatic way to do this is js.
            //     if (header.number != 0) {
            //         const seal_data = header.digest.logs[0].toJSON().seal[1];
            //         // Detect the PoW algorithm from the first byte of the seal.
            //         // This corresponds to `pub struct SupportedHashes` in multi-pow/src/lib.rs
            //         switch (seal_data.slice(0, 4)) {
            //             case "0x00":
            //                 group = "md5";
            //                 groupColor = "bg-red-600"
            //                 break;
            //             case "0x01":
            //                 group = "sha3";
            //                 groupColor = "bg-blue-600"
            //                 break;
            //             case "0x02":
            //                 group = "keccak";
            //                 groupColor = "bg-purple-600"
            //         }

            //         console.log(`group: ${group}`);
            //     }


            //     const newNodes = [
            //         ...nodes,
            //         {
            //             hash: header.hash.toHuman(),
            //             parentHash: header.parentHash.toHuman(),
            //             number: header.number.toHuman(),
            //             stateRoot: header.stateRoot.toHuman(),
            //             extrinsicsRoot: header.extrinsicsRoot.toHuman(),
            //             digestLogs: header.digest.logs.map((d) => {
            //                 return d.toHuman().Seal
            //             }),
            //             group: group,
            //             groupColor: groupColor
            //         },
            //     ];

            //     const styledNodes = newNodes.map((node, i) => {
            //         return {
            //             ...node,
            //             style:
            //                 newNodes.filter((n) => n.number === node.number && n.hash !== node.hash)
            //                     .length === 0
            //                     ? ""
            //                     : "bg-rose-900 text-white",
            //         };
            //     });

            //     return {
            //         nodes: styledNodes,
            //     };
            // });

            if (++count === MAX_CHAIN_COUNT) {
                unsubscribe();
                setRunning(false);
            }
        });
    }

    return (
        <div className="flex flex-col justify-center h-screen overflow-clip">
            <div className="flex items-center justify-center flex-grow gap-4">
                <div className="flex flex-col justify-center">
                    {!running && (
                        <button className="px-4 py-2 text-white bg-blue-600 rounded-full" onClick={() => main()}>start</button>
                    )}
                </div>
                { data.nodes.length > 0 && (
                    <BlockTable nodes={data.nodes} latestBlock={latestBlock} />
                )}
            </div>

            <HorizontalBlockList nodes={data.nodes} />
        </div>
    );
}
