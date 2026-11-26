import { useEffect, useState } from 'react'
import { Ban, FolderMinus, FolderPlus, HardDrive, RefreshCw, ScanLine, Trash2, Usb, Network, Folder } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { api, errText, fmtBytes, fmtDate, type LibOverview } from '../lib/api'
import { useStore } from '../lib/store'
import { useT } from '../i18n'
import { Badge, Card, Dot, Empty, ErrorLine, Note, PageHeader, btnGhost, btnIcon, btnPrimary, useLoad } from '../components/ui'
import { DuplicatesCard, IndexStatus, LoopsCard, SmartSearch } from '../components/SmartLibrary'

const kindIcon = { internal: HardDrive, external: Usb, network: Network, folder: Folder }

/** Pick a folder and authorize it. Shared with onboarding. */
export async function authorizeFolder(): Promise<string | null> {
  const dir = await open({ directory: true, multiple: false })
  if (typeof dir !== 'string') return null
  return api.addRoot(dir)
}

export default function Library() {
  const { settings, setSetting } = useStore()
  const { t, lang } = useT()
  const showExternal = settings.perm_external_devices !== '0'
  const volumes = useLoad(() => api.volumes(), [])
  const roots = useLoad(() => api.roots(), [])
  const [progress, setProgress] = useState<Record<string, number>>({})
  const [error, setError] = useState<string | null>(null)
  const [ov, setOv] = useState<LibOverview | null>(null)
  const [excl, setExcl] = useState<Record<string, string>>({})
  const loadOv = () => api.libOverview().then(setOv, () => {})

  // Plug/unplug detection: poll the mounted volumes while the page is open.
  useEffect(() => {
    loadOv()
    const h = setInterval(() => { volumes.reload(); roots.reload(); loadOv() }, 5000)
    return () => clearInterval(h)
  }, [volumes.reload, roots.reload]) // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const un = listen<{ root_id: string; count: number; done: boolean; error: string | null }>('library-scan', (e) => {
      setProgress((p) => ({ ...p, [e.payload.root_id]: e.payload.count }))
      if (e.payload.done) { roots.reload(); volumes.reload(); loadOv(); if (e.payload.error) setError(e.payload.error) }
    })
    const un2 = listen('library-analysis', () => loadOv())
    return () => { un.then((f) => f()); un2.then((f) => f()) }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const act = async (fn: () => Promise<unknown>) => {
    setError(null)
    try { await fn() } catch (e) { setError(errText(e)) }
    roots.reload(); volumes.reload(); loadOv()
  }

  const scan = (id: string) => act(async () => { setProgress((p) => ({ ...p, [id]: 0 })); await api.scan(id); setTimeout(roots.reload, 100) })
  const vols = (volumes.data ?? []).filter((v) => showExternal || v.kind === 'internal' || v.root_count > 0)

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.library.title}>
        <button className={btnGhost} onClick={() => act(async () => {})}><RefreshCw size={15} />{t.library.refresh}</button>
        <button className={btnPrimary} onClick={() => act(authorizeFolder)}><FolderPlus size={15} />{t.library.addFolder}</button>
      </PageHeader>
      <ErrorLine error={error ?? volumes.error ?? roots.error} />
      <SmartSearch />
      <IndexStatus ov={ov} paused={settings.library_analysis === 'paused'} onPause={(p) => act(() => setSetting('library_analysis', p ? 'paused' : 'on'))} onScanAll={() => act(() => api.scan(''))} onStop={() => act(() => api.libCancel())} />

      <Card title={t.library.devices}>
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {vols.map((v) => {
            const Icon = kindIcon[v.kind]
            const used = v.total_bytes && v.available_bytes != null ? 1 - v.available_bytes / v.total_bytes : null
            return (
              <div key={v.id} className={`rounded-xl border p-3 ${v.online ? 'border-line bg-panel-2/60' : 'border-dashed border-line opacity-70'}`} data-volume={v.label}>
                <div className="flex items-center gap-2">
                  <Icon size={18} className={v.online ? 'text-sky' : 'text-muted'} />
                  <span className="min-w-0 flex-1 truncate font-medium" title={v.mount_point ?? ''}>{v.label}</span>
                  <Dot on={v.online} />
                </div>
                <div className="mt-1.5 flex flex-wrap gap-1.5 text-[11px]">
                  <Badge>{t.library.kinds[v.kind]}</Badge>
                  {v.disk_type && v.disk_type !== 'unknown' && <Badge>{v.disk_type}</Badge>}
                  {v.fs_type && <Badge>{v.fs_type}</Badge>}
                  <Badge tone={v.online ? 'ok' : 'warn'}>{v.online ? t.common.online : t.common.offline}</Badge>
                  {v.excluded && <Badge tone="err">{t.smart.deviceExcluded}</Badge>}
                </div>
                {used != null && (
                  <div className="mt-2">
                    <div className="h-1.5 overflow-hidden rounded-full bg-white/5"><div className="h-full bg-gradient-to-r from-electric to-violet" style={{ width: `${used * 100}%` }} /></div>
                    <div className="mt-1 text-[11px] text-muted">{fmtBytes(v.available_bytes)} {t.library.free} {t.library.of} {fmtBytes(v.total_bytes)}</div>
                  </div>
                )}
                <div className="mt-1 flex items-center gap-2 text-[11px] text-muted">
                  <span className="flex-1">{v.root_count} · {v.file_count} {t.library.files}{!v.online && v.last_seen_at ? ` · ${fmtDate(v.last_seen_at, lang)}` : ''}</span>
                  {v.root_count > 0 && <button className="inline-flex items-center gap-1 hover:text-white" onClick={() => act(() => api.libExcludeDevice(v.id, !v.excluded))}><Ban size={11} />{v.excluded ? t.smart.includeDevice : t.smart.excludeDevice}</button>}
                </div>
              </div>
            )
          })}
        </div>
        <div className="mt-3"><Note>{t.library.reconnectHint}</Note></div>
      </Card>

      <Card title={t.library.folders}>
        {roots.data?.length === 0 ? <Empty>{t.library.noFolders}</Empty> : (
          <ul className="divide-y divide-line/60">
            {roots.data?.map((r) => {
              const scanning = r.scan_state === 'scanning'
              const state = !r.online ? 'offline' : r.scan_state
              return (
                <li key={r.id} className="flex flex-wrap items-center gap-3 py-2.5 text-sm">
                  <Dot on={r.online} />
                  <div className="min-w-0 flex-1">
                    <div className="truncate selectable" title={r.path}>{r.path}</div>
                    <div className="text-[11px] text-muted">{r.volume_label} · {t.library.lastScan}: {fmtDate(r.last_scan_at, lang)}{r.error ? ` · ${r.error}` : ''}</div>
                  </div>
                  <Badge tone={state === 'done' ? 'ok' : state === 'offline' || state === 'error' ? 'warn' : scanning ? 'info' : 'off'}>{t.library.state[state]}</Badge>
                  <span className="w-28 text-right text-xs text-muted">{(scanning ? progress[r.id] ?? 0 : r.file_count).toLocaleString(lang)} {t.library.files}</span>
                  <button className={btnGhost + ' !px-3 !py-1 !text-xs'} disabled={!r.online || scanning} onClick={() => scan(r.id)}><ScanLine size={13} />{scanning ? t.library.scanning : t.library.scan}</button>
                  <button className={btnIcon} title={t.common.remove} aria-label={t.common.remove} onClick={() => confirm(t.library.revokeConfirm) && act(() => api.removeRoot(r.id))}><Trash2 size={14} /></button>
                  <div className="flex w-full flex-wrap items-center gap-2 pl-5 text-xs">
                    {ov?.exclusions.filter((x) => x.root_id === r.id).map((x) => (
                      <Badge key={x.folder} tone="off"><FolderMinus size={11} />{x.folder}<button className="ml-1 hover:text-white" aria-label={t.common.remove} onClick={() => act(() => api.libExclude(r.id, x.folder, false))}>×</button></Badge>
                    ))}
                    <form className="flex gap-1" onSubmit={(e) => { e.preventDefault(); const f = (excl[r.id] ?? '').trim(); if (f) act(async () => { await api.libExclude(r.id, f, true); setExcl({ ...excl, [r.id]: '' }) }) }}>
                      <input className="field !w-56 !py-0.5 !text-xs" placeholder={t.smart.excludePlaceholder} aria-label={t.smart.exclude} value={excl[r.id] ?? ''} onChange={(e) => setExcl({ ...excl, [r.id]: e.target.value })} />
                      <button className={btnGhost + ' !px-2 !py-0.5 !text-xs'}>{t.smart.exclude}</button>
                    </form>
                    {(ov?.root_states.find((x) => x.root_id === r.id)?.missing_files ?? 0) > 0 && <Badge tone="err">{ov?.root_states.find((x) => x.root_id === r.id)?.missing_files} {t.smart.missing}</Badge>}
                  </div>
                </li>
              )
            })}
          </ul>
        )}
        <div className="mt-3"><Note>{t.library.readOnly}</Note></div>
      </Card>

      <div className="grid gap-5 lg:grid-cols-2">
        <LoopsCard />
        <DuplicatesCard />
      </div>
    </div>
  )
}
