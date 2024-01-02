"use client"

import { useState } from "react"

export default function NodeWsInput ({ addNode , children}) {

    const [ nodeAddr, setNodeAddr ] = useState("")

    const onAddNode = () => {
        addNode(nodeAddr)
        setNodeAddr("")
    }

    const onKeyDownHandler = e => {
        if (e.keyCode === 13) {
            onAddNode()
            setNodeAddr("")
        }
    }

    return (
        <div className="fixed z-50 flex flex-col top-1 left-1">
            <div className="flex flex-row gap-2">
                <input
                    className="px-2 py-1 my-1 rounded-md"
                    type="text"
                    name="nodeAddr"
                    value={nodeAddr}
                    placeholder="Enter Node WS Address"
                    onChange={(e) => {
                        setNodeAddr(e.target.value)
                    }}
                    onKeyDown={onKeyDownHandler}
                />

                <button className="text-white btn" onClick={onAddNode}>Add</button>
            </div>

            { children }
        </div>
    )
}
