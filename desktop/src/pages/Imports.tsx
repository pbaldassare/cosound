import { useState } from 'react'
import { FilePlus2, FolderSearch, Trash2 } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { api, errText, fmtBytes, fmtDate, type Provenance, type Role } from '../lib/api'
import { useStore } from '../lib/store'
import { fill, useT } from '../i18n'
import { Badge, Card, Empty, ErrorLine, Note, PageHeader, btnIcon, btnPrimary, useLoad } from '../components/ui'

const PROVENANCES: Provenance[] = ['own', 'suno', 'udio', 'other_ai', 'unknown']
const ROLES: Role[] = ['track', 'stem', 'loop', 'sample', 'voice', 'midi', 'project', 'preset']
const EXTENSIONS = ['wav', 'wave', 'aif', 'aiff', 'mp3', 'flac', 'ogg', 'm4a', 'caf', 'mid', 'midi', 'als', 'flp', 'ptx', 'cpr', 'rpp', 'dawproject']

export default function Imports() {
  const { project } = useStore()
  const { t, lang } = useT()
  const [provenance, setProvenance] = useState<Provenance>('own')
  const [role, setRole] = useState<Role | ''>('')
  const [error, setError] = useState<string | null>(null)
  const [msg, setMsg] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const list = useLoad(() => api.imports(project?.id ?? null), [project?.id])

  async function choose() {
    setError(null); setMsg(null)
    const picked = await open({ multiple: true, directory: false, filters: [{ name: t.imports.filter, extensions: EXTENSIONS }] })
    if (!picked || picked.length === 0) return
    setBusy(true)
    try {
      const ids = await api.importFiles(project?.id ?? null, picked, provenance, role || null)
      setMsg(fill(t.imports.imported, { n: ids.length }))
      list.reload()
    } catch (e) { setError(errText(e)) } finally { setBusy(false) }
  }

  const remove = async (id: string) => {
    if (!confirm(t.imports.removeConfirm)) return
    try { await api.removeImport(id) } catch (e) { setError(errText(e)) }
    list.reload()
  }

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.imports.title} />
      <Card>
        <div className="flex flex-wrap items-end gap-3">
          <label className="flex flex-col text-xs text-muted">{t.imports.provenance}
            <select className="field mt-1 !w-52" value={provenance} onChange={(e) => setProvenance(e.target.value as Provenance)}>
              {PROVENANCES.map((p) => <option key={p} value={p}>{t.imports.provenances[p]}</option>)}
            </select>
          </label>
          <label className="flex flex-col text-xs text-muted">{t.imports.role}
            <select className="field mt-1 !w-48" value={role} onChange={(e) => setRole(e.target.value as Role | '')}>
              <option value="">{t.imports.auto}</option>
              {ROLES.map((r) => <option key={r} value={r}>{t.imports.roles[r]}</option>)}
            </select>
          </label>
          <button className={btnPrimary} onClick={choose} disabled={busy}><FilePlus2 size={15} />{busy ? t.common.loading : t.imports.import}</button>
          <span className="text-sm text-muted">{t.imports.project}: <span className="text-white">{project?.name ?? '—'}</span></span>
        </div>
        <div className="mt-3 space-y-2">
          {!project && <Note>{t.imports.noProjectNote}</Note>}
          <Note>{t.imports.note}</Note>
          {msg && <p className="text-sm text-emerald-300">{msg}</p>}
          <ErrorLine error={error ?? list.error} />
        </div>
      </Card>
      <Card title={t.imports.title}>
        {list.data?.length === 0 ? <Empty>{t.imports.empty}</Empty> : (
          <ul className="divide-y divide-line/60 text-sm">
            {list.data?.map((i) => (
              <li key={i.id} className="flex flex-wrap items-center gap-2 py-2">
                <div className="min-w-0 flex-1">
                  <div className="truncate selectable" title={i.path}>{i.name}</div>
                  <div className="truncate font-mono text-[10px] text-muted selectable" title={i.sha256}>sha256 {i.sha256.slice(0, 16)}… · {fmtDate(i.imported_at, lang)}</div>
                </div>
                <Badge>{t.imports.roles[i.role]}</Badge>
                <Badge tone={i.provenance === 'own' ? 'off' : 'violet'}>{t.imports.provenances[i.provenance]}</Badge>
                {!i.exists && <Badge tone="warn">{t.imports.missing}</Badge>}
                <span className="w-16 text-right text-xs text-muted">{fmtBytes(i.size)}</span>
                <button className={btnIcon} disabled={!i.exists} title={t.common.reveal} aria-label={t.common.reveal} onClick={() => revealItemInDir(i.path)}><FolderSearch size={14} /></button>
                <button className={btnIcon} title={t.common.remove} aria-label={t.common.remove} onClick={() => remove(i.id)}><Trash2 size={14} /></button>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  )
}
