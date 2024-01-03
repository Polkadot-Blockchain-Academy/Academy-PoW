import '@/app/globals.css'

export const metadata = {
    title: 'Fork and MultiPoW Visualizer',
    description: 'Display the complete blockchain including forks and PoW seals as reported by multiple nodes.',
}

export default function RootLayout({ children }) {
    return (
        <html lang="en">
        <body>{children}</body>
        </html>
    )
}
