import { useEffect, useMemo, useRef, useState } from 'react'
import { CircleCheck, Dices, Download, FolderSearch, Play, Square, TriangleAlert } from 'lucide-react'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { api, errText, type MusicOptions, type Part, type PartId, type Sketch } from '../lib/api'
import { play } from '../lib/preview'
import { fill, useT } from '../i18n'
import { Badge, Card, ErrorLine, Note, PageHeader, btnGhost, btnIcon, btnPrimary, useLoad } from '../components/ui'

const TONICS = ['C', 'C#', 'Db', 'D', 'Eb', 'E', 'F', 'F#', 'Gb', 'G', 'Ab', 'A', 'Bb', 'B']
const PART_IDS: PartId[] = ['melody', 'chords', 'bass', 'drums']
const COLORS: Record<PartId, string> = { melody: '#22d3ee', chords: '#a855f7', bass: '#3b82f6', drums: '#f59e0b' }

type Form = {
  genre: string; tonic: string; mode: string; bpm: number; bars: number; progression: string; seed: number; swing: number
  sevenths: boolean; fill: boolean; chord_rhythm: string; bass_style: string; drum_style: string
}

const splitKey = (k: string) => {
  const [tonic, m = 'major'] = k.split(' ')
  return { tonic, mode: m === 'minor' ? 'natural_minor' : m }
}

function fromGenre(o: MusicOptions, id: string, seed: number): Form {
  const g = o.genres.find((x) => x.id === id) ?? o.genres[0]
  return {
    genre: g.id, ...splitKey(g.key), bpm: g.bpm, bars: 8, progression: g.progression, seed, swing: g.swing,
    sevenths: false, fill: true, chord_rhythm: g.chord_rhythm, bass_style: g.bass_style, drum_style: g.drum_style,
  }
}

const NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
const noteName = (p: number) => `${NAMES[p % 12]}${Math.floor(p / 12) - 1}`

function Roll({ part, sketch, height }: { part: Part; sketch: Sketch; height: number }) {
  const { t } = useT()
  const W = 1000
  const len = sketch.length_ticks
  const drums = part.id === 'drums'
  const rows = useMemo(() => {
    if (drums) return [...new Set(part.notes.map((n) => n.pitch))].sort((a, b) => a - b)
    const ps = part.notes.map((n) => n.pitch)
    const lo = Math.min(...ps), hi = Math.max(...ps, lo + 11)
    return Array.from({ length: hi - lo + 1 }, (_, i) => lo + i)
  }, [part, drums])
  const rh = height / Math.max(rows.length, 1)
  const y = (p: number) => height - (rows.indexOf(p) + 1) * rh
  const beats = Array.from({ length: (len / sketch.ppq) + 1 }, (_, i) => i)
  return (
    <div>
    <div className="mb-1 pl-[88px] text-[11px] text-muted"><span className="font-medium text-white">{t.sketch.parts[part.id]}</span> · {fill(t.sketch.notes, { n: part.notes.length })}</div>
    <div className="flex gap-2">
      <div className="relative w-20 shrink-0 text-[10px] leading-none text-muted" style={{ height }}>
        {drums
          ? rows.map((p) => <div key={p} className="absolute right-0 truncate" style={{ top: y(p) + rh / 2 - 5 }}>{t.sketch.drums[p] ?? p}</div>)
          : <><div className="absolute right-0 top-0">{noteName(rows[rows.length - 1])}</div><div className="absolute bottom-0 right-0">{noteName(rows[0])}</div></>}
      </div>
      <svg viewBox={`0 0 ${W} ${height}`} preserveAspectRatio="none" className="w-full rounded-md bg-ink/70" style={{ height }} role="img" aria-label={t.sketch.parts[part.id]}>
        {!drums && rows.map((p) => p % 12 === 0 && <line key={p} x1={0} x2={W} y1={y(p) + rh} y2={y(p) + rh} stroke="#23232f" strokeWidth={1} vectorEffect="non-scaling-stroke" />)}
        {beats.map((b) => <line key={b} x1={(b * sketch.ppq / len) * W} x2={(b * sketch.ppq / len) * W} y1={0} y2={height} stroke={b % 4 === 0 ? '#3a3a4d' : '#1c1c26'} strokeWidth={1} vectorEffect="non-scaling-stroke" />)}
        {part.notes.map((n, i) => (
          <rect key={i} x={(n.start / len) * W} y={y(n.pitch) + 0.5} width={Math.max((n.dur / len) * W, 1.5)} height={Math.max(rh - 1, 1)}
            rx={1} fill={COLORS[part.id]} opacity={0.35 + (n.vel / 127) * 0.65} />
        ))}
      </svg>
    </div>
    </div>
  )
}

export default function SketchPage() {
  const { t } = useT()
  const opts = useLoad(() => api.musicOptions())
  const [form, setForm] = useState<Form | null>(null)
  const [sketch, setSketch] = useState<Sketch | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [enabled, setEnabled] = useState<Record<PartId, boolean>>({ melody: true, chords: true, bass: true, drums: true })
  const [playing, setPlaying] = useState(false)
  const [exported, setExported] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const player = useRef<ReturnType<typeof play> | null>(null)
  const head = useRef<HTMLDivElement>(null)

  useEffect(() => { if (opts.data && !form) setForm(fromGenre(opts.data, 'melodic_techno', 1)) }, [opts.data, form])

  const input = useMemo(() => form && ({
    genre: form.genre || undefined, tonic: form.tonic, mode: form.mode, bpm: form.bpm, bars: form.bars, progression: form.progression,
    seed: form.seed, swing: form.swing, sevenths: form.sevenths, fill: form.fill,
    chord_rhythm: form.chord_rhythm, bass_style: form.bass_style, drum_style: form.drum_style,
  }), [form])

  useEffect(() => {
    if (!input) return
    let live = true
    api.musicSketch(input).then((s) => { if (live) { setSketch(s); setError(null) } }, (e) => { if (live) setError(errText(e)) })
    return () => { live = false }
  }, [input])

  const stop = () => { player.current?.stop(); player.current = null; setPlaying(false) }
  useEffect(() => stop, [])
  useEffect(() => { stop(); setExported(null) }, [input])

  const start = () => {
    if (!sketch) return
    stop()
    const p = play(sketch, sketch.parts.filter((x) => enabled[x.id]))
    player.current = p
    setPlaying(true)
    const tick = () => {
      if (player.current !== p) return
      const pos = p.tickAt() / sketch.length_ticks
      if (pos >= 1) return stop()
      if (head.current) head.current.style.left = `${Math.max(pos, 0) * 100}%`
      requestAnimationFrame(tick)
    }
    requestAnimationFrame(tick)
  }

  const doExport = async () => {
    if (!input) return
    setBusy(true)
    try {
      const r = await api.musicExport({ ...input, parts: PART_IDS.filter((p) => enabled[p]) })
      setExported(r.path)
    } catch (e) { setError(errText(e)) } finally { setBusy(false) }
  }

  if (!opts.data || !form) return <ErrorLine error={opts.error} />
  const o = opts.data
  const custom = (patch: Partial<Form>) => setForm({ ...form, ...patch })
  const sel = (label: string, value: string | number, values: (string | number)[], onChange: (v: string) => void, names?: Record<string, string>) => (
    <label className="block text-xs text-muted">{label}
      <select className="field mt-1" value={value} onChange={(e) => onChange(e.target.value)}>
        {values.map((v) => <option key={v} value={v}>{names?.[v] ?? v}</option>)}
      </select>
    </label>
  )
  const keyName = (s: string | null) => {
    if (!s) return ''
    const [tonic, ...rest] = s.split(' ')
    const id = rest.join('_') === 'minor' ? 'natural_minor' : rest.join('_')
    return `${tonic} ${t.sketch.modes[id] ?? rest.join(' ')}`
  }
  const sig = sketch?.key.signature
  const sigText = !sig ? '' : sig.accidental === 'sharps' ? fill(t.sketch.sharps, { n: sig.count }) : sig.accidental === 'flats' ? fill(t.sketch.flats, { n: sig.count }) : t.sketch.noAccidentals
  const failed = sketch ? Object.entries(sketch.checks).flatMap(([p, c]) => Object.entries(c).filter(([, ok]) => !ok).map(([k]) => `${t.sketch.parts[p]}: ${t.sketch.checks[k] ?? k}`)) : []

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.sketch.title}>
        <button className={btnGhost} onClick={() => custom({ seed: (form.seed % 9999) + 1 + Math.floor(Math.random() * 50) })}><Dices size={15} />{t.sketch.newSeed}</button>
        {playing
          ? <button className={btnGhost} onClick={stop}><Square size={15} />{t.sketch.stop}</button>
          : <button className={btnGhost} disabled={!sketch} onClick={start}><Play size={15} />{t.sketch.play}</button>}
        <button className={btnPrimary} disabled={!sketch?.all_checks_passed || busy || !PART_IDS.some((p) => enabled[p])} onClick={doExport}><Download size={15} />{busy ? t.sketch.exporting : t.sketch.export}</button>
      </PageHeader>
      <Note>{t.sketch.intro}</Note>
      {exported && (
        <div role="status" className="flex items-center gap-2 rounded-lg border border-emerald-400/30 bg-emerald-400/10 px-3 py-2 text-xs text-emerald-200">
          <CircleCheck size={14} /><span className="min-w-0 flex-1 truncate selectable" title={exported}>{t.sketch.exported} {exported}</span>
          <button className={btnIcon} title={t.common.reveal} aria-label={t.common.reveal} onClick={() => revealItemInDir(exported)}><FolderSearch size={14} /></button>
        </div>
      )}
      <ErrorLine error={error} />

      <div className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <Card title={t.sketch.genre}>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
            {sel(t.sketch.genre, form.genre, ['', ...o.genres.map((g) => g.id)], (v) => v ? setForm(fromGenre(o, v, form.seed)) : custom({ genre: '' }), { '': t.sketch.custom, ...Object.fromEntries(o.genres.map((g) => [g.id, g.label])) })}
            {sel(t.sketch.tonic, form.tonic, TONICS, (v) => custom({ tonic: v }))}
            {sel(t.sketch.mode, form.mode, o.modes.map((m) => m.id), (v) => custom({ mode: v, progression: o.modes.find((m) => m.id === v)?.default_progression ?? form.progression }), t.sketch.modes)}
            <label className="block text-xs text-muted">{t.sketch.bpm}
              <input className="field mt-1" type="number" min={40} max={250} value={form.bpm} onChange={(e) => custom({ bpm: Math.min(250, Math.max(40, Number(e.target.value) || 124)) })} />
            </label>
            {sel(t.sketch.bars, form.bars, [4, 8, 16, 32], (v) => custom({ bars: Number(v) }))}
            <label className="block text-xs text-muted">{t.sketch.seed}
              <input className="field mt-1" type="number" min={0} value={form.seed} onChange={(e) => custom({ seed: Math.max(0, Math.floor(Number(e.target.value) || 0)) })} />
            </label>
            <label className="col-span-2 block text-xs text-muted sm:col-span-3">{t.sketch.progression}
              <input className="field mt-1 font-mono" value={form.progression} onChange={(e) => custom({ progression: e.target.value })} spellCheck={false} />
              <span className="mt-1 block text-[11px]">{t.sketch.progressionHelp}</span>
            </label>
            {sel(t.sketch.chordRhythm, form.chord_rhythm, o.chord_rhythms, (v) => custom({ chord_rhythm: v }), t.sketch.styles)}
            {sel(t.sketch.bassStyle, form.bass_style, o.bass_styles, (v) => custom({ bass_style: v }), t.sketch.styles)}
            {sel(t.sketch.drumStyle, form.drum_style, o.drum_styles, (v) => custom({ drum_style: v }), t.sketch.styles)}
            <label className="col-span-2 block text-xs text-muted sm:col-span-1">{t.sketch.swing}: {form.swing}%
              <input className="mt-2 w-full accent-violet-500" type="range" min={0} max={100} step={5} value={form.swing} onChange={(e) => custom({ swing: Number(e.target.value) })} />
            </label>
            <label className="flex items-center gap-2 text-xs"><input type="checkbox" checked={form.sevenths} onChange={(e) => custom({ sevenths: e.target.checked })} />{t.sketch.sevenths}</label>
            <label className="flex items-center gap-2 text-xs"><input type="checkbox" checked={form.fill} onChange={(e) => custom({ fill: e.target.checked })} />{t.sketch.fill}</label>
          </div>
        </Card>

        <div className="space-y-5">
          {sketch && (
            <Card title={t.sketch.keyCard}>
              <div className="font-display text-lg font-semibold">{sketch.key.tonic} {t.sketch.modes[sketch.key.mode]}</div>
              <dl className="mt-2 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
                <dt className="text-muted">{t.sketch.scale}</dt><dd className="font-mono selectable">{sketch.key.scale.join(' ')}</dd>
                <dt className="text-muted">{t.sketch.signature}</dt><dd>{sigText}</dd>
                {sketch.key.relative && <><dt className="text-muted">{t.sketch.relative}</dt><dd>{keyName(sketch.key.relative)}</dd></>}
                {sketch.key.parallel && <><dt className="text-muted">{t.sketch.parallel}</dt><dd>{keyName(sketch.key.parallel)}</dd></>}
              </dl>
            </Card>
          )}
          {sketch && (
            <Card title={t.sketch.chordsCard}>
              <div className="flex flex-wrap gap-2">
                {sketch.progression.chords.map((c, i) => (
                  <div key={i} className="min-w-20 rounded-lg border border-line bg-white/[0.03] px-3 py-2" title={c.notes.join(' ')}>
                    <div className="font-display text-base font-semibold">{c.symbol}</div>
                    <div className="font-mono text-xs text-sky">{c.roman}</div>
                    <div className="mt-0.5 text-[11px] text-muted">{t.sketch.functions[c.function]} · {c.notes.join(' ')}</div>
                  </div>
                ))}
              </div>
              <p className="mt-3 text-xs text-muted">{t.sketch.loopCadence}: <span className="text-white">{t.sketch.cadences[sketch.progression.loop_cadence]}</span></p>
            </Card>
          )}
          {sketch && (
            <Card title={t.sketch.checksCard}>
              {sketch.all_checks_passed
                ? <p className="flex items-center gap-2 text-sm text-emerald-300"><CircleCheck size={15} />{t.sketch.checksOk}</p>
                : <div className="space-y-1 text-sm text-amber-200"><p className="flex items-center gap-2"><TriangleAlert size={15} />{t.sketch.checksFail}</p>{failed.map((f) => <p key={f} className="text-xs">{f}</p>)}</div>}
            </Card>
          )}
        </div>
      </div>

      {sketch && (
        <Card title={t.sketch.pianoRoll} actions={
          <div className="flex flex-wrap gap-1.5">
            {PART_IDS.map((p) => (
              <button key={p} onClick={() => setEnabled({ ...enabled, [p]: !enabled[p] })} aria-pressed={enabled[p]}
                className={`rounded-full border px-2.5 py-0.5 text-[11px] ${enabled[p] ? 'border-line bg-white/10 text-white' : 'border-line text-muted line-through'}`}>
                <span className="mr-1 inline-block h-2 w-2 rounded-full" style={{ background: COLORS[p] }} />{t.sketch.parts[p]}
              </button>
            ))}
          </div>
        }>
          <div className="mb-2 flex gap-2 pl-[88px] text-[10px] text-muted">
            {Array.from({ length: sketch.bars }, (_, b) => (
              <div key={b} className="min-w-0 flex-1 truncate">{b + 1} · <span className="text-white">{sketch.progression.chords[b % sketch.progression.chords.length].symbol}</span></div>
            ))}
          </div>
          <div className="relative space-y-3">
            {PART_IDS.map((id) => sketch.parts.find((p) => p.id === id)).filter((p): p is Part => !!p).map((p) => (
              <div key={p.id} className={enabled[p.id] ? '' : 'opacity-30'}>
                <Roll part={p} sketch={sketch} height={p.id === 'drums' ? 110 : p.id === 'bass' ? 70 : 120} />
              </div>
            ))}
            <div className="pointer-events-none absolute inset-y-0 left-[88px] right-0">
              <div ref={head} className={`absolute inset-y-0 w-px bg-white ${playing ? '' : 'hidden'}`} style={{ left: 0 }} />
            </div>
          </div>
          <div className="mt-3 flex flex-wrap items-center gap-2">
            <Badge tone="info">{sketch.bpm} BPM</Badge><Badge>{sketch.bars} {t.sketch.bars.toLowerCase()}</Badge><Badge>4/4</Badge>
            <span className="text-[11px] text-muted">{t.sketch.previewNote}</span>
          </div>
        </Card>
      )}
    </div>
  )
}
