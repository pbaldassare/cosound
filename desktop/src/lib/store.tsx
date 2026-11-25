import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from 'react'
import type { Session } from '@supabase/supabase-js'
import { api, type AppInfo, type PortHealth, type Project, type Settings } from './api'
import { supabase } from './supabase'

export type Page = 'chat' | 'production' | 'sketch' | 'library' | 'imports' | 'plugins' | 'daws' | 'agents' | 'settings' | 'account'

type Store = {
  info: AppInfo | null
  settings: Settings
  setSetting: (k: string, v: string) => Promise<void>
  page: Page
  go: (p: Page) => void
  project: Project | null
  openProject: (id: string | null) => Promise<void>
  agent: PortHealth | null
  session: Session | null
  assistantName: string
}

const Ctx = createContext<Store>(null as unknown as Store)
export const useStore = () => useContext(Ctx)

export function StoreProvider({ children }: { children: ReactNode }) {
  const [info, setInfo] = useState<AppInfo | null>(null)
  const [settings, setSettings] = useState<Settings | null>(null)
  const [page, go] = useState<Page>('chat')
  const [project, setProject] = useState<Project | null>(null)
  const [agent, setAgent] = useState<PortHealth | null>(null)
  const [session, setSession] = useState<Session | null>(null)

  useEffect(() => {
    api.appInfo().then(setInfo)
    api.ailStatus().then(setAgent)
    api.settings().then(async (s) => {
      setSettings(s)
      if (s.current_project) {
        const all = await api.projects(true)
        setProject(all.find((p) => p.id === s.current_project) ?? null)
      }
    })
    if (!supabase) return
    supabase.auth.getSession().then(({ data }) => setSession(data.session))
    const { data } = supabase.auth.onAuthStateChange((_e, s) => setSession(s))
    return () => data.subscription.unsubscribe()
  }, [])

  const setSetting = useCallback(async (k: string, v: string) => {
    await api.setSetting(k, v)
    setSettings((s) => ({ ...s, [k]: v }))
  }, [])

  const openProject = useCallback(async (id: string | null) => {
    if (!id) {
      await setSetting('current_project', '')
      setProject(null)
      return
    }
    setProject(await api.openProject(id))
    setSettings((s) => ({ ...s, current_project: id }))
  }, [setSetting])

  if (!settings) return null
  return (
    <Ctx.Provider value={{ info, settings, setSetting, page, go, project, openProject, agent, session, assistantName: settings.assistant_name || 'COSOUND' }}>
      {children}
    </Ctx.Provider>
  )
}
