"use client"

import { ApiPromise, WsProvider } from '@polkadot/api';
import { useState } from 'react';
import { OrgChartTree } from "@/app/tree/tree"

const MAX_CHAIN_COUNT = 256

// // Construct
const wsProvider = new WsProvider('wss://rpc.polkadot.io');



export default function Home() {

  const [ latestBlock, setLatestBlock ] = useState()
  const [ treeData, setTreeData ] = useState({ name: "root", id: 0, children: [] })
  const [ running, setRunning ] = useState(false)

  async function main () {
    const api = await ApiPromise.create({ provider: wsProvider });

    // We only display a couple, then unsubscribe
    let count = 0;
    setRunning(true)

    // Subscribe to the new headers on-chain. The callback is fired when new headers
    // are found, the call itself returns a promise with a subscription that can be
    // used to unsubscribe from the newHead subscription
    const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
      setLatestBlock(header.number.toHuman())

      setTreeData(( td ) => {
        const newRoot = {...td}

        // find parent and add child
        const hash = header.parentHash.toHuman()

        function appendChild(root, hash) {

          if (root.children.length === 0) {
            if (root.id === 0) {
              root.children = [...root.children, {
                name: `${header.number.toHuman()}`,
                id: header.hash.toHuman(),
                attributes: {
                  hash: header.hash.toHuman(),
                  parentHash: header.parentHash.toHuman(),
                  number: header.number.toHuman(),
                  stateRoot: header.stateRoot.toHuman(),
                  extrinsicsRoot: header.extrinsicsRoot.toHuman()
                },
                children: []
              }]
              return true
            }

            return false
          }

          const parent = root.children.find(c => c.id === hash)
          if (typeof parent !== 'undefined') {
            console.log("parent found", parent, typeof parent)
            parent.children = [...parent.children, {
              name: `${header.number.toHuman()}`,
              id: header.hash.toHuman(),
              attributes: {
                hash: header.hash.toHuman(),
                parentHash: header.parentHash.toHuman(),
                number: header.number.toHuman(),
                stateRoot: header.stateRoot.toHuman(),
                extrinsicsRoot: header.extrinsicsRoot.toHuman()
              },
              children: []
            }]

            return true
          }

          // for (let i = 0; i < root.children.length; i++) {
          //   if (appendChild(root.children[i], hash)) break
          // }
        }

        appendChild(newRoot, hash)

        return {...newRoot}
      })

      if (++count === MAX_CHAIN_COUNT) {
        unsubscribe();
        setRunning(false);
      }
    });
  }


  return (
    <>
      { latestBlock && <h1>latest block: { latestBlock } </h1> }
      { !running && (
        <div>
          <button onClick={() => main()}>Watch Chain</button>
        </div>
      )}

      <div className="flex items-stretch min-h-screen mx-6">
        <OrgChartTree treeData={treeData} />
      </div>

    </>
  )
}
