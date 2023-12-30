"use client"

import LayoutFlow from '@/app/components/LayoutFlow';


export default function Home () {
    return (
        <div className="flex flex-col justify-center overflow-y-scroll">
            <div className="flex w-auto h-screen">
                <LayoutFlow />
            </div>
        </div>
    );
};
