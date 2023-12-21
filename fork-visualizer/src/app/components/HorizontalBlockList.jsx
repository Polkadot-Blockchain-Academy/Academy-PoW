"use client";

import { useEffect, useRef, useState } from 'react'

const Block = ({ node }) => {
    console.log(node)
    return (
        <div className={`p-4 w-64 h-64 col-span-1 m-4 overflow-scroll border-2 rounded-md ${node.groupColor}`} id={node.hash}>
            <div className='flex flex-col self-center'>
                <span>Block number: { node.number }</span>
                <span>Group: { node.group }</span>
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
    const [ followEnd, setFollowEnd ] = useState(true);

    const blocksEndRef = useRef(null)

    const scrollToBottom = () => {
        if (!followEnd) return;
        blocksEndRef.current?.scrollIntoView({ behavior: "smooth" })
    }

    useEffect(() => {
        scrollToBottom()
    }, [nodes]);

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
                    <button className='px-4 py-2 m-4 text-white bg-blue-600 rounded-full' onClick={() => setFollowEnd(!followEnd)}>{ followEnd ? "Stop Following" : "Follow" }</button>
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

