import { useEffect, useState, type FormEvent, type ReactNode } from 'react'
import { Link, Navigate, useLocation, useNavigate } from 'react-router-dom'
import { supabase } from '../lib/supabase'
import { useAuth } from '../lib/auth'
import { btnPrimary } from '../components/Layout'
import { useT } from '../i18n'
import type { Dict } from '../i18n/en'

const MIN_PASSWORD = 8

function AuthCard({ title, subtitle, children }: { title: string; subtitle?: ReactNode; children: ReactNode }) {
  return (
    <section className="relative grid min-h-[80vh] place-items-center overflow-hidden px-4 py-16">
      <div className="absolute top-10 left-1/2 -z-10 h-80 w-[600px] -translate-x-1/2 rounded-full bg-violet/15 blur-[100px]" />
      <div className="card w-full max-w-md p-8">
        <h1 className="font-display text-2xl font-bold">{title}</h1>
        {subtitle && <p className="mt-2 text-sm text-muted">{subtitle}</p>}
        <div className="mt-7">{children}</div>
      </div>
    </section>
  )
}

function Field({ label, ...props }: { label: string } & React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <label className="block text-sm">
      <span className="mb-1.5 block text-muted">{label}</span>
      <input className="field" required {...props} />
    </label>
  )
}

function Message({ kind, children }: { kind: 'error' | 'ok'; children: ReactNode }) {
  return (
    <p role={kind === 'error' ? 'alert' : 'status'}
      className={`rounded-lg px-3 py-2 text-sm ${kind === 'error' ? 'bg-red-500/10 text-red-300' : 'bg-sky/10 text-sky'}`}>
      {children}
    </p>
  )
}

function useForm() {
  const { t } = useT()
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [done, setDone] = useState<string | null>(null)
  const submit = (fn: (data: FormData) => Promise<string | void>) => async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    setBusy(true); setError(null); setDone(null)
    try {
      const msg = await fn(new FormData(e.currentTarget))
      if (msg) setDone(msg)
    } catch (err) {
      setError(err instanceof Error ? t.auth.errors[err.message] ?? err.message : t.common.error)
    } finally {
      setBusy(false)
    }
  }
  return { busy, error, done, submit }
}

const str = (d: FormData, k: string) => String(d.get(k) ?? '').trim()

function checkPassword(d: FormData, t: Dict) {
  const password = String(d.get('password'))
  if (password.length < MIN_PASSWORD) throw new Error(t.auth.pwTooShort(MIN_PASSWORD))
  if (password !== String(d.get('confirm'))) throw new Error(t.auth.pwMismatch)
  return password
}

export function Signup() {
  const { session } = useAuth()
  const { t } = useT()
  const { busy, error, done, submit } = useForm()
  if (session) return <Navigate to="/account" replace />

  const onSubmit = submit(async (d) => {
    const password = checkPassword(d, t)
    const { data, error } = await supabase.auth.signUp({
      email: str(d, 'email'),
      password,
      options: {
        data: { first_name: str(d, 'first_name'), last_name: str(d, 'last_name') },
        emailRedirectTo: `${window.location.origin}/account`,
      },
    })
    if (error) throw error
    if (!data.session) return t.auth.checkInbox
  })

  return (
    <AuthCard title={t.auth.signupTitle} subtitle={<>{t.auth.haveAccount} <Link to="/login" className="text-sky hover:underline">{t.auth.signIn}</Link></>}>
      {done ? <Message kind="ok">{done}</Message> : (
        <form onSubmit={onSubmit} className="space-y-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label={t.auth.firstName} name="first_name" autoComplete="given-name" maxLength={80} />
            <Field label={t.auth.lastName} name="last_name" autoComplete="family-name" maxLength={80} />
          </div>
          <Field label={t.auth.email} name="email" type="email" autoComplete="email" />
          <Field label={t.auth.password} name="password" type="password" autoComplete="new-password" minLength={MIN_PASSWORD} />
          <Field label={t.auth.confirm} name="confirm" type="password" autoComplete="new-password" minLength={MIN_PASSWORD} />
          {error && <Message kind="error">{error}</Message>}
          <button className={`${btnPrimary} w-full`} disabled={busy}>{busy ? t.auth.creating : t.auth.create}</button>
        </form>
      )}
    </AuthCard>
  )
}

export function Login() {
  const { session } = useAuth()
  const navigate = useNavigate()
  const { t } = useT()
  const from = (useLocation().state as { from?: string } | null)?.from ?? '/account'
  const { busy, error, submit } = useForm()
  if (session) return <Navigate to={from} replace />

  const onSubmit = submit(async (d) => {
    const { error } = await supabase.auth.signInWithPassword({ email: str(d, 'email'), password: String(d.get('password')) })
    if (error) throw error
    navigate(from, { replace: true })
  })

  return (
    <AuthCard title={t.auth.loginTitle} subtitle={<>{t.auth.newHere} <Link to="/signup" className="text-sky hover:underline">{t.auth.createLink}</Link></>}>
      <form onSubmit={onSubmit} className="space-y-4">
        <Field label={t.auth.email} name="email" type="email" autoComplete="email" />
        <Field label={t.auth.password} name="password" type="password" autoComplete="current-password" />
        <div className="text-right text-sm"><Link to="/forgot-password" className="text-muted hover:text-white">{t.auth.forgot}</Link></div>
        {error && <Message kind="error">{error}</Message>}
        <button className={`${btnPrimary} w-full`} disabled={busy}>{busy ? t.auth.signingIn : t.auth.signIn}</button>
      </form>
    </AuthCard>
  )
}

export function ForgotPassword() {
  const { t } = useT()
  const { busy, error, done, submit } = useForm()
  const onSubmit = submit(async (d) => {
    const { error } = await supabase.auth.resetPasswordForEmail(str(d, 'email'), { redirectTo: `${window.location.origin}/reset-password` })
    if (error) throw error
    // Same message whether or not the address exists, to avoid account enumeration.
    return t.auth.resetSent
  })
  return (
    <AuthCard title={t.auth.forgotTitle} subtitle={t.auth.forgotText}>
      {done ? <Message kind="ok">{done}</Message> : (
        <form onSubmit={onSubmit} className="space-y-4">
          <Field label={t.auth.email} name="email" type="email" autoComplete="email" />
          {error && <Message kind="error">{error}</Message>}
          <button className={`${btnPrimary} w-full`} disabled={busy}>{busy ? t.auth.sending : t.auth.sendLink}</button>
        </form>
      )}
    </AuthCard>
  )
}

export function ResetPassword() {
  const { session, loading } = useAuth()
  const { t } = useT()
  const navigate = useNavigate()
  const { busy, error, submit } = useForm()
  const [linkError, setLinkError] = useState<string | null>(null)
  useEffect(() => {
    const p = new URLSearchParams(window.location.hash.slice(1))
    if (p.get('error_description')) setLinkError(p.get('error_description'))
  }, [])

  const onSubmit = submit(async (d) => {
    const password = checkPassword(d, t)
    const { error } = await supabase.auth.updateUser({ password })
    if (error) throw error
    navigate('/account', { replace: true })
  })

  if (loading) return <AuthCard title={t.auth.newPwTitle}><p className="text-muted">{t.common.loading}</p></AuthCard>
  if (!session) return (
    <AuthCard title={t.auth.invalidTitle}>
      <Message kind="error">{linkError ?? t.auth.invalidText}</Message>
      <Link to="/forgot-password" className={`${btnPrimary} mt-6 w-full`}>{t.auth.requestNew}</Link>
    </AuthCard>
  )
  return (
    <AuthCard title={t.auth.newPwTitle}>
      <form onSubmit={onSubmit} className="space-y-4">
        <Field label={t.auth.newPw} name="password" type="password" autoComplete="new-password" minLength={MIN_PASSWORD} />
        <Field label={t.auth.confirmNewPw} name="confirm" type="password" autoComplete="new-password" minLength={MIN_PASSWORD} />
        {error && <Message kind="error">{error}</Message>}
        <button className={`${btnPrimary} w-full`} disabled={busy}>{busy ? t.common.saving : t.auth.saveNewPw}</button>
      </form>
    </AuthCard>
  )
}
