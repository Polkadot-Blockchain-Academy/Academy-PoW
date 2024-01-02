"use client"

import { ApiPromise, WsProvider } from "@polkadot/api";
import { useState } from "react"

const defaultFunc = () => console.log("oops this is the default func")
export default function NodeWsController ({ wsAddress, updateStuff, removeNode }) {

    const [ isSubscribed, setIsSubscribed ] = useState(false)
    const [ loading, setLoading ] = useState(false)
    const [ unsubscribeFromNode, setUnsubscribeFromNode ] = useState(() => defaultFunc)
    const [ latestHash, setLatestHash ] = useState("")

    const subscribe = async () => {
        if (isSubscribed) return

        setLoading(true)

        console.log("subscribing...", wsAddress)

        try {
            const wsProvider = new WsProvider(wsAddress)
            const api = await ApiPromise.create({ provider: wsProvider })
            const unsubscribe = await api.rpc.chain.subscribeNewHeads(async (header) => {
                setLatestHash(header.hash.toHuman())
                await updateStuff(header, api, wsAddress)
            });

            setIsSubscribed(true)

            const newUnsubscribe = () => {
                console.log("unsubscribing from ", wsAddress)
                setLoading(true)
                unsubscribe();
                setIsSubscribed(false)
                setLoading(false)
            }

            // return a function that we can call later
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
        if (loading) setLoading(false)

        setLatestHash("")
        removeNode(wsAddress)
    }

    return (
        <tr>
            <th className="px-4"><button className="text-red-600 btn" onClick={handleRemoveNode}>remove</button></th>
            <th className="px-4">{ wsAddress }</th>
            <th className="px-4">
                <div className="flex content-center text-center justify-items-center">
                    { loading && (
                        <svg className="flex-grow w-5 h-5 mr-3 -ml-1 text-white animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                    )}

                    { !loading && !isSubscribed && <button className="flex-grow text-green-600 underline btn" onClick={subscribe}>subscribe</button> }
                    { !loading && isSubscribed && <button className="flex-grow text-red-600 underline btn animate-pulse" onClick={unsubscribeFromNode}>unsubscribe</button> }
                </div>
            </th>
            <th className="px-4">{ latestHash && <span>{ latestHash }</span> }</th>
        </tr>
    )
}
