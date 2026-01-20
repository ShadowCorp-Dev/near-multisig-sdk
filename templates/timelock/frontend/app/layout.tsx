import type { Metadata } from 'next'
import './globals.css'
import { WalletProvider } from '@/components/providers/WalletProvider'

export const metadata: Metadata = {
  title: 'Timelock Multisig',
  description: 'NEAR timelock multisig wallet interface',
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
