import { useEffect, useState } from 'react'
import { Copy, FolderSearch, Pause, Play, Repeat, Search, Sparkles, X } from 'lucide-react'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { api, errText, fmtBytes, type Duplicates, type LibFile, type LibOverview, type LibQuery, type Understood } from '../lib/api'
import { fill, useT } from '../i18n'
import { useStore } from '../lib/store'
import { Badge, Card, Empty, ErrorLine, Note, btnGhost, btnIcon, btnPrimary } from './ui'

const pct = (c: number | null | undefined) => (c == null ? '' : ` · ${Math.round(c * 100)}%`)

// The engine explains scores in English; translate the fixed phrases for the Italian UI.
const WHY_IT: [RegExp, string][] = [
  [/^same key$/, 'stessa tonalità'], [/^relative key$/, 'tonalità relativa'], [/^parallel key$/, 'tonalità parallela'],
  [/^different key/, 'tonalità diversa'], [/^minor key/, 'tonalità minore'], [/^major key/, 'tonalità maggiore'],
  [/^not minor/, 'non minore'], [/^not major/, 'non maggiore'], [/^key unknown$/, 'tonalità sconosciuta'],
  [/^one-shot: tempo not relevant$/, 'one-shot: tempo non rilevante'], [/^tempo unknown$/, 'tempo sconosciuto'],
  [/BPM in range$/, "BPM nell'intervallo"], [/BPM out of range$/, 'BPM fuori intervallo'], [/\(half\/double time\)/, '(metà/doppio tempo)'],
  [/\(close, stretchable\)/, '(vicino, adattabile)'], [/^category /, 'categoria '], [/^genre label/, 'genere'],
  [/ bars$/, ' battute'], [/^preset for /, 'preset per '], [/^text match /, 'testo trovato '],
]
const whyText = (s: string, lang: string) => (lang === 'it' ? WHY_IT.reduce((acc, [re, to]) => acc.replace(re, to), s) : s)

function Est({ label, v, unit }: { label: string; v: { value: string | number | null; source: string | null; confidence: number | null }; unit?: string }) {
  const { t } = useT()
  return (
    <span className="text-xs" title={v.source ? `${t.smart.source[v.source] ?? v.source}${pct(v.confidence)}` : undefined}>
      <span className="text-muted">{label} </span>
      {v.value == null ? <span className="text-muted">{t.smart.unknown}</span> : <span className={v.source === 'user' ? 'text-violet' : 'text-white'}>{v.value}{unit}</span>}
      {v.value != null && v.source && <span className="ml-1 text-[10px] text-muted">({t.smart.source[v.source] ?? v.source}{pct(v.confidence)})</span>}
    </span>
  )
}

function Chips({ u }: { u: Understood }) {
  const { t } = useT()
  const chips: string[] = [
    ...u.categories, ...u.types, ...u.genres.map((g) => g.replace(/_/g, ' ')), ...u.descriptors,
    ...(u.key ? [u.key] : u.mode ? [u.mode] : []), ...(u.bpm ? [`${u.bpm} BPM`] : []), ...(u.bpm_range ? [`${u.bpm_range[0]}–${u.bpm_range[1]} BPM`] : []),
    ...(u.bars ? [`${u.bars} bars`] : []), ...(u.max_duration_s ? [`≤ ${u.max_duration_s} s`] : []), ...(u.kind ? [u.kind] : []), ...(u.plugin ? [u.plugin] : []),
    ...(u.device ? [u.device] : []), ...u.formats, ...u.text.map((x) => `“${x}”`),
  ]
  if (!chips.length) return null
  return <div className="mb-2 flex flex-wrap items-center gap-1.5 text-xs"><span className="text-muted">{t.smart.understood}:</span>{chips.map((c) => <Badge key={c} tone="violet">{c}</Badge>)}</div>
}

function FileRow({ f, onOpen }: { f: LibFile; onOpen: (f: LibFile) => void }) {
  const { t, lang } = useT()
  const extra = f.score != null ? `${f.score}%` : f.similarity != null ? `${f.similarity}%` : f.fit ? `${f.fit.stretch_percent > 0 ? '+' : ''}${f.fit.stretch_percent}% ${t.smart.stretch}` : null
  return (
    <li className="py-2">
      <div className="flex items-center gap-2">
        <button className="min-w-0 flex-1 truncate text-left text-sm font-medium hover:text-sky" title={f.path ?? f.rel_path} onClick={() => onOpen(f)}>{f.name}</button>
        {!f.device_online && <Badge tone="warn">{t.smart.offline}</Badge>}
        {f.status !== 'present' && <Badge tone={f.status === 'missing' ? 'err' : 'info'}>{t.smart[f.status]}</Badge>}
        {extra && <Badge tone="ok">{extra}</Badge>}
        <button className={btnIcon} disabled={!f.path} title={t.common.reveal} aria-label={t.common.reveal} onClick={() => f.path && revealItemInDir(f.path)}><FolderSearch size={14} /></button>
      </div>
      <div className="mt-0.5 flex flex-wrap items-center gap-x-3 gap-y-0.5">
        {f.categories.slice(0, 3).map((c) => <Badge key={c.tag}>{c.tag}</Badge>)}
        <Est label={t.smart.bpm} v={f.bpm} />
        <Est label={t.smart.key} v={f.key} />
        {f.duration_s != null && <span className="text-xs text-muted">{f.duration_s.toFixed(2)} s</span>}
        <span className="truncate text-[11px] text-muted">{f.device} · {f.rel_path}</span>
      </div>
      {!!f.why?.length && <div className="mt-0.5 text-[11px] text-muted">{t.smart.why}: {f.why.map((w) => whyText(w, lang)).join(' · ')}</div>}
    </li>
  )
}

function Detail({ id, onClose }: { id: number; onClose: () => void }) {
  const { t } = useT()
  const [f, setF] = useState<LibFile | null>(null)
  const [sim, setSim] = useState<LibFile[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [bpm, setBpm] = useState('')
  const [key, setKey] = useState('')
  const [cat, setCat] = useState('')
  useEffect(() => { api.libDetail(id).then(setF, (e) => setError(errText(e))); setSim(null) }, [id])
  const fix = async (field: 'bpm' | 'key' | 'add' | 'remove', value: string) => {
    try { setF(await api.libCorrect(id, field, value)); setError(null) } catch (e) { setError(errText(e)) }
  }
  if (!f) return <Card><ErrorLine error={error} /></Card>
  const m = (f.details?.measurements ?? {}) as Record<string, number | null>
  return (
    <Card title={t.smart.details} actions={<button className={btnIcon} aria-label={t.smart.close} onClick={onClose}><X size={15} /></button>}>
      <div className="font-display text-base font-semibold selectable">{f.name}</div>
      <div className="mt-0.5 text-xs text-muted selectable">{f.path ?? `${f.device} · ${f.rel_path} (${t.smart.offline})`}</div>
      <ErrorLine error={error ?? f.analysis_error ?? null} />
      <div className="mt-3 grid gap-x-6 gap-y-1 text-xs sm:grid-cols-2">
        <Est label={t.smart.bpm} v={f.bpm} />
        <Est label={t.smart.key} v={f.key} />
        <span><span className="text-muted">{t.smart.format} </span>{f.format ?? f.ext} · {fmtBytes(f.size)}</span>
        <span><span className="text-muted">{t.smart.duration} </span>{f.duration_s != null ? `${f.duration_s.toFixed(2)} s` : t.smart.unknown}{f.loop_bars ? ` · ${f.loop_bars} bars` : ''}</span>
        {f.sample_rate != null && <span className="text-muted">{f.sample_rate} Hz · {f.channels} ch{f.bit_depth ? ` · ${f.bit_depth} bit` : ''}</span>}
        {f.lufs != null && <span className="text-muted">{f.lufs} LUFS · peak {f.peak_dbfs} dBFS{m.true_peak_dbtp != null ? ` · TP ${m.true_peak_dbtp} dBTP` : ''}</span>}
        {f.moved_from && <span className="text-muted">{t.smart.moved}: {f.moved_from}</span>}
      </div>
      {(['categories', 'types', 'genres', 'descriptors', 'plugins'] as const).map((k) => f[k].length > 0 && (
        <div key={k} className="mt-2 flex flex-wrap items-center gap-1.5 text-xs">
          <span className="w-28 text-muted">{k === 'categories' ? t.smart.categories : k === 'descriptors' ? t.smart.descriptors : k === 'genres' ? t.smart.genres : k}</span>
          {f[k].map((l) => (
            <span key={l.tag} className="inline-flex items-center gap-1" title={`${t.smart.source[l.source] ?? l.source}${pct(l.confidence)}`}>
              <Badge tone={l.source === 'user' ? 'violet' : 'off'}>{l.tag} <span className="opacity-60">{Math.round(l.confidence * 100)}%</span></Badge>
              {k !== 'plugins' && <button className="text-muted hover:text-rose-300" aria-label={`${t.smart.removeTag} ${l.tag}`} onClick={() => fix('remove', `${k === 'categories' ? 'category' : k === 'types' ? 'type' : k === 'genres' ? 'genre' : 'descriptor'}:${l.tag}`)}><X size={11} /></button>}
            </span>
          ))}
        </div>
      ))}
      <div className="mt-3 grid gap-2 sm:grid-cols-3">
        <form className="flex gap-1" onSubmit={(e) => { e.preventDefault(); fix('bpm', bpm) }}><input className="field !py-1" placeholder={t.smart.setBpm} value={bpm} onChange={(e) => setBpm(e.target.value)} /><button className={btnGhost + ' !px-2 !py-1 !text-xs'}>{t.smart.apply}</button></form>
        <form className="flex gap-1" onSubmit={(e) => { e.preventDefault(); fix('key', key) }}><input className="field !py-1" placeholder={t.smart.setKey} value={key} onChange={(e) => setKey(e.target.value)} /><button className={btnGhost + ' !px-2 !py-1 !text-xs'}>{t.smart.apply}</button></form>
        <form className="flex gap-1" onSubmit={(e) => { e.preventDefault(); if (cat.trim()) fix('add', `category:${cat.trim().toLowerCase()}`) }}><input className="field !py-1" placeholder={t.smart.addCat} value={cat} onChange={(e) => setCat(e.target.value)} /><button className={btnGhost + ' !px-2 !py-1 !text-xs'}>{t.smart.apply}</button></form>
      </div>
      {f.kind === 'audio' && (
        <div className="mt-3">
          <button className={btnGhost + ' !py-1 !text-xs'} onClick={() => api.libSimilar(id).then((r) => setSim(r.results), (e) => setError(errText(e)))}><Sparkles size={13} />{t.smart.similar}</button>
          {sim && <div className="mt-2"><Note>{t.smart.similarNote}</Note><ul className="divide-y divide-line/50">{sim.slice(0, 8).map((s) => <FileRow key={s.id} f={s} onOpen={() => {}} />)}</ul></div>}
        </div>
      )}
    </Card>
  )
}

export function SmartSearch() {
  const { t } = useT()
  const [q, setQ] = useState('')
  const [kind, setKind] = useState('')
  const [res, setRes] = useState<LibQuery | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [open, setOpen] = useState<number | null>(null)
  useEffect(() => {
    if (!q.trim()) { setRes(null); return }
    const h = setTimeout(() => {
      setBusy(true)
      api.libQuery(q.trim(), kind).then((r) => { setRes(r); setError(null) }, (e) => setError(errText(e))).finally(() => setBusy(false))
    }, 300)
    return () => clearTimeout(h)
  }, [q, kind])
  return (
    <div className="space-y-4">
      <Card title={t.library.results}>
        <div className="mb-3 flex gap-2">
          <div className="relative flex-1">
            <Search size={15} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted" />
            <input className="field !pl-9" value={q} onChange={(e) => setQ(e.target.value)} placeholder={t.smart.search} aria-label={t.library.results} />
          </div>
          <select className="field !w-36" value={kind} onChange={(e) => setKind(e.target.value)} aria-label={t.smart.format}>
            <option value="">{t.common.all}</option>
            {(['audio', 'midi', 'preset', 'project'] as const).map((k) => <option key={k} value={k}>{t.library.fileKinds[k]}</option>)}
          </select>
        </div>
        <ErrorLine error={error} />
        {busy && <p className="text-xs text-muted">{t.smart.searching}</p>}
        {res && <Chips u={res.understood} />}
        {res && (res.results.length === 0 ? <Empty>{t.smart.noResults}</Empty> : (
          <ul className="max-h-[480px] divide-y divide-line/50 overflow-y-auto">{res.results.map((f) => <FileRow key={f.id} f={f} onOpen={(x) => setOpen(x.id)} />)}</ul>
        ))}
      </Card>
      {open != null && <Detail id={open} onClose={() => setOpen(null)} />}
    </div>
  )
}

export function LoopsCard() {
  const { t } = useT()
  const [bpm, setBpm] = useState('124')
  const [key, setKey] = useState('')
  const [cat, setCat] = useState('')
  const [res, setRes] = useState<LibFile[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  return (
    <Card title={t.smart.loops}>
      <form className="flex flex-wrap gap-2" onSubmit={(e) => { e.preventDefault(); api.libLoops(Number(bpm), key, cat).then((r) => { setRes(r.results); setError(null) }, (er) => setError(errText(er))) }}>
        <input className="field !w-28" type="number" min={40} max={250} value={bpm} onChange={(e) => setBpm(e.target.value)} aria-label={t.smart.projectBpm} placeholder={t.smart.projectBpm} />
        <input className="field !w-40" value={key} onChange={(e) => setKey(e.target.value)} placeholder={t.smart.projectKey} aria-label={t.smart.projectKey} />
        <input className="field !w-40" value={cat} onChange={(e) => setCat(e.target.value)} placeholder={t.smart.category} aria-label={t.smart.category} />
        <button className={btnPrimary}><Repeat size={14} />{t.smart.find}</button>
      </form>
      <ErrorLine error={error} />
      <div className="mt-2"><Note>{t.smart.loopsNote}</Note></div>
      {res && (res.length === 0 ? <Empty>{t.smart.noResults}</Empty> : <ul className="mt-2 max-h-72 divide-y divide-line/50 overflow-y-auto">{res.map((f) => <FileRow key={f.id} f={f} onOpen={() => {}} />)}</ul>)}
    </Card>
  )
}

export function DuplicatesCard() {
  const { t } = useT()
  const [similar, setSimilar] = useState(false)
  const [d, setD] = useState<Duplicates | null>(null)
  const [busy, setBusy] = useState(false)
  const group = (title: string, gs: { files: Duplicates['identical_files'][number]['files']; similarity?: number }[]) => gs.length > 0 && (
    <div className="mt-3">
      <div className="mb-1 font-mono text-[10px] uppercase tracking-widest text-sky">{title} · {gs.length}</div>
      <ul className="space-y-2">{gs.slice(0, 30).map((g, i) => (
        <li key={i} className="rounded-lg border border-line p-2 text-xs">
          {g.similarity != null && <Badge tone="info">{g.similarity * 100 | 0}%</Badge>}
          {g.files.map((f) => <div key={f.id} className="truncate selectable" title={f.path ?? ''}>{f.name} <span className="text-muted">· {f.device} · {f.rel_path}</span></div>)}
        </li>
      ))}</ul>
    </div>
  )
  return (
    <Card title={t.smart.duplicates} actions={
      <div className="flex items-center gap-2">
        <label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={similar} onChange={(e) => setSimilar(e.target.checked)} />{t.smart.includeSimilar}</label>
        <button className={btnGhost + ' !py-1 !text-xs'} disabled={busy} onClick={() => { setBusy(true); api.libDuplicates(similar).then(setD).finally(() => setBusy(false)) }}><Copy size={13} />{t.smart.findDuplicates}</button>
      </div>
    }>
      <Note>{t.smart.nothingDeleted}</Note>
      {d && (d.identical_files.length + d.same_audio.length + d.similar_not_duplicates.length === 0 ? <Empty>{t.smart.noDuplicates}</Empty> : <>
        {group(t.smart.identical, d.identical_files)}
        {group(t.smart.sameAudio, d.same_audio)}
        {group(t.smart.similarNotDup, d.similar_not_duplicates)}
      </>)}
    </Card>
  )
}

export function IndexStatus({ ov, paused, onPause, onScanAll, onStop }: { ov: LibOverview | null; paused: boolean; onPause: (p: boolean) => void; onScanAll: () => void; onStop: () => void }) {
  const { t, lang } = useT()
  const { settings, setSetting } = useStore()
  const every = (m: number) => (m < 60 ? `${m} min` : `${m / 60} ${lang === 'it' ? (m === 60 ? 'ora' : 'ore') : m === 60 ? 'hour' : 'hours'}`)
  if (!ov) return null
  const act = ov.service.activity.split(' ')[0]
  return (
    <Card title={t.smart.status} actions={
      <div className="flex gap-2">
        <button className={btnGhost + ' !py-1 !text-xs'} onClick={onScanAll}>{t.smart.scanAll}</button>
        <button className={btnGhost + ' !py-1 !text-xs'} onClick={onStop}>{t.smart.stop}</button>
        <button className={btnGhost + ' !py-1 !text-xs'} onClick={() => onPause(!paused)}>{paused ? <Play size={13} /> : <Pause size={13} />}{paused ? t.smart.resume : t.smart.pause}</button>
      </div>
    }>
      <div className="flex flex-wrap gap-x-6 gap-y-1 text-sm">
        <span>{ov.stats.files.toLocaleString()} {t.library.files}</span>
        <span className="text-emerald-300">{ov.analysis.done} {t.smart.analyzed}</span>
        <span className="text-sky">{ov.analysis.pending} {t.smart.pending}</span>
        {ov.analysis.errors > 0 && <span className="text-amber-200">{ov.analysis.errors} {t.smart.errors}</span>}
        <span className="text-muted">{t.smart.activity}: {t.smart.activities[act] ?? act}{ov.service.paused_reason ? ` · ${ov.service.paused_reason}` : ''}</span>
      </div>
      <div className="mt-2 flex flex-wrap gap-1.5">{Object.entries(ov.stats.by_category).sort((a, b) => b[1] - a[1]).slice(0, 14).map(([k, n]) => <Badge key={k}>{k === 'unknown' ? t.smart.unknown : k} {n}</Badge>)}</div>
      <div className="mt-3 flex flex-wrap items-center gap-x-5 gap-y-2 text-xs">
        <label className="flex items-center gap-2">
          <span className="text-muted">{t.smart.rescan}</span>
          <select className="field !w-auto !py-0.5 !text-xs" value={settings.library_rescan_minutes || '0'} onChange={(e) => setSetting('library_rescan_minutes', e.target.value)}>
            <option value="0">{t.smart.rescanOff}</option>
            {[15, 60, 360, 1440].map((m) => <option key={m} value={String(m)}>{fill(t.smart.rescanEvery, { n: every(m) })}</option>)}
          </select>
        </label>
        <label className="flex items-center gap-2">
          <input type="checkbox" checked={settings.library_pause_during_daw !== '0'} onChange={(e) => setSetting('library_pause_during_daw', e.target.checked ? '1' : '0')} />
          {t.smart.pauseDaw}
        </label>
      </div>
      <p className="mt-2 text-[11px] text-muted">{t.smart.formats}: {(ov.formats.audio_decoded ?? []).join(', ')} · MIDI · {(ov.formats.presets_read ?? []).join(', ')}</p>
    </Card>
  )
}
