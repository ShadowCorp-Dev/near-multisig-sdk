import type { Metadata } from 'next'
import './globals.css'
import { WalletProvider } from '@/components/providers/WalletProvider'

export const metadata: Metadata = {
  title: 'Weighted Multisig',
  description: 'NEAR weighted multisig wallet interface',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>
        <WalletProvider>
          {children}
        </WalletProvider>
      </body>
    </html>
  )
}
