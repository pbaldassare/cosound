import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import type { Session } from '@supabase/supabase-js'
import { supabase } from './supabase'
import { useT } from '../i18n'

type AuthState = { session: Session | null; isAdmin: boolean; loading: boolean }

const AuthContext = createContext<AuthState>({ session: null, isAdmin: false, loading: true })

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>({ session: null, isAdmin: false, loading: true })

  useEffect(() => {
    const load = async (session: Session | null) => {
      // UI hint only: every admin read/write is enforced server-side by RLS.
      const isAdmin = session ? Boolean((await supabase.rpc('is_admin')).data) : false
      setState({ session, isAdmin, loading: false })
    }
    supabase.auth.getSession().then(({ data }) => load(data.session))
    const { data } = supabase.auth.onAuthStateChange((_event, session) => {
      // Defer: calling supabase inside this callback can deadlock the auth lock.
      setTimeout(() => load(session), 0)
    })
    return () => data.subscription.unsubscribe()
  }, [])

  return <AuthContext.Provider value={state}>{children}</AuthContext.Provider>
}

export const useAuth = () => useContext(AuthContext)

export function RequireAuth({ children, admin = false }: { children: ReactNode; admin?: boolean }) {
  const { session, isAdmin, loading } = useAuth()
  const location = useLocation()
  const { t } = useT()
  if (loading) return <div className="min-h-[60vh] grid place-items-center text-muted">{t.common.loading}</div>
  if (!session) return <Navigate to="/login" replace state={{ from: location.pathname }} />
  if (admin && !isAdmin) return <Navigate to="/account" replace />
  return children
}
