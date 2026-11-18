import { useState, type FormEvent, type ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { Download, Laptop, Megaphone, User } from 'lucide-react'
import { supabase, type Announcement, type Device, type Release, type Subscription } from '../../lib/supabase'
import { useAuth } from '../../lib/auth'
import { useQuery } from '../../lib/useQuery'
import { useT } from '../../i18n'
import { btnGhost, btnPrimary } from '../../components/Layout'

function Panel({ id, title, icon, children }: { id?: string; title: string; icon: ReactNode; children: ReactNode }) {
  return (
    <section id={id} className="card scroll-mt-20 p-6">
      <h2 className="mb-5 flex items-center gap-2 font-display text-lg font-semibold"><span className="text-sky">{icon}</span>{title}</h2>
      {children}
    </section>
  )
}

export default function Dashboard() {
  const { session } = useAuth()
  const { t } = useT()
  const a = t.account
  const date = (d: string) => new Date(d).toLocaleDateString(t.locale)
  const user = session!.user
  const profile = useQuery(() => supabase.from('profiles').select('first_name,last_name,created_at').eq('id', user.id).single(), [user.id])
  const sub = useQuery(() => supabase.from('subscriptions').select('plan_code,status,billing_interval,current_period_end').eq('user_id', user.id).maybeSingle(), [user.id])
  const plans = useQuery(() => supabase.from('plans').select('code,name'))
  const releases = useQuery(() => supabase.from('releases').select('*').eq('published', true).order('released_at', { ascending: false }))
  const devices = useQuery(() => supabase.from('devices').select('*').order('created_at', { ascending: false }), [user.id])
  const news = useQuery(() => supabase.from('announcements').select('*').eq('published', true).order('created_at', { ascending: false }).limit(3))

  const [saving, setSaving] = useState(false)
  const [msg, setMsg] = useState<string | null>(null)
  const [dlError, setDlError] = useState<string | null>(null)

  const p = profile.data as { first_name: string; last_name: string; created_at: string } | null
  const s = sub.data as Subscription | null
  const planName = (plans.data as { code: string; name: string }[] | null)?.find((x) => x.code === s?.plan_code)?.name
  const rel = (releases.data ?? []) as Release[]
  const devs = (devices.data ?? []) as Device[]
  const entitled = s?.status === 'active' || s?.status === 'trialing'

  const saveProfile = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const d = new FormData(e.currentTarget)
    setSaving(true); setMsg(null)
    const { error } = await supabase.from('profiles')
      .update({ first_name: String(d.get('first_name')).trim(), last_name: String(d.get('last_name')).trim() })
      .eq('id', user.id)
    setSaving(false)
    setMsg(error ? error.message : t.common.saved)
    if (!error) profile.reload()
  }

  const download = async (r: Release) => {
    setDlError(null)
    const { data, error } = await supabase.storage.from('releases').createSignedUrl(r.file_path!, 60)
    if (error) return setDlError(a.dlError)
    window.location.href = data.signedUrl
  }

  return (
    <div className="mx-auto max-w-6xl px-4 py-12 sm:px-6">
      <p className="font-mono text-xs uppercase tracking-[0.3em] text-sky">{a.eyebrow}</p>
      <h1 className="mt-2 font-display text-3xl font-bold">{a.welcome}{p?.first_name ? `, ${p.first_name}` : ''}</h1>

      {(news.data as Announcement[] | null)?.map((n) => (
        <div key={n.id} className="mt-6 flex gap-3 rounded-xl border border-violet/40 bg-violet/10 p-4 text-sm">
          <Megaphone size={18} className="shrink-0 text-violet" />
          <div><p className="font-semibold">{n.title}</p>{n.body && <p className="mt-1 whitespace-pre-line text-muted">{n.body}</p>}</div>
        </div>
      ))}

      <div className="mt-8 grid gap-6 lg:grid-cols-2">
        <Panel title={a.account} icon={<User size={18} />}>
          {p && (
            <form onSubmit={saveProfile} className="space-y-4">
              <div className="grid gap-4 sm:grid-cols-2">
                <label className="text-sm"><span className="mb-1.5 block text-muted">{t.auth.firstName}</span>
                  <input name="first_name" defaultValue={p.first_name} maxLength={80} className="field" /></label>
                <label className="text-sm"><span className="mb-1.5 block text-muted">{t.auth.lastName}</span>
                  <input name="last_name" defaultValue={p.last_name} maxLength={80} className="field" /></label>
              </div>
              <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
                <dt className="text-muted">{t.auth.email}</dt><dd className="truncate">{user.email}</dd>
                <dt className="text-muted">{a.verified}</dt><dd>{user.email_confirmed_at ? a.yes : a.no}</dd>
                <dt className="text-muted">{a.since}</dt><dd>{date(p.created_at)}</dd>
              </dl>
              <div className="flex items-center gap-4">
                <button className={btnGhost} disabled={saving}>{saving ? t.common.saving : a.saveChanges}</button>
                {msg && <span role="status" className="text-sm text-muted">{msg}</span>}
              </div>
              <Link to="/forgot-password" className="block text-sm text-muted hover:text-white">{a.changePw}</Link>
            </form>
          )}
        </Panel>

        <Panel title={a.subscription} icon={<span className="font-mono text-sm">◆</span>}>
          {s ? (
            <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
              <dt className="text-muted">{a.plan}</dt><dd className="font-semibold">{planName ?? s.plan_code}</dd>
              <dt className="text-muted">{a.status}</dt><dd>{a.subStatus[s.status]}</dd>
              {s.billing_interval && <><dt className="text-muted">{a.billing}</dt><dd>{s.billing_interval === 'year' ? t.pricing.yearly : t.pricing.monthly}</dd></>}
              {s.current_period_end && <><dt className="text-muted">{a.renews}</dt><dd>{date(s.current_period_end)}</dd></>}
            </dl>
          ) : (
            <>
              <p className="text-sm text-muted">{a.noSub}</p>
              <Link to="/pricing" className={`${btnPrimary} mt-5`}>{a.seePlans}</Link>
            </>
          )}
        </Panel>

        <Panel id="downloads" title={a.downloads} icon={<Download size={18} />}>
          {rel.length === 0 ? (
            <p className="font-display text-sm font-semibold tracking-[0.25em] text-gradient">{t.download.comingSoon}</p>
          ) : (
            <ul className="divide-y divide-line text-sm">
              {rel.map((r) => (
                <li key={r.id} className="flex items-center justify-between gap-4 py-3">
                  <div>
                    <p className="font-semibold">{r.platform === 'macos' ? 'macOS' : 'Windows'} · v{r.version}</p>
                    <p className="text-xs text-muted">{date(r.released_at)}</p>
                    {r.notes && <p className="mt-1 whitespace-pre-line text-xs text-muted">{r.notes}</p>}
                  </div>
                  {r.file_path && entitled && <button onClick={() => download(r)} className={`${btnGhost} !px-4 !py-2`}>{a.downloadBtn}</button>}
                </li>
              ))}
            </ul>
          )}
          {rel.length > 0 && !entitled && <p className="mt-3 text-xs text-muted">{a.needSub}</p>}
          {dlError && <p role="alert" className="mt-3 text-sm text-red-300">{dlError}</p>}
        </Panel>

        <Panel title={a.devices} icon={<Laptop size={18} />}>
          {devs.length === 0 ? (
            <p className="text-sm text-muted">{a.noDevices}</p>
          ) : (
            <ul className="divide-y divide-line text-sm">
              {devs.map((d) => (
                <li key={d.id} className="flex justify-between py-3">
                  <span>{d.name} <span className="text-muted">· {d.platform}</span></span>
                  <span className="text-xs text-muted">{d.revoked_at ? a.revoked : d.last_seen_at ? `${a.lastSeen} ${date(d.last_seen_at)}` : a.never}</span>
                </li>
              ))}
            </ul>
          )}
        </Panel>
      </div>
    </div>
  )
}
