'use client'

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react'
import { setupWalletSelector, WalletSelector } from '@near-wallet-selector/core'
import { setupModal, WalletSelectorModal } from '@near-wallet-selector/modal-ui'
import { setupMyNearWallet } from '@near-wallet-selector/my-near-wallet'
import * as nearAPI from 'near-api-js'
import { NEAR_CONFIG } from '@/utils/near'

import '@near-wallet-selector/modal-ui/styles.css'

interface WalletContextType {
  selector: WalletSelector | null
  modal: WalletSelectorModal | null
  accountId: string | null
  account: nearAPI.Account | null
  signIn: () => void
  signOut: () => void
  isConnected: boolean
}

const WalletContext = createContext<WalletContextType>({
  selector: null,
  modal: null,
  accountId: null,
  account: null,
  signIn: () => {},
  signOut: () => {},
  isConnected: false,
})

export function useWallet() {
  return useContext(WalletContext)
}

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [selector, setSelector] = useState<WalletSelector | null>(null)
  const [modal, setModal] = useState<WalletSelectorModal | null>(null)
  const [accountId, setAccountId] = useState<string | null>(null)
  const [account, setAccount] = useState<nearAPI.Account | null>(null)

  useEffect(() => {
    const initWallet = async () => {
      const _selector = await setupWalletSelector({
        network: NEAR_CONFIG.networkId as any,
        modules: [setupMyNearWallet()],
      })

      const _modal = setupModal(_selector, {
        contractId: '',
        description: 'Connect your NEAR wallet',
      })

      setSelector(_selector)
      setModal(_modal)

      // Check if already signed in
      if (_selector.isSignedIn()) {
        const state = _selector.store.getState()
        const accountId = state.accounts[0]?.accountId
        if (accountId) {
          setAccountId(accountId)

          // Create account instance
          const { connect, keyStores } = nearAPI
          const keyStore = new keyStores.BrowserLocalStorageKeyStore()
          const near = await connect({ ...NEAR_CONFIG, keyStore })
          const acc = await near.account(accountId)
          setAccount(acc)
        }
      }
    }

    initWallet()
  }, [])

  const signIn = useCallback(() => {
    modal?.show()
  }, [modal])

  const signOut = useCallback(async () => {
    if (selector) {
      const wallet = await selector.wallet()
      await wallet.signOut()
      setAccountId(null)
      setAccount(null)
    }
  }, [selector])

  return (
    <WalletContext.Provider
      value={{
        selector,
        modal,
        accountId,
        account,
        signIn,
        signOut,
        isConnected: !!accountId,
      }}
    >
      {children}
    </WalletContext.Provider>
  )
}
