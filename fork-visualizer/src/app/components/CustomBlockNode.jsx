"use client"

import { memo as Memo } from 'react';
import { Handle, Position } from 'reactflow';

const CustomBlockNode = Memo(({ data, isConnectable }) => {

    const classNameMap = {
        "genesis": "bg-green-600 flex flex-col rounded-lg",
        "md5": "bg-red-600 flex flex-col rounded-lg",
        "sha3": "bg-blue-600 flex flex-col rounded-lg",
        "keccak": "bg-purple-600 flex flex-col rounded-lg",
    }

    return (
        <div className={`${ classNameMap[data.group] }`}>
            <Handle
                type="target"
                position={Position.Left}
                onConnect={(params) => console.log('handle onConnect', params)}
                isConnectable={isConnectable}
            />
            <div className='flex flex-col px-4 py-4 mx-4 my-4 text-white'>
                <span className='text-3xl'>Block number: <span className='underline'>{ data.number }</span></span>
                <span className='text-2xl'>hash: { data.label }</span>
                <span className='text-2xl'>algo: <span className='underline'>{ data.group }</span></span>
                <span className='text-xl'>parent hash: { data.parentHash }</span>

                <span className='text-xl'>author account: { data.authorAccount }</span>
                <span className='text-xl'>reporting node: { data.reportingNode }</span>
            </div>
            <Handle
                type="source"
                position={Position.Right}
                id="a"
                isConnectable={isConnectable}
            />
        </div>
    );
});

export default CustomBlockNode
