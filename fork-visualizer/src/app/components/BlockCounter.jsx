"use client"

export default function BlockCounter ({ latestBlockNumber }) {
    return (
        <div className="fixed z-50 flex w-auto h-4 text-white top-1 left-1">
            Latest Block Number: { latestBlockNumber }
        </div>
    )
}
