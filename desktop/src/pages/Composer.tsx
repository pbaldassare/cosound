import { useEffect, useMemo, useRef, useState } from 'react'
import { FileMusic, FolderSearch, History, Play, Search, Sparkles, Square, Undo2, Wand2, Download } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { api, errText, type Clip, type Composed, type EditResult, type Part, type PartId, type Sketch, type TrackV } from '../lib/api'
import { play } from '../lib/preview'
import { useT } from '../i18n'
import { Badge, Card, ErrorLine, Note, PageHeader, btnGhost, btnPrimary } from '../components/ui'

const COLORS: Record<string, string> = { Melody: '#22d3ee', Chords: '#a855f7', Bass: '#3b82f6', Arp: '#34d399', Drums: '#f59e0b' }
const PART_ID: Record<string, PartId> = { Melody: 'melody', Chords: 'chords', Bass: 'bass', Arp: 'melody', Drums: 'drums' }
const NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
const noteName = (p: number) => `${NAMES[p % 12]}${Math.floor(p / 12) - 1}`
const small = btnGhost + ' !px-2 !py-1 !text-xs'

function Roll({ track, length, ppq, height = 90 }: { track: TrackV; length: number; ppq: number; height?: number }) {
  const W = 1000
  const ps = track.notes.map((n) => n.pitch)
  const lo = Math.min(...ps), hi = Math.max(...ps, lo + 11)
  const rows = hi - lo + 1
  const rh = height / rows
  const y = (p: number) => height - (p - lo + 1) * rh
  const beats = Array.from({ length: Math.floor(length / ppq) + 1 }, (_, i) => i)
  return (
    <div>
      <div className="mb-1 text-[11px] text-muted"><span className="font-medium text-white">{track.name}</span> · {track.notes.length} · {noteName(lo)}–{noteName(hi)}{track.cc.length > 0 && ` · CC${track.cc[0][1]}`}</div>
      <svg viewBox={`0 0 ${W} ${height}`} preserveAspectRatio="none" className="w-full rounded-md bg-ink/70" style={{ height }} role="img" aria-label={track.name}>
        {beats.map((b) => <line key={b} x1={(b * ppq / length) * W} x2={(b * ppq / length) * W} y1={0} y2={height} stroke={b % 4 === 0 ? '#3a3a4d' : '#1c1c26'} strokeWidth={1} vectorEffect="non-scaling-stroke" />)}
        {track.notes.map((n, i) => <rect key={i} x={(n.start / length) * W} y={y(n.pitch) + 0.5} width={Math.max((n.dur / length) * W, 1.5)} height={Math.max(rh - 1, 1)} rx={1} fill={COLORS[track.name] ?? '#94a3b8'} opacity={0.35 + (n.vel / 127) * 0.65} />)}
      </svg>
    </div>
  )
}

function Player({ tracks, bpm, length, ppq }: { tracks: TrackV[]; bpm: number; length: number; ppq: number }) {
  const { t } = useT()
  const [on, setOn] = useState(false)
  const p = useRef<ReturnType<typeof play> | null>(null)
  const stop = () => { p.current?.stop(); p.current = null; setOn(false) }
  useEffect(() => stop, [])
  useEffect(() => { stop() }, [tracks]) // eslint-disable-line react-hooks/exhaustive-deps
  const start = () => {
    stop()
    const parts: Part[] = tracks.map((tr) => ({ id: PART_ID[tr.name] ?? 'melody', name: tr.name, channel: tr.channel, program: tr.program ?? 0, notes: tr.notes }))
    const pl = play({ bpm, ppq, length_ticks: length } as unknown as Sketch, parts)
    p.current = pl
    setOn(true)
    const end = () => { if (p.current !== pl) return; if (pl.tickAt() >= length) stop(); else requestAnimationFrame(end) }
    requestAnimationFrame(end)
  }
  return on ? <button className={btnGhost} onClick={stop}><Square size={14} />{t.composer.stop}</button> : <button className={btnGhost} onClick={start}><Play size={14} />{t.composer.play}</button>
}

function Theory() {
  const { t, lang } = useT()
  const [kind, setKind] = useState('scale')
  const [a, setA] = useState('Fa diesis')
  const [b, setB] = useState('minore')
  const [res, setRes] = useState<Record<string, any> | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [scales, setScales] = useState<{ id: string; nome: string; name: string }[]>([])
  useEffect(() => { api.musicTheory({ what: 'scales' }).then((v) => setScales(v as never), () => {}) }, [])
  useEffect(() => { setRes(null); if (kind === 'scale') { setA('Fa diesis'); setB('minore') } else if (kind === 'chord') setA('Am9'); else if (kind === 'interval') { setA('C4'); setB('G4') } else setA('La minore') }, [kind])
  const go = () => {
    const input = kind === 'scale' ? { what: 'scale', tonic: a, scale: b } : kind === 'chord' ? { what: 'chord', chord: a } : kind === 'interval' ? { what: 'interval', from: a, to: b } : { what: 'key', key: a }
    api.musicTheory(input).then((r) => { setRes(r); setError(null) }, (e) => setError(errText(e)))
  }
  return (
    <Card title={t.composer.theory}>
      <form className="flex flex-wrap gap-2" onSubmit={(e) => { e.preventDefault(); go() }}>
        <select className="field !w-36" value={kind} onChange={(e) => setKind(e.target.value)} aria-label={t.composer.theory}>{Object.entries(t.composer.theoryKinds).map(([k, v]) => <option key={k} value={k}>{v}</option>)}</select>
        <input className="field !w-40" value={a} onChange={(e) => setA(e.target.value)} aria-label="a" />
        {kind === 'scale' && <select className="field !w-56" value={b} onChange={(e) => setB(e.target.value)} aria-label={t.composer.theoryKinds.scale}><option value="minore">{lang === 'it' ? 'Minore naturale' : 'Natural minor'}</option>{scales.map((s) => <option key={s.id} value={s.id}>{lang === 'it' ? s.nome : s.name}</option>)}</select>}
        {kind === 'interval' && <input className="field !w-28" value={b} onChange={(e) => setB(e.target.value)} aria-label="b" />}
        <button className={btnPrimary}><Search size={14} />{t.composer.lookup}</button>
      </form>
      <ErrorLine error={error} />
      {res && (
        <div className="mt-3 space-y-1 text-xs">
          {res.notes && <p className="font-mono text-sm selectable">{(lang === 'it' && res.note_italiane ? res.note_italiane : res.notes).join('  ')}</p>}
          {res.scale && kind === 'key' && <p className="font-mono text-sm">{res.nome} · {(res.scale as string[]).join(' ')}</p>}
          {res.short && <p className="text-sm">{res.short} · {lang === 'it' ? res.nome : res.name} · {res.semitones} {lang === 'it' ? 'semitoni' : 'semitones'} · {res.from} → {res.to}</p>}
          {res.midi_from_octave_4 && <p className="text-muted">MIDI: {(res.midi_from_octave_4 as number[]).join(' ')}</p>}
          {res.carattere && <p className="text-muted">{lang === 'it' ? res.carattere : res.character} · {res.contexts}</p>}
          {res.diatonic_chords && <p>{(res.diatonic_chords as { triad: string }[]).map((c) => c.triad).join(' · ')}</p>}
          {res.triads && <p>{(res.triads as { symbol: string; roman: string }[]).map((c) => `${c.symbol} (${c.roman})`).join(' · ')}</p>}
          {res.signature && <p className="text-muted">{(res.signature.accidentals as string[]).join(' ') || '—'} · {res.relative && `rel. ${res.relative}`}</p>}
          {res.voicings && <p className="text-muted">MIDI: {(res.voicings.close as number[]).join(' ')} · drop2 {(res.voicings.drop2 as number[]).join(' ')}</p>}
          {res.source && <p className="text-[10px] text-muted">{res.source}</p>}
        </div>
      )}
    </Card>
  )
}

function Analyzer() {
  const { t } = useT()
  const [res, setRes] = useState<Record<string, any> | null>(null)
  const [error, setError] = useState<string | null>(null)
  const pick = async () => {
    const f = await open({ multiple: false, directory: false, filters: [{ name: 'MIDI', extensions: ['mid', 'midi'] }] })
    if (typeof f === 'string') api.musicAnalyzeFile(f).then((r) => { setRes(r); setError(null) }, (e) => setError(errText(e)))
  }
  return (
    <Card title={t.composer.analyze}>
      <button className={btnGhost} onClick={pick}><FileMusic size={14} />{t.composer.chooseMidi}</button>
      <ErrorLine error={error} />
      {res && (
        <div className="mt-3 space-y-1 text-xs">
          <p><span className="text-muted">{t.composer.key}:</span> {res.key?.nome ?? res.key?.key ?? '—'} ({Math.round((res.key?.confidence ?? 0) * 100)}%) · <span className="text-muted">{t.composer.candidates}:</span> {(res.key?.candidates ?? []).slice(0, 3).map((c: { key: string }) => c.key).join(', ')}</p>
          {(res.key?.flags ?? []).map((f: string) => <p key={f} className="text-amber-200">{f}</p>)}
          <p><span className="text-muted">BPM:</span> {res.bpm ?? '—'} · {res.time_signature ?? ''} · {res.bars} {t.composer.bars} · {res.rhythm?.grid}</p>
          {res.chords?.length > 0 && <p><span className="text-muted">{t.composer.chords}:</span> {res.chords.map((c: { bar: number; symbol: string }) => `${c.bar}: ${c.symbol}`).join(' · ')}</p>}
          {res.progression && <p className="font-mono">{res.progression.chords.map((c: { roman: string }) => c.roman).join(' – ')}</p>}
          {(res.tracks ?? []).map((tr: Record<string, any>) => (
            <div key={`${tr.track}-${tr.channel}`} className="rounded border border-line p-2">
              <span className="font-medium">{tr.name || `Track ${tr.track}`}</span> <span className="text-muted">ch {tr.channel} · {tr.notes} · {(tr.range ?? []).join('–')}{tr.phrases ? ` · ${tr.phrases.length} ${t.composer.phrases}` : ''}</span>
              {tr.outside_key && <div className="text-muted">{t.composer.outside}: {tr.outside_key.length ? tr.outside_key.slice(0, 6).map((o: { note: string; reason: string }) => `${o.note} (${o.reason})`).join(' · ') : t.composer.none}</div>}
            </div>
          ))}
          <p className="text-[10px] text-muted">{res.note}</p>
        </div>
      )}
    </Card>
  )
}

export default function Composer() {
  const { t } = useT()
  const [text, setText] = useState('Crea una melodia emozionale e ipnotica in tonalità di La minore, adatta a una produzione Melodic Techno a 124 BPM.')
  const [seed, setSeed] = useState(1)
  const [c, setC] = useState<Composed | null>(null)
  const [clip, setClip] = useState<Clip | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [editText, setEditText] = useState('')
  const [track, setTrack] = useState('')
  const [last, setLast] = useState<EditResult | null>(null)
  const [cmp, setCmp] = useState<{ from: number; to: number; res: { track: string; diff: Record<string, number> | null }[] } | null>(null)
  const [exported, setExported] = useState<string | null>(null)
  const [clips, setClips] = useState<Awaited<ReturnType<typeof api.musicClips>>>([])
  const loadClips = () => api.musicClips().then(setClips, () => {})
  useEffect(() => { loadClips() }, [])
  const refresh = async (id: string) => setClip(await api.musicClip(id))
  const run = async (fn: () => Promise<unknown>) => { setBusy(true); setError(null); try { await fn() } catch (e) { setError(errText(e)) } finally { setBusy(false) } }
  const compose = () => run(async () => {
    const r = await api.musicCompose({ text, seed })
    setC(r); setLast(null); setCmp(null); setExported(null)
    await refresh(r.clip_id)
    loadClips()
  })
  const open_ = (id: string) => run(async () => { const k = await api.musicClip(id); setClip(k); setC(k.report); setLast(null); setCmp(null) })
  const tracks = clip?.tracks ?? c?.tracks ?? []
  const length = clip?.length_ticks ?? c?.length_ticks ?? 1920
  const ppq = clip?.ppq ?? c?.ppq ?? 480
  const v = c?.validation
  const groups = useMemo(() => {
    const g: Record<string, number> = {}
    v?.findings.forEach((f) => { g[f.kind] = (g[f.kind] ?? 0) + 1 })
    return g
  }, [v])

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.composer.title}>
        {clips.length > 0 && (
          <select className="field !w-72" value={clip?.clip_id ?? ''} onChange={(e) => e.target.value && open_(e.target.value)} aria-label={t.composer.clips}>
            <option value="">{t.composer.clips}</option>
            {clips.map((k) => <option key={k.clip_id} value={k.clip_id}>{k.title} · v{k.versions}</option>)}
          </select>
        )}
      </PageHeader>
      <Note>{t.composer.intro}</Note>
      <Card>
        <form className="space-y-2" onSubmit={(e) => { e.preventDefault(); compose() }}>
          <textarea className="field min-h-20" value={text} onChange={(e) => setText(e.target.value)} placeholder={t.composer.placeholder} aria-label={t.composer.compose} />
          <div className="flex flex-wrap items-center gap-2">
            <label className="flex items-center gap-2 text-xs text-muted">{t.composer.seed}<input className="field !w-20 !py-1" type="number" min={0} value={seed} onChange={(e) => setSeed(Number(e.target.value) || 0)} /></label>
            <button className={btnPrimary} disabled={busy || !text.trim()}><Sparkles size={14} />{busy ? t.composer.composing : t.composer.compose}</button>
          </div>
        </form>
        <ErrorLine error={error} />
      </Card>
      {c && (
        <>
          <Card title={c.title}>
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <Badge tone="violet">{t.composer.key}: {c.nome ?? c.key}</Badge>
              <Badge tone="info">{c.bpm} {t.composer.bpm}</Badge>
              <Badge>{c.bars} {t.composer.bars}</Badge>
              <Badge>{t.composer.style}: {c.style.label}</Badge>
              {c.moods.length > 0 && <Badge>{t.composer.moods}: {c.moods.join(', ')}</Badge>}
            </div>
            {c.request_understood && (
              <p className="mt-2 text-xs text-muted">{t.composer.understood}: {[c.request_understood.parts.join('+'), c.request_understood.key, c.request_understood.bpm && `${c.request_understood.bpm} BPM`, c.request_understood.styles.map((s) => s[0]).join('+'), c.request_understood.moods.join(', ')].filter(Boolean).join(' · ')}
                {c.request_understood.unknown.length > 0 && <span className="text-amber-200"> · {t.composer.notUnderstood}: {c.request_understood.unknown.join(', ')}</span>}</p>
            )}
            {c.memory_used && <p className="text-xs text-violet">{t.composer.memory}</p>}
            {c.choices.map((x) => <p key={x} className="text-xs text-muted">{x}</p>)}
            <div className="mt-3 flex flex-wrap gap-2">
              {c.progression.chords.map((ch, i) => (
                <div key={i} className="min-w-20 rounded-lg border border-line bg-white/[0.03] px-3 py-2" title={ch.notes.join(' ')}>
                  <div className="font-display text-base font-semibold">{ch.symbol}</div>
                  <div className="font-mono text-xs text-sky">{ch.roman}</div>
                  <div className="text-[11px] text-muted">{t.composer.functions[ch.function] ?? ch.function}</div>
                </div>
              ))}
            </div>
            <p className="mt-2 text-xs text-muted">{t.composer.loop}: {c.progression.loop_cadence}</p>
            {c.alternative_progressions.length > 0 && (
              <details className="mt-2 text-xs"><summary className="cursor-pointer text-sky">{t.composer.alternatives} ({c.alternative_progressions.length})</summary>
                <ul className="mt-1 space-y-0.5">{c.alternative_progressions.map((a) => <li key={a.roman}><span className="font-mono">{a.roman}</span> · {a.chords.join(' – ')} <span className="text-muted">({a.origin})</span></li>)}</ul>
              </details>
            )}
          </Card>
          <Card title={t.composer.tracks} actions={<div className="flex gap-2"><Player tracks={tracks} bpm={c.bpm} length={length} ppq={ppq} /><button className={btnGhost} disabled={!clip} onClick={() => clip && run(async () => setExported((await api.musicExportClip(clip.clip_id)).path))}><Download size={14} />{t.composer.export}</button></div>}>
            {exported && <p className="mb-2 flex items-center gap-2 text-xs text-emerald-300">{t.composer.exported} <span className="truncate selectable">{exported}</span><button className="text-muted hover:text-white" onClick={() => revealItemInDir(exported)}><FolderSearch size={13} /></button></p>}
            <div className="space-y-3">{tracks.map((tr) => <Roll key={tr.name} track={tr} length={length} ppq={ppq} />)}</div>
          </Card>
          {v && (
            <Card title={t.composer.validation}>
              <div className="flex flex-wrap gap-2 text-xs">
                <Badge tone={v.passed ? 'ok' : 'err'}>{v.passed ? t.composer.passed : `${v.technical_errors} ${t.composer.errors}`}</Badge>
                {Object.entries(groups).map(([k, n]) => <Badge key={k} tone={k === 'technical_error' ? 'err' : k === 'stylistic_choice' ? 'warn' : 'off'}>{t.composer.kinds[k]} {n}</Badge>)}
              </div>
              <ul className="mt-2 max-h-48 space-y-0.5 overflow-y-auto text-xs">{v.findings.map((f, i) => <li key={i} className={f.kind === 'technical_error' ? 'text-rose-300' : f.kind === 'stylistic_choice' ? 'text-amber-200' : 'text-muted'}>{f.part} · {f.at ?? ''} {f.detail}</li>)}</ul>
            </Card>
          )}
          {clip && (
            <Card title={t.composer.edit} actions={<span className="text-xs text-muted">{t.composer.version} {clip.version}</span>}>
              <form className="flex flex-wrap gap-2" onSubmit={(e) => { e.preventDefault(); if (editText.trim()) run(async () => { const r = await api.musicEdit(clip.clip_id, track, editText); setLast(r); await refresh(clip.clip_id) }) }}>
                <select className="field !w-40" value={track} onChange={(e) => setTrack(e.target.value)} aria-label={t.composer.tracks}><option value="">{t.composer.allTracks}</option>{clip.tracks.map((x) => <option key={x.name} value={x.name}>{x.name}</option>)}</select>
                <input className="field min-w-64 flex-1" value={editText} onChange={(e) => setEditText(e.target.value)} placeholder={t.composer.editPlaceholder} aria-label={t.composer.edit} />
                <button className={btnPrimary} disabled={busy}><Wand2 size={14} />{t.composer.apply}</button>
              </form>
              {last && (
                <div className="mt-2 text-xs">
                  {last.not_understood.length > 0 && <p className="text-amber-200">{t.composer.notUnderstood}: {last.not_understood.join(', ')}</p>}
                  {Object.entries(last.diff).map(([name, d]) => <p key={name}><span className="font-medium">{name}</span>: {d.notes_before} → {d.notes_after} {t.composer.notes} · {t.composer.pitch} {d.average_pitch_change > 0 ? '+' : ''}{d.average_pitch_change} · {t.composer.sync} {d.syncopation_before} → {d.syncopation_after}</p>)}
                </div>
              )}
              <div className="mt-3 flex flex-wrap items-center gap-2 text-xs">
                <span className="text-muted">{t.composer.variations}:</span>
                {Object.entries(t.composer.variationNames).map(([k, label]) => <button key={k} className={small} disabled={busy || !clip.tracks.some((x) => x.name === 'Melody')} onClick={() => run(async () => { const r = await api.musicVary(clip.clip_id, k, seed); setLast(null); await refresh(clip.clip_id); setError(null); setCmp(null); setExported(`${label}: ${t.composer.identity} ${Math.round(r.variation.identity_kept * 100)}%`) })}>{label}</button>)}
              </div>
              <div className="mt-3 flex flex-wrap items-center gap-2 text-xs">
                <History size={13} className="text-muted" /><span className="text-muted">{t.composer.history}:</span>
                {clip.history.map((h) => <Badge key={h.version}>v{h.version} {h.operation}{h.description ? `: ${h.description.slice(0, 40)}` : ''}</Badge>)}
                <button className={small} disabled={busy || clip.version <= 1} onClick={() => run(async () => { setClip(await api.musicUndo(clip.clip_id)); setLast(null) })}><Undo2 size={13} />{t.composer.undo}</button>
                {clip.version > 1 && <button className={small} onClick={() => run(async () => { const r = await api.musicCompare(clip.clip_id, 1, clip.version); setCmp({ from: 1, to: clip.version, res: r.tracks as never }) })}>{t.composer.compare} v1 {t.composer.with} v{clip.version}</button>}
              </div>
              {cmp && <ul className="mt-2 text-xs">{cmp.res.map((r) => r.diff && <li key={r.track}>{r.track}: {r.diff.notes_before} → {r.diff.notes_after} {t.composer.notes} · {t.composer.pitch} {r.diff.average_pitch_change}</li>)}</ul>}
            </Card>
          )}
        </>
      )}
      <div className="grid gap-5 lg:grid-cols-2">
        <Theory />
        <Analyzer />
      </div>
    </div>
  )
}

