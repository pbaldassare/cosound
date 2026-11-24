import { useEffect, useState, type FormEvent } from 'react'
import { ExternalLink, LogOut } from 'lucide-react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { errText } from '../lib/api'
import { useStore } from '../lib/store'
import { siteUrl, supabase, type Plan, type Profile, type Subscription } from '../lib/supabase'
import { fill, useT } from '../i18n'
import { Badge, Card, ErrorLine, Note, PageHeader, btnGhost, btnPrimary } from '../components/ui'

/** Sign in / sign up form. Shared with onboarding. */
export function AuthForm({ onDone }: { onDone?: () => void }) {
  const { t } = useT()
  const [mode, setMode] = useState<'in' | 'up'>('in')
  const [f, setF] = useState({ email: '', password: '', first: '', last: '' })
  const [error, setError] = useState<string | null>(null)
  const [msg, setMsg] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  if (!supabase) return <Note>{t.account.notConfigured}</Note>

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    setBusy(true); setError(null); setMsg(null)
    try {
      if (mode === 'in') {
        const { error } = await supabase!.auth.signInWithPassword({ email: f.email, password: f.password })
        if (error) throw error
        onDone?.()
      } else {
        const { data, error } = await supabase!.auth.signUp({ email: f.email, password: f.password, options: { data: { first_name: f.first, last_name: f.last } } })
        if (error) throw error
        if (data.session) onDone?.()
        else { setMsg(t.account.checkEmail); setMode('in') }
      }
    } catch (err) {
      const m = errText(err)
      setError(m === 'Invalid login credentials' ? t.account.invalidCredentials : m)
    } finally { setBusy(false) }
  }

  return (
    <form onSubmit={submit} className="space-y-2.5">
      {mode === 'up' && (
        <div className="grid grid-cols-2 gap-2">
          <input className="field" placeholder={t.account.firstName} aria-label={t.account.firstName} maxLength={80} value={f.first} onChange={(e) => setF({ ...f, first: e.target.value })} />
          <input className="field" placeholder={t.account.lastName} aria-label={t.account.lastName} maxLength={80} value={f.last} onChange={(e) => setF({ ...f, last: e.target.value })} />
        </div>
      )}
      <input className="field" type="email" required autoComplete="email" placeholder={t.account.email} aria-label={t.account.email} value={f.email} onChange={(e) => setF({ ...f, email: e.target.value })} />
      <input className="field" type="password" required minLength={mode === 'up' ? 8 : 1} autoComplete={mode === 'up' ? 'new-password' : 'current-password'} placeholder={t.account.password} aria-label={t.account.password} value={f.password} onChange={(e) => setF({ ...f, password: e.target.value })} />
      <ErrorLine error={error} />
      {msg && <p className="text-sm text-emerald-300">{msg}</p>}
      <div className="flex flex-wrap items-center gap-3">
        <button className={btnPrimary} disabled={busy}>{mode === 'in' ? t.account.signIn : t.account.signUp}</button>
        <button type="button" className="text-sm text-muted hover:text-white" onClick={() => setMode(mode === 'in' ? 'up' : 'in')}>
          {mode === 'in' ? `${t.account.noAccount} ${t.account.signUp}` : `${t.account.haveAccount} ${t.account.signIn}`}
        </button>
      </div>
    </form>
  )
}

export default function Account() {
  const { session, settings, setSetting } = useStore()
  const { t, lang } = useT()
  const [profile, setProfile] = useState<Profile | null>(null)
  const [sub, setSub] = useState<Subscription | null>(null)
  const [plans, setPlans] = useState<Plan[]>([])
  const [trialDays, setTrialDays] = useState<number | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [msg, setMsg] = useState<string | null>(null)

  useEffect(() => {
    if (!supabase || !session) return
    const uid = session.user.id
    Promise.all([
      supabase.from('profiles').select('id, first_name, last_name, assistant_name').eq('id', uid).maybeSingle(),
      supabase.from('subscriptions').select('plan_code, status, billing_interval, current_period_end').eq('user_id', uid).maybeSingle(),
      supabase.from('plans').select('code, name, tagline, features, features_it, sort').order('sort'),
      supabase.from('app_settings').select('value').eq('key', 'trial_days').maybeSingle(),
    ]).then(([p, s, pl, ts]) => {
      const err = p.error ?? s.error ?? pl.error
      if (err) setError(err.message)
      setProfile(p.data as Profile | null)
      setSub(s.data as Subscription | null)
      setPlans((pl.data ?? []) as Plan[])
      setTrialDays(typeof ts.data?.value === 'number' ? ts.data.value : null)
      // Online name is used only if this computer has none yet.
      const online = (p.data as Profile | null)?.assistant_name
      if (online && !settings.assistant_name) setSetting('assistant_name', online)
    })
  }, [session?.user.id]) // eslint-disable-line react-hooks/exhaustive-deps

  if (!supabase) return <div className="mx-auto max-w-3xl"><PageHeader title={t.account.title} /><Note>{t.account.notConfigured}</Note></div>
  if (!session) {
    return (
      <div className="mx-auto max-w-md space-y-4">
        <PageHeader title={t.account.title} />
        <Card><AuthForm /></Card>
        <Note>{t.account.offlineOk}</Note>
      </div>
    )
  }

  const plan = plans.find((p) => p.code === sub?.plan_code)
  const features = plan ? (lang === 'it' && plan.features_it?.length ? plan.features_it : plan.features) : []
  const date = (s: string | null) => (s ? new Date(s).toLocaleDateString(lang, { dateStyle: 'long' }) : '—')
  const live = sub && (sub.status === 'active' || sub.status === 'trialing')

  const saveProfile = async (e: FormEvent) => {
    e.preventDefault()
    if (!profile) return
    setMsg(null); setError(null)
    const { error } = await supabase!.from('profiles').update({ first_name: profile.first_name, last_name: profile.last_name }).eq('id', profile.id)
    if (error) setError(error.message); else setMsg(t.common.saved)
  }

  return (
    <div className="mx-auto max-w-4xl space-y-5">
      <PageHeader title={t.account.title}>
        <button className={btnGhost} onClick={() => supabase!.auth.signOut()}><LogOut size={15} />{t.account.signOut}</button>
      </PageHeader>
      <ErrorLine error={error} />
      <Card title={t.account.profile}>
        <p className="mb-3 text-sm">{session.user.email}</p>
        {profile && (
          <form onSubmit={saveProfile} className="flex flex-wrap gap-2">
            <input className="field !w-48" maxLength={80} aria-label={t.account.firstName} placeholder={t.account.firstName} value={profile.first_name} onChange={(e) => setProfile({ ...profile, first_name: e.target.value })} />
            <input className="field !w-48" maxLength={80} aria-label={t.account.lastName} placeholder={t.account.lastName} value={profile.last_name} onChange={(e) => setProfile({ ...profile, last_name: e.target.value })} />
            <button className={btnPrimary}>{t.common.save}</button>
            {msg && <span className="self-center text-xs text-emerald-300">{msg}</span>}
          </form>
        )}
      </Card>
      <div className="grid gap-5 md:grid-cols-2">
        <Card title={t.account.plan}>
          {plan && live ? (
            <>
              <div className="flex items-center gap-2"><span className="font-display text-xl font-bold text-gradient">{plan.name}</span><Badge tone={sub.status === 'active' ? 'ok' : 'info'}>{t.account.statuses[sub.status]}</Badge></div>
              <p className="mt-1 text-sm text-muted">{plan.tagline}</p>
              <div className="mt-3 text-xs uppercase tracking-wider text-muted">{t.account.features}</div>
              <ul className="mt-1 list-inside list-disc text-sm">{features.map((f) => <li key={f}>{f}</li>)}</ul>
            </>
          ) : <p className="text-sm text-muted">{t.account.noPlan}{sub ? ` (${t.account.statuses[sub.status]})` : ''}</p>}
        </Card>
        <Card title={t.account.trial}>
          <dl className="grid grid-cols-[140px_1fr] gap-y-1.5 text-sm">
            <dt className="text-muted">{t.account.trial}</dt>
            <dd>{sub?.status === 'trialing' ? fill(t.account.trialActive, { date: date(sub.current_period_end) }) : t.account.trialNone}</dd>
            {trialDays && <><dt className="text-muted" /><dd className="text-xs text-muted">{fill(t.account.trialLength, { n: trialDays })}</dd></>}
            <dt className="text-muted">{t.account.expiry}</dt>
            <dd>{sub?.status === 'trialing' || sub?.status === 'canceled' ? date(sub.current_period_end) : '—'}</dd>
            <dt className="text-muted">{t.account.renewal}</dt>
            <dd>{sub?.status === 'active' ? date(sub.current_period_end) : '—'}</dd>
          </dl>
        </Card>
      </div>
      <Card title={t.account.manage}>
        <Note>{t.account.paymentsOff}</Note>
        <div className="mt-3 flex gap-2">
          <button className={btnPrimary} disabled>{t.account.manage}</button>
          {siteUrl && <button className={btnGhost} onClick={() => openUrl(`${siteUrl}/account`)}><ExternalLink size={14} />{t.account.manageOnSite}</button>}
        </div>
      </Card>
    </div>
  )
}
