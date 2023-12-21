"use client";

import { useEffect, useRef, useState } from 'react'

export default function BlockTable({ nodes }) {
    const [ autoScroll, setAutoScroll ] = useState(true);

    const blocksEndRef = useRef(null)

    const scrollToBottom = () => {
        blocksEndRef.current?.scrollIntoView({ behavior: "smooth" })
    }

    useEffect(() => {
        if (!autoScroll) return;
        scrollToBottom()
    }, [nodes]);

    return (
        <div className='flex flex-col'>
            <div>
                <span>auto scrolling: { autoScroll ? "enabled" : "disabled" }</span>
                <button className={`px-4 py-2 m-4 text-white rounded-full ${autoScroll ? "bg-red-600" : "bg-blue-600"}`} onClick={() => setAutoScroll(!autoScroll)}>{ autoScroll ? "Stop Auto Scroll" : "Start Auto Scroll" }</button>
            </div>
            <div className="flex flex-col p-6 m-6 overflow-y-scroll border rounded-lg w-max h-96">
                <table className="table-fixed">
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
