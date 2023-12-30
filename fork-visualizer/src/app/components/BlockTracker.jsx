"use client"

export default function BlockTracker ({ blocks }) {

    return (
        <div className="fixed top-0 right-0 z-50 w-2/5 gap-2 px-2 py-4 text-white overflow-y-clip h-60">
            <span className="pb-2 mb-4 underline">Latest Block Hashes:</span>
            <div className="flex flex-col h-full overflow-x-auto overflow-y-auto">
                { blocks && blocks.sort((a,b) => a.data.number < b.data.number).map((block, i) => {
                    return(
                        <div key={i} className="flex flex-row w-full">
                            <span>{block.data.number }-{ block.data.label }</span>
                        </div>
                    )
                })}
            </div>
        </div>
    )
}
