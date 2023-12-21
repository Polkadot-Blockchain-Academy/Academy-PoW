"use client";

export default function BlockTable({ nodes }) {
    return (
        <div className="flex flex-col p-6 m-6 overflow-y-scroll border rounded-lg w-max h-96">
            <table className="table-fixed">
                <thead>
                    <tr>
                        <th>#</th>
                        <th>hash</th>
                    </tr>
                </thead>
                <tbody>
                    { nodes && nodes.map((node, index) => {
                        return (
                            <tr className="gap-4">
                                <td>{ node.number }</td>
                                <td>{ node.hash }</td>
                            </tr>
                        )
                    })}
                </tbody>
            </table>
        </div>
    );
}
