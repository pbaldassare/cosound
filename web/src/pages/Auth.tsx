import { useEffect, useState, type FormEvent, type ReactNode } from 'react'
import { Link, Navigate, useLocation, useNavigate } from 'react-router-dom'
import { supabase } from '../lib/supabase'
import { useAuth } from '../lib/auth'
import { btnPrimary } from '../components/Layout'

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
      setError(err instanceof Error ? err.message : 'Something went wrong. Please try again.')
    } finally {
      setBusy(false)
    }
  }
  return { busy, error, done, submit }
}

const str = (d: FormData, k: string) => String(d.get(k) ?? '').trim()

export function Signup() {
  const { session } = useAuth()
  const { busy, error, done, submit } = useForm()
  if (session) return <Navigate to="/account" replace />

  const onSubmit = submit(async (d) => {
    const password = String(d.get('password'))
    if (password.length < MIN_PASSWORD) throw new Error(`Password must be at least ${MIN_PASSWORD} characters.`)
    if (password !== String(d.get('confirm'))) throw new Error('Passwords do not match.')
    const { data, error } = await supabase.auth.signUp({
      email: str(d, 'email'),
      password,
      options: {
        data: { first_name: str(d, 'first_name'), last_name: str(d, 'last_name') },
        emailRedirectTo: `${window.location.origin}/account`,
      },
    })
    if (error) throw error
    if (!data.session) return 'Check your inbox: we sent you a link to verify your email address.'
  })

  return (
    <AuthCard title="Create your account" subtitle={<>Already have one? <Link to="/login" className="text-sky hover:underline">Sign in</Link></>}>
      {done ? <Message kind="ok">{done}</Message> : (
        <form onSubmit={onSubmit} className="space-y-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="First name" name="first_name" autoComplete="given-name" maxLength={80} />
            <Field label="Last name" name="last_name" autoComplete="family-name" maxLength={80} />
          </div>
          <Field label="Email" name="email" type="email" autoComplete="email" />
          <Field label="Password" name="password" type="password" autoComplete="new-password" minLength={MIN_PASSWORD} />
          <Field label="Confirm password" name="confirm" type="password" autoComplete="new-password" minLength={MIN_PASSWORD} />
          {error && <Message kind="error">{error}</Message>}
          <button className={`${btnPrimary} w-full`} disabled={busy}>{busy ? 'Creating account…' : 'Create account'}</button>
        </form>
      )}
    </AuthCard>
  )
}

export function Login() {
  const { session } = useAuth()
  const navigate = useNavigate()
  const from = (useLocation().state as { from?: string } | null)?.from ?? '/account'
  const { busy, error, submit } = useForm()
  if (session) return <Navigate to={from} replace />

  const onSubmit = submit(async (d) => {
    const { error } = await supabase.auth.signInWithPassword({ email: str(d, 'email'), password: String(d.get('password')) })
    if (error) throw error
    navigate(from, { replace: true })
  })

  return (
    <AuthCard title="Sign in" subtitle={<>New to COSOUND? <Link to="/signup" className="text-sky hover:underline">Create an account</Link></>}>
      <form onSubmit={onSubmit} className="space-y-4">
        <Field label="Email" name="email" type="email" autoComplete="email" />
        <Field label="Password" name="password" type="password" autoComplete="current-password" />
        <div className="text-right text-sm"><Link to="/forgot-password" className="text-muted hover:text-white">Forgot password?</Link></div>
        {error && <Message kind="error">{error}</Message>}
        <button className={`${btnPrimary} w-full`} disabled={busy}>{busy ? 'Signing in…' : 'Sign in'}</button>
      </form>
    </AuthCard>
  )
}

export function ForgotPassword() {
  const { busy, error, done, submit } = useForm()
  const onSubmit = submit(async (d) => {
    const { error } = await supabase.auth.resetPasswordForEmail(str(d, 'email'), { redirectTo: `${window.location.origin}/reset-password` })
    if (error) throw error
    // Same message whether or not the address exists, to avoid account enumeration.
    return 'If an account exists for this email, you will receive a reset link shortly.'
  })
  return (
    <AuthCard title="Reset your password" subtitle="We'll email you a secure link to choose a new password.">
      {done ? <Message kind="ok">{done}</Message> : (
        <form onSubmit={onSubmit} className="space-y-4">
          <Field label="Email" name="email" type="email" autoComplete="email" />
          {error && <Message kind="error">{error}</Message>}
          <button className={`${btnPrimary} w-full`} disabled={busy}>{busy ? 'Sending…' : 'Send reset link'}</button>
        </form>
      )}
    </AuthCard>
  )
}

export function ResetPassword() {
  const { session, loading } = useAuth()
  const navigate = useNavigate()
  const { busy, error, submit } = useForm()
  const [linkError, setLinkError] = useState<string | null>(null)
  useEffect(() => {
    const p = new URLSearchParams(window.location.hash.slice(1))
    if (p.get('error_description')) setLinkError(p.get('error_description'))
  }, [])

  const onSubmit = submit(async (d) => {
    const password = String(d.get('password'))
    if (password.length < MIN_PASSWORD) throw new Error(`Password must be at least ${MIN_PASSWORD} characters.`)
    if (password !== String(d.get('confirm'))) throw new Error('Passwords do not match.')
    const { error } = await supabase.auth.updateUser({ password })
    if (error) throw error
    navigate('/account', { replace: true })
  })

  if (loading) return <AuthCard title="Choose a new password"><p className="text-muted">Loading…</p></AuthCard>
  if (!session) return (
    <AuthCard title="Link not valid">
      <Message kind="error">{linkError ?? 'This reset link is invalid or has expired.'}</Message>
      <Link to="/forgot-password" className={`${btnPrimary} mt-6 w-full`}>Request a new link</Link>
    </AuthCard>
  )
  return (
    <AuthCard title="Choose a new password">
      <form onSubmit={onSubmit} className="space-y-4">
        <Field label="New password" name="password" type="password" autoComplete="new-password" minLength={MIN_PASSWORD} />
        <Field label="Confirm new password" name="confirm" type="password" autoComplete="new-password" minLength={MIN_PASSWORD} />
        {error && <Message kind="error">{error}</Message>}
        <button className={`${btnPrimary} w-full`} disabled={busy}>{busy ? 'Saving…' : 'Save new password'}</button>
      </form>
    </AuthCard>
  )
}
