import { useState, type FormEvent, type ReactNode } from 'react'
import { supabase, type Announcement, type Daw, type Plan, type Release } from '../../lib/supabase'
import { useQuery } from '../../lib/useQuery'
import { btnGhost, btnPrimary } from '../../components/Layout'

// All writes here are authorised by RLS (public.is_admin()); the UI guard is only convenience.

const tabs = ['Users', 'Plans', 'DAWs', 'Releases', 'Announcements'] as const
type Tab = (typeof tabs)[number]

const small = `${btnGhost} !px-3 !py-1.5 !text-xs`
const Label = ({ text, children }: { text: string; children: ReactNode }) => (
  <label className="block text-sm"><span className="mb-1 block text-muted">{text}</span>{children}</label>
)
const Err = ({ msg }: { msg: string | null }) => (msg ? <p role="alert" className="text-sm text-red-300">{msg}</p> : null)

/** Runs a mutation, then reloads; returns the error message (if any) for display. */
function useMutate(reload: () => void) {
  const [error, setError] = useState<string | null>(null)
  const run = async (op: PromiseLike<{ error: { message: string } | null }>) => {
    const { error } = await op
    setError(error?.message ?? null)
    if (!error) reload()
    return !error
  }
  return { error, run }
}

function Users() {
  type Row = { id: string; email: string; first_name: string; last_name: string; created_at: string; email_confirmed: boolean; plan_code: string | null; status: string | null }
  const { data, error } = useQuery(() => supabase.rpc('admin_list_users'))
  const rows = (data ?? []) as Row[]
  return (
    <div className="overflow-x-auto">
      <Err msg={error} />
      <p className="mb-3 text-sm text-muted">{rows.length} users</p>
      <table className="w-full min-w-[640px] text-left text-sm">
        <thead className="text-muted"><tr>{['Name', 'Email', 'Verified', 'Plan', 'Status', 'Joined'].map((h) => <th key={h} className="py-2 font-medium">{h}</th>)}</tr></thead>
        <tbody className="divide-y divide-line">
          {rows.map((u) => (
            <tr key={u.id}>
              <td className="py-2">{u.first_name} {u.last_name}</td>
              <td>{u.email}</td>
              <td>{u.email_confirmed ? 'Yes' : 'No'}</td>
              <td>{u.plan_code ?? '—'}</td>
              <td>{u.status ?? '—'}</td>
              <td>{new Date(u.created_at).toLocaleDateString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

const toCents = (v: FormDataEntryValue | null) => (String(v ?? '').trim() === '' ? null : Math.round(Number(v) * 100))
const fromCents = (c: number | null) => (c == null ? '' : (c / 100).toFixed(2))

function Plans() {
  const { data, reload } = useQuery(() => supabase.from('plans').select('*').order('sort'))
  const { error, run } = useMutate(reload)
  const save = (code: string) => (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const d = new FormData(e.currentTarget)
    run(supabase.from('plans').update({
      name: String(d.get('name')),
      features: String(d.get('features')).split('\n').map((f) => f.trim()).filter(Boolean),
      price_monthly_cents: toCents(d.get('monthly')),
      price_yearly_cents: toCents(d.get('yearly')),
      stripe_price_monthly: String(d.get('stripe_m')).trim() || null,
      stripe_price_yearly: String(d.get('stripe_y')).trim() || null,
    }).eq('code', code))
  }
  return (
    <div className="space-y-6">
      <p className="text-sm text-muted">Leave a price empty to show "Price announced soon". Stripe price IDs are used once checkout is enabled.</p>
      <Err msg={error} />
      <div className="grid gap-6 lg:grid-cols-3">
        {((data ?? []) as Plan[]).map((p) => (
          <form key={p.code} onSubmit={save(p.code)} className="card space-y-3 p-5">
            <p className="font-mono text-xs uppercase tracking-widest text-sky">{p.tagline}</p>
            <Label text="Name"><input name="name" defaultValue={p.name} className="field" required /></Label>
            <div className="grid grid-cols-2 gap-3">
              <Label text={`Monthly (${p.currency})`}><input name="monthly" type="number" min="0" step="0.01" defaultValue={fromCents(p.price_monthly_cents)} className="field" /></Label>
              <Label text={`Yearly (${p.currency})`}><input name="yearly" type="number" min="0" step="0.01" defaultValue={fromCents(p.price_yearly_cents)} className="field" /></Label>
            </div>
            <Label text="Stripe price ID – monthly"><input name="stripe_m" defaultValue={p.stripe_price_monthly ?? ''} className="field font-mono text-xs" /></Label>
            <Label text="Stripe price ID – yearly"><input name="stripe_y" defaultValue={p.stripe_price_yearly ?? ''} className="field font-mono text-xs" /></Label>
            <Label text="Features (one per line)"><textarea name="features" rows={7} defaultValue={p.features.join('\n')} className="field" /></Label>
            <button className={btnPrimary}>Save</button>
          </form>
        ))}
      </div>
    </div>
  )
}

function Daws() {
  const { data, reload } = useQuery(() => supabase.from('daws').select('*').order('sort'))
  const { error, run } = useMutate(reload)
  const values = (d: FormData) => ({
    name: String(d.get('name')), vendor: String(d.get('vendor')), description: String(d.get('description')),
    status: String(d.get('status')) as Daw['status'], sort: Number(d.get('sort') || 0),
  })
  const save = (slug: string) => (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    run(supabase.from('daws').update(values(new FormData(e.currentTarget))).eq('slug', slug))
  }
  const add = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const form = e.currentTarget
    const d = new FormData(form)
    if (await run(supabase.from('daws').insert({ slug: String(d.get('slug')).trim().toLowerCase(), ...values(d) }))) form.reset()
  }
  const fields = (d?: Daw) => (
    <>
      <div className="grid grid-cols-2 gap-3">
        <Label text="Name"><input name="name" defaultValue={d?.name} className="field" required /></Label>
        <Label text="Vendor"><input name="vendor" defaultValue={d?.vendor} className="field" required /></Label>
      </div>
      <Label text="Description"><textarea name="description" rows={3} defaultValue={d?.description} className="field" required /></Label>
      <div className="grid grid-cols-2 gap-3">
        <Label text="Status">
          <select name="status" defaultValue={d?.status ?? 'planned'} className="field">
            <option value="planned">Planned</option><option value="in_development">In development</option><option value="available">Available (tested)</option>
          </select>
        </Label>
        <Label text="Order"><input name="sort" type="number" defaultValue={d?.sort ?? 10} className="field" /></Label>
      </div>
    </>
  )
  return (
    <div className="space-y-6">
      <Err msg={error} />
      <div className="grid gap-6 lg:grid-cols-2">
        {((data ?? []) as Daw[]).map((d) => (
          <form key={d.slug} onSubmit={save(d.slug)} className="card space-y-3 p-5">
            <p className="font-mono text-xs text-muted">{d.slug}</p>
            {fields(d)}
            <div className="flex gap-3">
              <button className={btnPrimary}>Save</button>
              <button type="button" className={btnGhost} onClick={() => confirm(`Delete ${d.name}?`) && run(supabase.from('daws').delete().eq('slug', d.slug))}>Delete</button>
            </div>
          </form>
        ))}
        <form onSubmit={add} className="card space-y-3 border-dashed p-5">
          <p className="font-display font-semibold">Add a DAW</p>
          <Label text="Slug (e.g. cubase)"><input name="slug" pattern="[a-z0-9-]+" className="field" required /></Label>
          {fields()}
          <button className={btnPrimary}>Add</button>
        </form>
      </div>
    </div>
  )
}

function Releases() {
  const { data, reload } = useQuery(() => supabase.from('releases').select('*').order('created_at', { ascending: false }))
  const { error, run } = useMutate(reload)
  const [busy, setBusy] = useState(false)
  const [uploadErr, setUploadErr] = useState<string | null>(null)

  const add = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const form = e.currentTarget
    const d = new FormData(form)
    const platform = String(d.get('platform'))
    const version = String(d.get('version')).trim()
    const file = d.get('file') as File | null
    setBusy(true); setUploadErr(null)
    let file_path: string | null = null
    if (file && file.size > 0) {
      file_path = `${platform}/${version}/${file.name}`
      const { error } = await supabase.storage.from('releases').upload(file_path, file, { upsert: false })
      if (error) { setBusy(false); return setUploadErr(error.message) }
    }
    const ok = await run(supabase.from('releases').insert({
      platform, version, file_path, notes: String(d.get('notes')), released_at: String(d.get('released_at')), published: false,
    }))
    setBusy(false)
    if (ok) form.reset()
  }

  return (
    <div className="space-y-6">
      <form onSubmit={add} className="card grid gap-3 p-5 md:grid-cols-2">
        <p className="font-display font-semibold md:col-span-2">New release (saved as draft)</p>
        <Label text="Platform"><select name="platform" className="field"><option value="macos">macOS</option><option value="windows">Windows</option></select></Label>
        <Label text="Version"><input name="version" placeholder="1.0.0" pattern="[0-9A-Za-z.\-]+" className="field" required /></Label>
        <Label text="Release date"><input name="released_at" type="date" defaultValue={new Date().toISOString().slice(0, 10)} className="field" required /></Label>
        <Label text="Installer (.dmg / .pkg / .exe / .msi)"><input name="file" type="file" accept=".dmg,.pkg,.zip,.exe,.msi" className="field" /></Label>
        <div className="md:col-span-2"><Label text="Release notes"><textarea name="notes" rows={4} className="field" /></Label></div>
        <Err msg={uploadErr ?? error} />
        <button className={`${btnPrimary} md:col-span-2 md:justify-self-start`} disabled={busy}>{busy ? 'Uploading…' : 'Create draft'}</button>
      </form>
      <ul className="divide-y divide-line text-sm">
        {((data ?? []) as Release[]).map((r) => (
          <li key={r.id} className="flex flex-wrap items-center justify-between gap-3 py-3">
            <div>
              <p className="font-semibold">{r.platform} · v{r.version} <span className={r.published ? 'text-emerald-300' : 'text-muted'}>({r.published ? 'published' : 'draft'})</span></p>
              <p className="font-mono text-xs text-muted">{r.file_path ?? 'no installer attached'}</p>
            </div>
            <div className="flex gap-2">
              <button className={small} onClick={() => run(supabase.from('releases').update({ published: !r.published }).eq('id', r.id))}>{r.published ? 'Unpublish' : 'Publish'}</button>
              <button className={small} onClick={async () => {
                if (!confirm(`Delete v${r.version}?`)) return
                if (r.file_path) await supabase.storage.from('releases').remove([r.file_path])
                run(supabase.from('releases').delete().eq('id', r.id))
              }}>Delete</button>
            </div>
          </li>
        ))}
      </ul>
    </div>
  )
}

function Announcements() {
  const { data, reload } = useQuery(() => supabase.from('announcements').select('*').order('created_at', { ascending: false }))
  const { error, run } = useMutate(reload)
  const add = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const form = e.currentTarget
    const d = new FormData(form)
    if (await run(supabase.from('announcements').insert({ title: String(d.get('title')), body: String(d.get('body')) }))) form.reset()
  }
  return (
    <div className="space-y-6">
      <form onSubmit={add} className="card space-y-3 p-5">
        <p className="font-display font-semibold">New announcement (shown in users' personal area once published)</p>
        <Label text="Title"><input name="title" className="field" required /></Label>
        <Label text="Message"><textarea name="body" rows={3} className="field" /></Label>
        <Err msg={error} />
        <button className={btnPrimary}>Create draft</button>
      </form>
      <ul className="divide-y divide-line text-sm">
        {((data ?? []) as Announcement[]).map((a) => (
          <li key={a.id} className="flex flex-wrap items-center justify-between gap-3 py-3">
            <p><span className="font-semibold">{a.title}</span> <span className={a.published ? 'text-emerald-300' : 'text-muted'}>({a.published ? 'published' : 'draft'})</span></p>
            <div className="flex gap-2">
              <button className={small} onClick={() => run(supabase.from('announcements').update({ published: !a.published }).eq('id', a.id))}>{a.published ? 'Unpublish' : 'Publish'}</button>
              <button className={small} onClick={() => confirm('Delete this announcement?') && run(supabase.from('announcements').delete().eq('id', a.id))}>Delete</button>
            </div>
          </li>
        ))}
      </ul>
    </div>
  )
}

const panels: Record<Tab, () => ReactNode> = { Users, Plans, DAWs: Daws, Releases, Announcements }

export default function Admin() {
  const [tab, setTab] = useState<Tab>('Users')
  const Panel = panels[tab]
  return (
    <div className="mx-auto max-w-7xl px-4 py-12 sm:px-6">
      <p className="font-mono text-xs uppercase tracking-[0.3em] text-violet">Restricted</p>
      <h1 className="mt-2 font-display text-3xl font-bold">Admin panel</h1>
      <div className="mt-8 flex gap-2 overflow-x-auto border-b border-line pb-px" role="tablist">
        {tabs.map((t) => (
          <button key={t} role="tab" aria-selected={tab === t} onClick={() => setTab(t)}
            className={`shrink-0 border-b-2 px-4 py-2.5 text-sm transition ${tab === t ? 'border-violet text-white' : 'border-transparent text-muted hover:text-white'}`}>
            {t}
          </button>
        ))}
      </div>
      <div className="mt-8"><Panel /></div>
    </div>
  )
}
