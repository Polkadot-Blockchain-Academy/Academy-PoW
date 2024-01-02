"use client";

import { useEffect, useRef, useState } from 'react'
import { AUTO_SCROLL_DEFAULT } from "@/constants"

const Block = ({ node }) => {
    return (
        <div className={`p-4 w-32 h-32 col-span-1 m-4 overflow-scroll ${ node.duplicate ? "border-8 border-rose-900" : "" } rounded-lg ${node.groupColor}`} id={node.hash}>
            <div className='flex flex-col'>
                <span className='flex self-center'>{ node.number }</span>
                <span className='flex self-center'>{ node.group }</span>
            </div>
            {/* <div className={`grid grid-cols-3 ${ node.style }`}>
                <span className={`col-span-3 text-xl`}>
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
                        node.digestLogs.map((d, i) => {
                            return (
                                <div key={i}>
                                    <span>seal1: {d[0]}</span>
                                    <span>seal2: {d[1]}</span>
                                </div>
                            );
                        })}
                </div>
            </div> */}
        </div>
    );
}


export default function HorizontalBlockList({ nodes }) {
    const [ followEnd, setFollowEnd ] = useState(AUTO_SCROLL_DEFAULT);

    const blocksEndRef = useRef(null)

    useEffect(() => {
        const scrollToBottom = () => {
            if (!followEnd) return;
            blocksEndRef.current?.scrollIntoView({ behavior: "smooth" })
        }

        scrollToBottom()
    }, [followEnd]);

    return (
        <div className={`flex flex-col ${ nodes.length > 0 ? "flex-grow" : "" }`}>
            { nodes.length > 0 && (
                <>
                    <div className='flex gap-4 m-4'>
                        <div className='flex flex-row items-center gap-2'>
                            <span className='w-16 h-16 p-10 bg-green-600'></span>
                            <span>genesis</span>
                        </div>

                        <div className='flex flex-row items-center gap-2'>
                            <span className='w-16 h-16 p-10 bg-red-600'></span>
                            <span>md5</span>
                        </div>

                        <div className='flex flex-row items-center gap-2'>
                            <span className='w-16 h-16 p-10 bg-blue-600'></span>
                            <span>sha3</span>
                        </div>

                        <div className='flex flex-row items-center gap-2'>
                            <span className='w-16 h-16 p-10 bg-purple-600'></span>
                            <span>keccak</span>
                        </div>
                    </div>
                    <button className={`px-4 py-2 m-4 text-white rounded-full ${followEnd ? "bg-red-600" : "bg-blue-600"}`} onClick={() => setFollowEnd(!followEnd)}>{ followEnd ? "Stop Auto Scroll" : "Start Auto Scroll" }</button>
                </>
            )}
            <div className="flex flex-row justify-start flex-grow overflow-scroll">
                { nodes && nodes.map((node, index) => {
                    return (
                        <div key={index}>
                            <Block node={node} />
                        </div>
                    )
                })}
                <div ref={blocksEndRef} />
            </div>
        </div>
    );
}

