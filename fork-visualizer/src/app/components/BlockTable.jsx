"use client";

import { useEffect, useRef, useState } from 'react'

export default function BlockTable({ nodes, latestBlock }) {
    const [ autoScroll, setAutoScroll ] = useState(true);

    const [ md5Count, setMd5Count ] = useState(0)
    const [ sha3Count, setSha3Count ] = useState(0)
    const [ keccakCount, setKeccakCount ] = useState(0)

    const blocksEndRef = useRef(null)

    const scrollToBottom = () => {
        blocksEndRef.current?.scrollIntoView({ behavior: "smooth" })
    }

    useEffect(() => {
        let _md5Count = nodes.filter(node => node.group === "md5").length
        let _sha3Count = nodes.filter(node => node.group === "sha3").length
        let _keccakCount = nodes.filter(node => node.group === "keccak").length

        setMd5Count(_md5Count)
        setSha3Count(_sha3Count)
        setKeccakCount(_keccakCount)

        if (!autoScroll) return;
        scrollToBottom()
    }, [nodes]);

    return (
        <div className='flex flex-col m-6'>
            {latestBlock && (
                <h1 className='flex self-center'>latest block: {latestBlock}</h1>
            )}

            <div className='flex items-center justify-center'>
                <button className={`px-3 py-1 m-4 text-white rounded-full ${autoScroll ? "bg-red-600" : "bg-blue-600"}`} onClick={() => setAutoScroll(!autoScroll)}>{ autoScroll ? "Stop Auto Scroll" : "Start Auto Scroll" }</button>
                <span className='p-4 m-2'>auto scrolling: <span className={ autoScroll ? "underline" : "" }>{ autoScroll ? "enabled" : "disabled" }</span></span>

                <div className='flex flex-col'>
                    <span className='self-center'>Counts</span>
                    <div className='flex gap-6'>
                        <span className='text-red-600'>md5: { md5Count }</span>
                        <span className='text-blue-600'>sha3: { sha3Count }</span>
                        <span className='text-purple-600'>keccak: { keccakCount }</span>
                    </div>
                </div>
            </div>
            <div className="flex flex-col p-6 overflow-y-scroll border rounded-lg w-max h-96">
                <table className="table-auto">
                    <thead>
                        <tr>
                            <th>#</th>
                            <th>algo</th>
                            <th>hash</th>
                        </tr>
                    </thead>
                    <tbody>
                        { nodes && nodes.map((node, index) => {
                            return (
                                <tr className="gap-4" ref={blocksEndRef}>
                                    <td className={`px-2 ${ node.style }`}>{ node.number }</td>
                                    <td className={`px-2 ${ node.groupColor }`}>{ node.group }</td>
                                    <td className="px-2">{ node.hash }</td>
                                </tr>
                            )
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
