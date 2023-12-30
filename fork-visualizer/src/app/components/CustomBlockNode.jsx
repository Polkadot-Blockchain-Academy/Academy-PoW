import { memo as Memo } from 'react';
import { Handle, Position } from 'reactflow';

const CustomBlockNode = Memo(({ data, isConnectable }) => {
    return (
        <div className={`flex flex-col ${ data.groupColor } rounded-sm`}>
            <Handle
                type="target"
                position={Position.Left}
                onConnect={(params) => console.log('handle onConnect', params)}
                isConnectable={isConnectable}
            />
            <div className='flex flex-col px-4 py-2 mx-4 my-2'>
                <span className='text-2xl'>Block number: { data.number }</span>
                <span className='text-xl'>hash: { data.label }</span>
                <span className='text-xl'>algo: { data.group }</span>
                <span className='text-lg'>parent hash: { data.parentHash }</span>

                <span className='text-sm'>author account: { data.authorAccount }</span>
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
