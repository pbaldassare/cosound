import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { StoreProvider, useStore } from './lib/store'
import { Shell } from './components/Shell'
import Onboarding from './pages/Onboarding'
import Chat from './pages/Chat'
import Production from './pages/Production'
import Library from './pages/Library'
import Imports from './pages/Imports'
import Plugins from './pages/Plugins'
import Daws from './pages/Daws'
import Agents from './pages/Agents'
import SettingsPage from './pages/Settings'
import Account from './pages/Account'

const pages = { chat: Chat, production: Production, library: Library, imports: Imports, plugins: Plugins, daws: Daws, agents: Agents, settings: SettingsPage, account: Account }

function App() {
  const { settings, page } = useStore()
  if (settings.onboarding_done !== '1') return <Onboarding />
  const Page = pages[page]
  return <Shell><Page /></Shell>
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <StoreProvider><App /></StoreProvider>
  </StrictMode>,
)
