'use client'

import { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { setupWalletSelector, WalletSelector } from '@near-wallet-selector/core'
import { setupModal, WalletSelectorModal } from '@near-wallet-selector/modal-ui'
import { setupMyNearWallet } from '@near-wallet-selector/my-near-wallet'
import '@near-wallet-selector/modal-ui/styles.css'
import { Account, connect, keyStores } from 'near-api-js'
import { NEAR_CONFIG } from '@/utils/near'

interface WalletContextType {
  selector: WalletSelector | null
  modal: WalletSelectorModal | null
  accountId: string | null
  account: Account | null
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

export function WalletProvider({ children }: { children: ReactNode }) {
  const [selector, setSelector] = useState<WalletSelector | null>(null)
  const [modal, setModal] = useState<WalletSelectorModal | null>(null)
  const [accountId, setAccountId] = useState<string | null>(null)
  const [account, setAccount] = useState<Account | null>(null)

  useEffect(() => {
    async function initWallet() {
      const _selector = await setupWalletSelector({
        network: NEAR_CONFIG.networkId as any,
        modules: [setupMyNearWallet()],
      })

      const _modal = setupModal(_selector, {
        contractId: '',
      })

      setSelector(_selector)
      setModal(_modal)

      const state = _selector.store.getState()
      const accounts = state.accounts

      if (accounts.length > 0) {
        const accountId = accounts[0].accountId
        setAccountId(accountId)

        // Create Account object
        const keyStore = new keyStores.BrowserLocalStorageKeyStore()
        const nearConnection = await connect({
          networkId: NEAR_CONFIG.networkId,
          keyStore,
          nodeUrl: NEAR_CONFIG.nodeUrl,
        })
        const account = await nearConnection.account(accountId)
        setAccount(account)
      }
    }

    initWallet()
  }, [])

  useEffect(() => {
    if (!selector) return

    const subscription = selector.store.observable.subscribe(async (state) => {
      const accounts = state.accounts

      if (accounts.length > 0) {
        const accountId = accounts[0].accountId
        setAccountId(accountId)

        const keyStore = new keyStores.BrowserLocalStorageKeyStore()
        const nearConnection = await connect({
          networkId: NEAR_CONFIG.networkId,
          keyStore,
          nodeUrl: NEAR_CONFIG.nodeUrl,
        })
        const account = await nearConnection.account(accountId)
        setAccount(account)
      } else {
        setAccountId(null)
        setAccount(null)
      }
    })

    return () => subscription.unsubscribe()
  }, [selector])

  const signIn = () => {
    modal?.show()
  }

  const signOut = async () => {
    const wallet = await selector?.wallet()
    await wallet?.signOut()
    setAccountId(null)
    setAccount(null)
  }

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

export function useWallet() {
  return useContext(WalletContext)
}
