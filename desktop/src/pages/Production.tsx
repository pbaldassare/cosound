import { useEffect, useState, type FormEvent } from 'react'
import { Archive, ArchiveRestore, Copy, FolderOpen, Pencil, Plus } from 'lucide-react'
import { api, errText, fmtBytes, fmtDate, type Project } from '../lib/api'
import { useStore } from '../lib/store'
import { useT } from '../i18n'
import { OpList } from '../components/OpList'
import { Badge, Card, Empty, ErrorLine, Note, PageHeader, btnGhost, btnIcon, btnPrimary, useLoad } from '../components/ui'

export const DAW_IDS = ['logic', 'ableton', 'fl', 'protools', 'cubase', 'reaper'] as const
const num = (s: string) => (s.trim() === '' ? null : Number(s))

export default function Production() {
  const { project, openProject, go } = useStore()
  const { t, lang } = useT()
  const [showArchived, setShowArchived] = useState(false)
  const list = useLoad(() => api.projects(showArchived), [showArchived])
  const [error, setError] = useState<string | null>(null)
  const [form, setForm] = useState({ name: '', daw: '', bpm: '', key: '' })

  const run = async (fn: () => Promise<unknown>) => {
    setError(null)
    try { await fn() } catch (e) { setError(errText(e)) }
    await list.reload()
  }

  const create = (e: FormEvent) => {
    e.preventDefault()
    run(async () => {
      const p = await api.createProject({ name: form.name, daw: form.daw || null, bpm: num(form.bpm), musicalKey: form.key || null })
      setForm({ name: '', daw: '', bpm: '', key: '' })
      await openProject(p.id)
    })
  }

  const rename = (p: Project) => {
    const name = prompt(t.production.rename, p.name)
    if (name && name !== p.name) run(async () => { await api.renameProject(p.id, name); if (project?.id === p.id) await openProject(p.id) })
  }

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.production.title} />
      <div className="grid gap-5 lg:grid-cols-[380px_minmax(0,1fr)]">
        <div className="space-y-5">
          <Card title={t.production.newProject}>
            <form onSubmit={create} className="space-y-2.5">
              <input className="field" required maxLength={120} placeholder={t.production.name} aria-label={t.production.name} value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
              <div className="grid grid-cols-3 gap-2">
                <select className="field" aria-label={t.production.daw} value={form.daw} onChange={(e) => setForm({ ...form, daw: e.target.value })}>
                  <option value="">{t.production.daw}</option>
                  {DAW_IDS.map((d) => <option key={d} value={d}>{t.production.daws[d]}</option>)}
                </select>
                <input className="field" type="number" min={20} max={400} step="0.01" placeholder="BPM" aria-label="BPM" value={form.bpm} onChange={(e) => setForm({ ...form, bpm: e.target.value })} />
                <input className="field" maxLength={20} placeholder={t.production.key} aria-label={t.production.key} value={form.key} onChange={(e) => setForm({ ...form, key: e.target.value })} />
              </div>
              <button className={btnPrimary + ' w-full'}><Plus size={15} />{t.production.create}</button>
            </form>
          </Card>
          <Card title={t.production.recent} actions={
            <label className="flex items-center gap-1.5 text-xs text-muted"><input type="checkbox" checked={showArchived} onChange={(e) => setShowArchived(e.target.checked)} />{t.production.showArchived}</label>
          }>
            <ErrorLine error={error ?? list.error} />
            {list.data?.length === 0 && <Empty>{t.production.noProjects}</Empty>}
            <ul className="space-y-1">
              {list.data?.map((p) => (
                <li key={p.id} className={`group flex items-center gap-2 rounded-lg px-2 py-1.5 ${project?.id === p.id ? 'bg-violet/10' : 'hover:bg-white/5'}`}>
                  <button className="min-w-0 flex-1 text-left" onClick={() => openProject(p.id)}>
                    <div className="truncate text-sm font-medium">{p.name}</div>
                    <div className="text-[11px] text-muted">{[p.daw && t.production.daws[p.daw as keyof typeof t.production.daws], p.bpm && `${p.bpm} BPM`, p.musical_key].filter(Boolean).join(' · ') || fmtDate(p.created_at, lang)}</div>
                  </button>
                  {project?.id === p.id && <Badge tone="violet">{t.production.current}</Badge>}
                  {p.archived_at && <Badge>{t.production.archived}</Badge>}
                  <button className={btnIcon} title={t.production.rename} aria-label={t.production.rename} onClick={() => rename(p)}><Pencil size={14} /></button>
                  <button className={btnIcon} title={t.production.duplicate} aria-label={t.production.duplicate} onClick={() => run(() => api.duplicateProject(p.id, t.production.copySuffix))}><Copy size={14} /></button>
                  <button className={btnIcon} title={p.archived_at ? t.production.unarchive : t.production.archive} aria-label={p.archived_at ? t.production.unarchive : t.production.archive}
                    onClick={() => run(async () => { await api.archiveProject(p.id, !p.archived_at); if (project?.id === p.id && !p.archived_at) await openProject(null) })}>
                    {p.archived_at ? <ArchiveRestore size={14} /> : <Archive size={14} />}
                  </button>
                </li>
              ))}
            </ul>
          </Card>
        </div>
        {project ? <ProjectDetail key={project.id} p={project} onChanged={list.reload} /> : (
          <Card><Empty>{t.common.noProject}</Empty><div className="mt-3 text-center"><button className={btnGhost} onClick={() => go('library')}><FolderOpen size={15} />{t.nav.library}</button></div></Card>
        )}
      </div>
    </div>
  )
}

function ProjectDetail({ p, onChanged }: { p: Project; onChanged: () => void }) {
  const { openProject, go } = useStore()
  const { t } = useT()
  const [f, setF] = useState({ daw: p.daw ?? '', bpm: p.bpm?.toString() ?? '', key: p.musical_key ?? '', notes: p.notes })
  const [msg, setMsg] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const imports = useLoad(() => api.imports(p.id), [p.id])
  const roots = useLoad(() => api.roots(), [])
  const linked = useLoad(() => api.projectRoots(p.id), [p.id])
  const history = useLoad(() => api.ops(p.id, 100), [p.id])
  useEffect(() => { if (msg) { const h = setTimeout(() => setMsg(null), 2000); return () => clearTimeout(h) } }, [msg])

  const save = async (e: FormEvent) => {
    e.preventDefault()
    setError(null)
    try {
      await api.saveProject(p.id, { daw: f.daw || null, bpm: num(f.bpm), musical_key: f.key || null, notes: f.notes })
      setMsg(t.common.saved)
      await openProject(p.id)
      onChanged()
    } catch (err) { setError(errText(err)) }
  }

  return (
    <div className="min-w-0 space-y-5">
      <Card title={t.production.details}>
        <h2 className="mb-3 font-display text-xl font-semibold">{p.name}</h2>
        <form onSubmit={save} className="space-y-2.5">
          <div className="grid grid-cols-3 gap-2">
            <label className="text-xs text-muted">{t.production.daw}
              <select className="field mt-1" value={f.daw} onChange={(e) => setF({ ...f, daw: e.target.value })}>
                <option value="">—</option>
                {DAW_IDS.map((d) => <option key={d} value={d}>{t.production.daws[d]}</option>)}
              </select>
            </label>
            <label className="text-xs text-muted">BPM<input className="field mt-1" type="number" min={20} max={400} step="0.01" value={f.bpm} onChange={(e) => setF({ ...f, bpm: e.target.value })} /></label>
            <label className="text-xs text-muted">{t.production.key}<input className="field mt-1" maxLength={20} placeholder={t.production.unknownKey} value={f.key} onChange={(e) => setF({ ...f, key: e.target.value })} /></label>
          </div>
          <label className="block text-xs text-muted">{t.production.notes}<textarea className="field mt-1 h-20 resize-none" maxLength={4000} value={f.notes} onChange={(e) => setF({ ...f, notes: e.target.value })} /></label>
          <div className="flex items-center gap-3"><button className={btnPrimary}>{t.common.save}</button>{msg && <span className="text-xs text-emerald-300">{msg}</span>}<ErrorLine error={error} /></div>
        </form>
      </Card>

      <Card title={t.production.materials} actions={<button className={btnGhost + ' !py-1 !text-xs'} onClick={() => go('imports')}>{t.production.addMaterial}</button>}>
        {imports.data?.length === 0 ? <Empty>{t.imports.empty}</Empty> : (
          <ul className="space-y-1 text-sm">
            {imports.data?.map((i) => (
              <li key={i.id} className="flex items-center gap-2">
                <span className="min-w-0 flex-1 truncate selectable" title={i.path}>{i.name}</span>
                <Badge>{t.imports.roles[i.role]}</Badge>
                <Badge tone={i.provenance === 'own' ? 'off' : 'violet'}>{t.imports.provenances[i.provenance]}</Badge>
                <span className="w-16 text-right text-xs text-muted">{fmtBytes(i.size)}</span>
                {!i.exists && <Badge tone="warn">{t.imports.missing}</Badge>}
              </li>
            ))}
          </ul>
        )}
        <div className="mt-3"><Note>{t.production.referencesNote}</Note></div>
      </Card>

      <Card title={t.production.libraries}>
        {roots.data?.length === 0 ? <Empty>{t.production.noLibraries}</Empty> : (
          <ul className="space-y-1 text-sm">
            {roots.data?.map((r) => (
              <li key={r.id}>
                <label className="flex items-center gap-2">
                  <input type="checkbox" checked={!!linked.data?.includes(r.id)} onChange={async (e) => { await api.linkRoot(p.id, r.id, e.target.checked); linked.reload(); history.reload() }} />
                  <span className="truncate selectable">{r.path}</span>
                  <span className="whitespace-nowrap text-xs text-muted">{r.file_count} {t.library.files}</span>
                </label>
              </li>
            ))}
          </ul>
        )}
      </Card>

      <Card title={t.production.history}>
        <OpList ops={history.data ?? []} empty={t.production.noHistory} onChange={async () => { await history.reload(); await openProject(p.id); onChanged() }} />
      </Card>
    </div>
  )
}
