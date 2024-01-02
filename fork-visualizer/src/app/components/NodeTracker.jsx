"use client"

import { ApiPromise, WsProvider } from "@polkadot/api";
import { useState } from "react"

import { MAX_CHAIN_COUNT } from '@/constants'


const defaultFunc = () => console.log("oops this is the default func")
export function NodeState ({ wsAddress, updateStuff, removeNode }) {

    const [ isSubscribed, setIsSubscribed ] = useState(false)
    const [ loading, setLoading ] = useState(false)
    const [ unsubscribeFromNode, setUnsubscribeFromNode ] = useState(() => defaultFunc)

    const subscribe = async () => {
        if (isSubscribed) return

        setLoading(true)

        let count = 0
        console.log("subscribing...", wsAddress)

        try {
            const wsProvider = new WsProvider(wsAddress)
            const api = await ApiPromise.create({ provider: wsProvider })
            const unsubscribe = await api.rpc.chain.subscribeNewHeads(async (header) => {
                await updateStuff(header, api, wsAddress)

                if (++count === MAX_CHAIN_COUNT) {
                    // would be nice for this to be defined
                    // somewhere for here and below
                    console.log("unsubscribing from ", wsAddress)
                    setLoading(true)
                    unsubscribe();
                    setIsSubscribed(false)
                    setLoading(false)
                }
            });

            setIsSubscribed(true)

            const newUnsubscribe = () => {
                // would be nice for this to be defined
                // somewhere for here and above
                console.log("unsubscribing from ", wsAddress)
                setLoading(true)
                unsubscribe();
                setIsSubscribed(false)
                setLoading(false)
            }

            setUnsubscribeFromNode(() => newUnsubscribe)
        } catch (error) {
            console.error("failed to subscribe: ", error)
        }

        // update is state
        setLoading(false)
    }

    const handleRemoveNode = () => {
        // if subscribed, unsubscribe
        if (isSubscribed) unsubscribeFromNode()

        removeNode(wsAddress)
    }

    return (
        <div className="flex flex-row gap-3 px-4 text-white">

            <button className="text-red-600 btn" onClick={handleRemoveNode}>remove</button>

            <span>{ wsAddress }</span>

            { loading && (
                <svg className="w-5 h-5 mr-3 -ml-1 text-white animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
            )}

            { !loading && !isSubscribed && <button className="text-green-600 underline btn" onClick={subscribe}>subscribe</button> }
            { !loading && isSubscribed && <button className="text-red-600 underline btn animate-pulse" onClick={unsubscribeFromNode}>unsubscribe</button> }

        </div>
    )
}


export default function NodeTracker ({ addNode , children}) {

    const [ nodeAddr, setNodeAddr ] = useState("")

    const onAddNode = () => {
        addNode(nodeAddr)
        setNodeAddr("")
    }

    return (
        <div className="fixed z-50 flex flex-col top-1 left-1">
            <div className="flex flex-row gap-2">
                <input
                    className="px-2 py-1 my-1 rounded-md"
                    type="text"
                    name="nodeAddr"
                    value={nodeAddr}
                    placeholder="WS address to subscribe to"
                    onChange={(e) => {
                        setNodeAddr(e.target.value)
                    }}
                />

                <button className="text-white btn" onClick={onAddNode}>Add</button>
            </div>

            { children }
        </div>
    )
}
