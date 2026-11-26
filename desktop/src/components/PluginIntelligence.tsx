import { useEffect, useState } from 'react'
import { AudioLines, Check, Cpu, FileAudio, FolderSearch, Link2, Save, Search, SlidersHorizontal, Sparkles, Undo2, X } from 'lucide-react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { api, errText, type Audition, type ChainProposal, type DawCaps, type Design, type PluginDetail, type PluginOverview, type PluginPreset, type PluginRow } from '../lib/api'
import { fill, useT } from '../i18n'
import { Badge, Card, Empty, ErrorLine, Note, btnGhost, btnIcon, btnPrimary } from './ui'

const num = (v: number | null | undefined, d = 1) => (v == null ? '—' : Math.abs(v) >= 1000 ? Math.round(v).toString() : v.toFixed(d))
const small = btnGhost + ' !px-2 !py-1 !text-xs'
// The engine explains itself in English; fixed phrases are translated for the Italian UI.
const Q_IT: Record<string, string> = { aggressive: 'aggressivo', warm: 'caldo', deep: 'profondo', bright: 'brillante', dark: 'scuro', soft: 'morbido', punchy: 'incisivo', wide: 'ampio', atmospheric: 'atmosferico', movement: 'movimento', clean: 'pulito', long: 'lungo', short: 'corto' }
const TR_IT: [RegExp, string | ((...m: string[]) => string)][] = [
  [/^(\w+): no '(\w+)' parameter is exposed by these plug-ins$/, (_, d, r) => `${Q_IT[d] ?? d}: questi plug-in non hanno un parametro '${r}'`],
  [/^(\w+): nothing to change on the parameters available$/, (_, d) => `${Q_IT[d] ?? d}: nulla da cambiare sui parametri disponibili`],
  [/^no sound quality recognized in the request.*$/, 'nessuna qualità del suono riconosciuta nella richiesta (es. aggressivo, caldo, profondo, brillante, scuro, morbido, ampio, atmosferico, movimento)'],
  [/^less (\w+)/, (_, d) => `meno ${Q_IT[d] ?? d}`],
  [/^(\w+) \(limited to keep it (.+)\)$/, (_, d, k) => `${Q_IT[d] ?? d} (limitato per mantenerlo ${k.split(', ').map((x) => Q_IT[x] ?? x).join(', ')})`],
  [/^spectral centroid \(Hz\).*$/, 'centroide spettrale (Hz): più armoniche = più alto'],
  [/^sound-character estimate 0–1$/, 'stima del carattere 0–1'],
  [/^synthesizer$/, 'sintetizzatore'],
  [/^General MIDI sound set.*$/, 'suoni General MIDI: il suono si sceglie con il program change'],
  [/^Apple built-in General MIDI \/ sampler instrument.*$/, 'strumento Apple incluso (suoni di base)'],
  [/^your usual instrument for this role.*$/, 'il tuo strumento abituale per questo ruolo (memoria)'],
  [/^COSOUND cannot load this format yet.*$/, 'COSOUND non può ancora caricare questo formato: solo suggerimenti'],
  [/^(\w+) instrument$/, (_, r) => `strumento per ${r}`],
  [/^(filter|distortion|reverb|delay): no installed effect that COSOUND can load$/, (_, f) => `${f}: nessun effetto installato che COSOUND possa caricare`],
  [/^cutoff of (.+) opens from (\d+) to (\d+) over the phrase$/, (_, u, a, b) => `il filtro di ${u} si apre da ${a} a ${b} Hz durante la frase`],
  [/^skipped: crest factor (.+) dB, the material is already dense.*$/, (_, x) => `saltato: fattore di cresta ${x} dB, il materiale è già denso`],
  [/^halved: the material is already bright \((.+)\)$/, (_, x) => `dimezzato: il materiale è già brillante (${x})`],
  [/^little low end in the material \((.+) of energy\).*$/, (_, x) => `poche basse nel materiale (${x} dell'energia): il passa-alto è una sicurezza`],
  [/^the material is already wide.*$/, 'il materiale è già ampio (correlazione < 0,5): verifica la compatibilità mono prima di allargare'],
  [/^no installed plug-in in category (.+)$/, (_, c) => `nessun plug-in installato nella categoria ${c}`],
  [/^input peaks at (.+) dBFS: lower the input of the (.+) stage.*$/, (_, x, st) => `l'ingresso arriva a ${x} dBFS: abbassa l'ingresso dello stadio ${st}, gli stadi di saturazione reagiscono al livello`],
  [/^output reaches (.+): overload risk, lower the chain's output by at least (.+) dB$/, (_, x, y) => `l'uscita arriva a ${x}: rischio di sovraccarico, abbassa l'uscita della catena di almeno ${y} dB`],
  [/^the chain removes (.+) dB RMS.*$/, (_, x) => `la catena toglie ${x} dB RMS: uno stadio taglia il segnale utile (controlla filtri e frequenze)`],
  [/^the chain adds (.+) dB RMS.*$/, (_, x) => `la catena aggiunge ${x} dB RMS: confronta a parità di volume e controlla il margine`],
  [/^the limiter should be the last processor.*$/, 'il limiter deve essere l\'ultimo processore'],
  [/^(\w+) after (\w+) distorts the reverb\/echo tails too.*$/, (_, a, b) => `${a} dopo ${b} distorce anche le code: di solito la saturazione va prima`],
  [/^compressing after the reverb pumps the tail.*$/, 'comprimere dopo il riverbero fa pompare la coda: di solito la dinamica va prima degli effetti di tempo'],
  [/^level after the change reaches (.+): risk of clipping.*$/, (_, x) => `dopo la modifica il livello arriva a ${x}: rischio di clipping, abbassa l'uscita`],
]
const trIt = (s: string, lang: string) => {
  if (lang !== 'it') return s
  if (Q_IT[s]) return Q_IT[s]
  for (const [re, to] of TR_IT) if (re.test(s)) return s.replace(re, to as never)
  return s
}
const LEVELS = ['recognized', 'hosting', 'analyzed', 'parameters_available', 'parameters_controllable', 'parameters_with_sound_role', 'presets_found', 'presets_readable', 'presets_modifiable', 'control_verified', 'control_in_daw']

function useRun() {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const run = async <T,>(fn: () => Promise<T>): Promise<T | undefined> => {
    setBusy(true)
    setError(null)
    try { return await fn() } catch (e) { setError(errText(e)) } finally { setBusy(false) }
  }
  return { busy, error, run, setError }
}

export function Catalog({ ov, version }: { ov: PluginOverview | null; version: number }) {
  const { t } = useT()
  const [text, setText] = useState('')
  const [kind, setKind] = useState('')
  const [cat, setCat] = useState('')
  const [rows, setRows] = useState<PluginRow[]>([])
  const [open_, setOpen] = useState<string | null>(null)
  const [sel, setSel] = useState<string[]>([])
  const [cmp, setCmp] = useState<Awaited<ReturnType<typeof api.pluginsCompare>> | null>(null)
  useEffect(() => { const h = setTimeout(() => api.pluginsList(text, kind, cat).then(setRows, () => {}), 200); return () => clearTimeout(h) }, [text, kind, cat, version])
  const cats = ov ? [...ov.taxonomy.instrument, ...ov.taxonomy.effect].filter((c) => (ov.by_category[c] ?? 0) > 0) : []
  return (
    <div className="space-y-4">
      <Card title={fill(t.plugins.found, { n: rows.length })} actions={sel.length > 1 && <button className={small} onClick={() => api.pluginsCompare(sel).then(setCmp)}>{fill(t.plugins.compareSel, { n: sel.length })}</button>}>
        <div className="mb-3 flex flex-wrap gap-2">
          <div className="relative min-w-60 flex-1">
            <Search size={15} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted" />
            <input className="field !pl-9" value={text} onChange={(e) => setText(e.target.value)} placeholder={t.plugins.search} aria-label={t.plugins.search} />
          </div>
          <select className="field !w-40" value={kind} onChange={(e) => setKind(e.target.value)} aria-label={t.plugins.kind}>
            <option value="">{t.plugins.all}</option>
            {['instrument', 'effect', 'midi_effect', 'generator'].map((k) => <option key={k} value={k}>{t.plugins.kinds[k]}</option>)}
          </select>
          <select className="field !w-44" value={cat} onChange={(e) => setCat(e.target.value)} aria-label={t.plugins.category}>
            <option value="">{t.plugins.all}</option>
            {cats.map((c) => <option key={c} value={c}>{t.plugins.cats[c] ?? c} ({ov?.by_category[c]})</option>)}
          </select>
        </div>
        {rows.length === 0 ? <Empty>{t.plugins.empty}</Empty> : (
          <ul className="max-h-[420px] divide-y divide-line/50 overflow-y-auto">
            {rows.map((p) => (
              <li key={p.id} className="flex items-center gap-2 py-1.5">
                <input type="checkbox" aria-label={t.plugins.compare} checked={sel.includes(p.id)} onChange={(e) => setSel(e.target.checked ? [...sel, p.id].slice(-4) : sel.filter((x) => x !== p.id))} />
                <button className="min-w-0 flex-1 truncate text-left text-sm font-medium hover:text-sky" onClick={() => setOpen(p.id)}>{p.name}<span className="ml-2 text-xs font-normal text-muted">{p.vendor}</span></button>
                {!p.available && <Badge tone="warn">{t.plugins.statuses.missing}</Badge>}
                <Badge tone={p.kind === 'instrument' ? 'violet' : 'off'}>{t.plugins.kinds[p.kind] ?? p.kind}</Badge>
                <span className="hidden gap-1 md:flex">{p.categories.slice(0, 3).map((c) => <Badge key={c}>{t.plugins.cats[c] ?? c}</Badge>)}</span>
                {p.formats.map((f) => <Badge key={f} tone="info">{f}</Badge>)}
                <span className="w-40 whitespace-nowrap text-right text-[11px] text-muted">{p.parameters > 0 && `${p.parameters} ${t.plugins.params}`}{p.presets > 0 && ` · ${p.presets} ${t.plugins.presets}`}</span>
              </li>
            ))}
          </ul>
        )}
      </Card>
      {cmp && (
        <Card title={t.plugins.compare} actions={<button className={btnIcon} aria-label="close" onClick={() => setCmp(null)}><X size={15} /></button>}>
          <div className="grid gap-3" style={{ gridTemplateColumns: `repeat(${cmp.compared.length}, minmax(0, 1fr))` }}>
            {cmp.compared.map((c) => (
              <div key={c.product.id} className="rounded-lg border border-line p-3 text-xs">
                <div className="font-display text-sm font-semibold">{c.product.name}</div>
                <div className="text-muted">{c.product.vendor}</div>
                <div className="mt-2 flex flex-wrap gap-1">{c.categories.map((x) => <Badge key={x.category}>{t.plugins.cats[x.category] ?? x.category}</Badge>)}</div>
                <div className="mt-2">{c.formats.join(', ')} · {c.versions.filter(Boolean).join(', ')}</div>
                <div className="mt-1 text-muted">{t.plugins.role}: {c.sound_roles.join(', ') || '—'}</div>
                <div className="text-muted">{c.presets} {t.plugins.presets}</div>
              </div>
            ))}
          </div>
        </Card>
      )}
      {open_ && <Detail id={open_} onClose={() => setOpen(null)} />}
    </div>
  )
}

function Detail({ id, onClose }: { id: string; onClose: () => void }) {
  const { t } = useT()
  const [d, setD] = useState<PluginDetail | null>(null)
  const r = useRun()
  const load = () => api.pluginsDetail(id).then(setD, (e) => r.setError(errText(e)))
  useEffect(() => { setD(null); load() }, [id]) // eslint-disable-line react-hooks/exhaustive-deps
  if (!d) return <Card><ErrorLine error={r.error} /></Card>
  const k = d.knowledge
  return (
    <Card title={d.product.name} actions={<button className={btnIcon} aria-label="close" onClick={onClose}><X size={15} /></button>}>
      <div className="flex flex-wrap items-center gap-2 text-xs">
        <span className="text-muted">{d.product.vendor}</span>
        <Badge tone={d.product.kind === 'instrument' ? 'violet' : 'off'}>{t.plugins.kinds[d.product.kind]}</Badge>
        {d.categories.map((c) => <span key={c.category} title={`${c.source} · ${Math.round(c.confidence * 100)}%`}><Badge>{t.plugins.cats[c.category] ?? c.category}</Badge></span>)}
      </div>
      <ErrorLine error={r.error} />
      <h4 className="mt-4 mb-1 font-mono text-[10px] uppercase tracking-widest text-sky">{t.plugins.installs}</h4>
      {d.installs.map((i) => (
        <div key={i.id} className="mb-3 rounded-lg border border-line p-3 text-xs">
          <div className="flex flex-wrap items-center gap-2">
            <Badge tone="info">{i.format}</Badge>
            <span>{t.plugins.version} {i.version ?? '—'}</span>
            {i.archs.length > 0 && <span className="text-muted">{i.archs.join(' + ')}</span>}
            <span className="text-muted">{t.plugins.scopes[i.scope] ?? i.scope}</span>
            <Badge tone={i.status === 'available' ? 'ok' : i.status === 'missing' ? 'warn' : 'err'}>{t.plugins.statuses[i.status] ?? i.status}</Badge>
            <span className="flex-1" />
            {i.path && <button className={btnIcon} title={i.path} aria-label={t.plugins.path} onClick={() => revealItemInDir(i.path!)}><FolderSearch size={14} /></button>}
            {i.au_code && i.status === 'available' && <button className={small} disabled={r.busy} onClick={() => r.run(async () => { await api.pluginsDiscover(i.id); await load() })}><SlidersHorizontal size={13} />{r.busy ? t.plugins.reading : t.plugins.readParams}</button>}
          </div>
          {i.status_reason && <p className="mt-1 text-amber-200">{i.status_reason}</p>}
          {!!i.details.daw_compat?.length && <p className="mt-1 text-muted">{t.plugins.daws}: {i.details.daw_compat.map((x) => x.name).join(', ')}</p>}
          <div className="mt-2 grid gap-x-4 gap-y-0.5 sm:grid-cols-2">
            {LEVELS.map((lk) => [lk, i.levels[lk]] as const).map(([lk, v]) => (
              <div key={lk} className="flex justify-between gap-2"><span className="text-muted">{t.plugins.levelNames[lk] ?? lk}</span><span className="text-right">{v === true ? t.plugins.yes : v === false ? t.plugins.no : v == null ? '—' : typeof v === 'object' ? `${t.plugins.levelValues[(v as { where: string }).where]} · ${(v as { at: string }).at}` : t.plugins.levelValues[String(v)] ?? String(v)}</span></div>
            ))}
          </div>
          {i.parameters.length > 0 && (
            <details className="mt-2">
              <summary className="cursor-pointer text-sky">{t.plugins.parameters} ({i.parameters.length})</summary>
              <table className="mt-1 w-full text-left">
                <thead className="text-[10px] uppercase text-muted"><tr><th>{t.plugins.parameters}</th><th>{t.plugins.role}</th><th>{t.plugins.unit}</th><th>{t.plugins.range}</th><th>{t.plugins.def}</th></tr></thead>
                <tbody>{i.parameters.map((p) => (
                  <tr key={p.address} className="border-t border-line/40"><td className="py-0.5">{p.name}{!p.writable && <span className="text-muted"> ({t.plugins.readOnly})</span>}</td><td className={p.role ? 'text-violet' : 'text-muted'}>{p.role ?? '—'}</td><td className="text-muted">{p.unit}</td><td className="text-muted">{num(p.min)}…{num(p.max)}</td><td>{num(p.default, 2)}</td></tr>
                ))}</tbody>
              </table>
            </details>
          )}
        </div>
      ))}
      <h4 className="mt-4 mb-1 font-mono text-[10px] uppercase tracking-widest text-sky">{t.plugins.presetList} ({d.presets.length})</h4>
      <PresetList presets={d.presets} onChanged={load} />
      {d.library_presets.length > 0 && <p className="mt-1 text-xs text-muted">{t.plugins.libraryPresets}: {d.library_presets.map((p) => p.name).join(', ')}</p>}
      <h4 className="mt-4 mb-1 font-mono text-[10px] uppercase tracking-widest text-sky">{t.plugins.knowledge}</h4>
      <div className="space-y-1 text-xs">
        {k.documentation ? <p>{t.plugins.docSource}: {Object.values(k.documentation.source).filter((x) => typeof x === 'string').join(' · ')} — {k.documentation.parameters?.length ?? 0} {t.plugins.params}</p> : <p className="text-muted">{t.plugins.noDoc}</p>}
        {k.synth_profile && <p>{k.synth_profile.name}: {k.synth_profile.synthesis} <span className="text-muted">({k.synth_profile.source})</span></p>}
        {k.effect_facts.map((f) => <p key={f.category}><span className="text-violet">{t.plugins.cats[f.category] ?? f.category}</span>: {f.what_it_does} <span className="text-muted">({f.source})</span></p>)}
        {k.limits.map((l) => <p key={l} className="text-amber-200">{l}</p>)}
      </div>
      {d.errors.length > 0 && <><h4 className="mt-4 mb-1 font-mono text-[10px] uppercase tracking-widest text-rose-300">{t.plugins.errors}</h4>{d.errors.map((e, n) => <p key={n} className="text-xs text-rose-300">{e.at} · {e.operation}: {e.error}</p>)}</>}
    </Card>
  )
}

function PresetList({ presets, onChanged }: { presets: PluginPreset[]; onChanged?: () => void }) {
  const { t } = useT()
  const r = useRun()
  if (presets.length === 0) return <p className="text-xs text-muted">—</p>
  return (
    <>
      <ErrorLine error={r.error} />
      <ul className="max-h-60 divide-y divide-line/40 overflow-y-auto text-xs">
        {presets.map((p) => (
          <li key={`${p.origin ?? ''}${p.id}`} className="flex items-center gap-2 py-1">
            <span className="min-w-0 flex-1 truncate">{p.name} <span className="text-muted">· {p.plugin} · {t.plugins.sources[p.origin === 'library' ? 'library' : p.source] ?? p.source}{p.category ? ` · ${p.category}` : ''}</span></span>
            {p.why && p.why.length > 0 && <span className="hidden truncate text-[11px] text-muted md:inline">{p.why.join(' · ')}</span>}
            {p.details?.timbre && <span className="text-[11px] text-violet">{Object.entries(p.details.timbre.descriptors).filter(([, v]) => v >= 0.5).map(([k]) => k).slice(0, 3).join(', ')}</span>}
            {p.origin !== 'library' && (p.source === 'factory' || p.format === 'aupreset') && !p.details?.timbre && <button className={small} disabled={r.busy} onClick={() => r.run(async () => { await api.pluginsPresetAnalyze(p.id); onChanged?.() })}>{t.plugins.analyze}</button>}
          </li>
        ))}
      </ul>
    </>
  )
}

export function PresetSearch() {
  const { t } = useT()
  const [q, setQ] = useState('')
  const [res, setRes] = useState<PluginPreset[] | null>(null)
  const go = () => q.trim() && api.pluginsPresetSearch(q).then((r) => setRes(r.results))
  return (
    <Card title={t.presetSearch.title}>
      <form className="mb-2 flex gap-2" onSubmit={(e) => { e.preventDefault(); go() }}>
        <input className="field" value={q} onChange={(e) => setQ(e.target.value)} placeholder={t.presetSearch.placeholder} aria-label={t.presetSearch.title} />
        <button className={btnPrimary}><Search size={14} /></button>
      </form>
      {res && (res.length === 0 ? <Empty>{t.presetSearch.none}</Empty> : <PresetList presets={res} onChanged={go} />)}
    </Card>
  )
}

export function DesignStudio({ ov }: { ov: PluginOverview | null }) {
  const { t, lang } = useT()
  const tr = (x: string) => trIt(x, lang)
  const [role, setRole] = useState('bass')
  const [genre, setGenre] = useState('melodic_techno')
  const [key, setKey] = useState('A minor')
  const [bpm, setBpm] = useState('124')
  const [text, setText] = useState('')
  const [instrument, setInstrument] = useState('')
  const [instruments, setInstruments] = useState<PluginRow[]>([])
  const [d, setD] = useState<Design | null>(null)
  const [aud, setAud] = useState<Audition | null>(null)
  const [mod, setMod] = useState('')
  const [name, setName] = useState('')
  const [msg, setMsg] = useState<string | null>(null)
  const [daw, setDaw] = useState('logic')
  const [plan, setPlan] = useState<Awaited<ReturnType<typeof api.designPlan>> | null>(null)
  const [history, setHistory] = useState<Design[]>([])
  const r = useRun()
  useEffect(() => { api.pluginsList('', 'instrument', '').then(setInstruments, () => {}); api.designs().then(setHistory, () => {}) }, [ov])
  const reset = (x: Design) => { setD(x); setAud(null); setPlan(null); setMsg(null) }
  const refresh = async (id: string) => { const x = await api.designGet(id); setD((old) => ({ ...old, ...x })) }
  if (ov && !ov.host_available) return <Card title={t.design.title}><Note>{t.plugins.host}: {t.plugins.hostNo}</Note></Card>
  return (
    <Card title={t.design.title}>
      <Note>{t.design.note}</Note>
      <form className="mt-3 grid gap-2 sm:grid-cols-6" onSubmit={(e) => { e.preventDefault(); r.run(async () => reset(await api.designCreate(text, role, genre, key, bpm ? Number(bpm) : null, instrument))) }}>
        <select className="field" value={role} onChange={(e) => setRole(e.target.value)} aria-label={t.design.role}>{Object.entries(t.design.roles).map(([k, v]) => <option key={k} value={k}>{v}</option>)}</select>
        <select className="field" value={genre} onChange={(e) => setGenre(e.target.value)} aria-label={t.design.genre}>{['melodic_techno', 'techno', 'progressive_house', 'house', 'deep_house', 'tech_house', 'afro_house', 'trance', 'minimal', 'edm'].map((g) => <option key={g} value={g}>{g.replace(/_/g, ' ')}</option>)}</select>
        <input className="field" value={key} onChange={(e) => setKey(e.target.value)} placeholder={t.design.key} aria-label={t.design.key} />
        <input className="field" type="number" min={40} max={250} value={bpm} onChange={(e) => setBpm(e.target.value)} placeholder={t.design.bpm} aria-label={t.design.bpm} />
        <select className="field sm:col-span-2" value={instrument} onChange={(e) => setInstrument(e.target.value)} aria-label={t.design.instrument}>
          <option value="">{t.design.auto}</option>
          {instruments.map((p) => <option key={p.id} value={p.id}>{p.name}{p.vendor ? ` (${p.vendor})` : ''}</option>)}
        </select>
        <input className="field sm:col-span-5" value={text} onChange={(e) => setText(e.target.value)} placeholder={t.design.request} aria-label={t.design.request} />
        <button className={btnPrimary} disabled={r.busy}><Sparkles size={14} />{t.design.create}</button>
      </form>
      <ErrorLine error={r.error} />
      {d && (
        <div className="mt-4 space-y-3 text-xs">
          <div className="flex flex-wrap items-center gap-2">
            <span className="font-display text-sm font-semibold">{d.title}</span>
            {d.instrument_choice && <span className="text-muted">{d.instrument_choice.why.map(tr).join(' · ')}</span>}
          </div>
          {d.music && <p className="text-muted">{t.design.music}: {d.music.part} · {d.music.key} · {d.music.bpm} BPM</p>}
          {!!d.effects_added?.length && <p>{t.design.effects}: {d.effects_added.map(tr).join(' · ')}</p>}
          {d.automation && <p>{t.design.automation}: {tr(d.automation)}</p>}
          <Changes d={d} />
          {!!d.not_available?.length && <div className="text-amber-200">{t.design.notAvailable}: {d.not_available.map(tr).join(' · ')}</div>}
          <form className="flex gap-2" onSubmit={(e) => { e.preventDefault(); if (mod.trim()) r.run(async () => { const x = await api.designModify(d.id, mod); setD({ ...d, ...x }); setAud(null) }) }}>
            <input className="field" value={mod} onChange={(e) => setMod(e.target.value)} placeholder={t.design.modify} aria-label={t.design.modify} />
            <button className={btnGhost} disabled={r.busy}>{t.design.apply}</button>
          </form>
          <button className={btnPrimary} disabled={r.busy} onClick={() => r.run(async () => setAud(await api.designAudition(d.id)))}><AudioLines size={14} />{r.busy ? t.design.listening : t.design.listen}</button>
          {aud && <AuditionView a={aud} />}
          <div className="flex flex-wrap items-center gap-2 border-t border-line/60 pt-3">
            <input className="field !w-56" value={name} onChange={(e) => setName(e.target.value)} placeholder={t.design.name} aria-label={t.design.name} />
            <button className={small} disabled={r.busy || !name.trim()} onClick={() => r.run(async () => { await api.designSave(d.id, name.trim()); await refresh(d.id); setMsg(t.design.saved) })}><Save size={13} />{t.design.save}</button>
            {d.files.some((f) => f.kind === 'preset') && <button className={small} disabled={r.busy} onClick={() => confirm(t.design.installConfirm) && r.run(async () => { await api.designInstall(d.id); await refresh(d.id); setMsg(t.design.installed) })}><Check size={13} />{t.design.install}</button>}
            {d.files.some((f) => f.location === 'plugin_presets') && <button className={small} disabled={r.busy} onClick={() => r.run(async () => { await api.designUninstall(d.id); await refresh(d.id) })}><Undo2 size={13} />{t.design.undo}</button>}
            {d.instrument_choice && <button className={small} onClick={() => r.run(async () => { await api.pluginsRemember(role, d.instrument_choice!.product_id); setMsg(t.design.remembered) })}>{t.design.remember}</button>}
          </div>
          {msg && <p className="text-emerald-300">{msg}</p>}
          {d.files.length > 0 && <ul className="text-[11px] text-muted">{d.files.map((f) => <li key={f.path} className="truncate selectable">{f.path}</li>)}</ul>}
          <div className="flex items-center gap-2">
            <select className="field !w-40 !py-1" value={daw} onChange={(e) => setDaw(e.target.value)} aria-label="DAW">{['logic', 'ableton', 'reaper', 'fl_studio', 'cubase', 'pro_tools'].map((x) => <option key={x} value={x}>{x}</option>)}</select>
            <button className={small} onClick={() => r.run(async () => setPlan(await api.designPlan(d.id, daw)))}><Link2 size={13} />{t.design.plan}</button>
          </div>
          {plan && <ol className="list-decimal space-y-0.5 pl-5">{plan.steps.map((s, n) => <li key={n}>{s.step} <Badge tone={s.status === 'available' ? 'ok' : s.status === 'planned' ? 'info' : 'off'}>{t.dawCaps.status[s.status] ?? s.status}</Badge>{s.fallback && <span className="text-muted"> — {s.fallback}</span>}</li>)}</ol>}
        </div>
      )}
      {history.length > 0 && (
        <details className="mt-4 text-xs">
          <summary className="cursor-pointer text-muted">{t.design.history} ({history.length})</summary>
          <ul className="mt-1 space-y-0.5">{history.map((h) => <li key={h.id}><button className="hover:text-sky" onClick={() => api.designGet(h.id).then(reset)}>{h.title}</button> <span className="text-muted">· {h.status} · {h.created_at}</span></li>)}</ul>
        </details>
      )}
    </Card>
  )
}

function Changes({ d }: { d: Design }) {
  const { t, lang } = useT()
  if (!d.changes.length) return <p className="text-muted">{t.design.noChanges}</p>
  return (
    <div>
      <div className="mb-1 font-mono text-[10px] uppercase tracking-widest text-sky">{t.design.proposal}</div>
      <table className="w-full text-left">
        <thead className="text-[10px] uppercase text-muted"><tr><th>{t.design.parameter}</th><th>{t.design.from}</th><th>{t.design.to}</th><th>{t.design.reason}</th></tr></thead>
        <tbody>{d.changes.map((c) => (
          <tr key={`${c.slot}-${c.address}`} className="border-t border-line/40"><td className="py-0.5">{c.unit} · {c.parameter}</td><td>{num(c.from, 2)}</td><td className="text-violet">{num(c.to, 2)}</td><td className="text-muted">{c.reasons.map((x) => trIt(x, lang)).join(', ')}</td></tr>
        ))}</tbody>
      </table>
    </div>
  )
}

function AuditionView({ a }: { a: Audition }) {
  const { t, lang } = useT()
  return (
    <div className="space-y-2">
      <div className="grid gap-2 sm:grid-cols-2">
        {(['before', 'after'] as const).map((w) => a[w] && (
          <div key={w}>
            <div className="text-muted">{t.design[w]} · {num(a[w]!.measured.lufs)} LUFS · peak {num(a[w]!.measured.peak_dbfs)} dBFS · {num(a[w]!.measured.centroid_hz, 0)} Hz</div>
            <audio controls preload="none" src={convertFileSrc(a[w]!.file)} className="mt-1 h-8 w-full" />
          </div>
        ))}
      </div>
      {a.checks.length > 0 && (
        <table className="w-full text-left">
          <thead className="text-[10px] uppercase text-muted"><tr><th>{t.design.quality}</th><th>{t.design.metric}</th><th>{t.design.before}</th><th>{t.design.after}</th><th /></tr></thead>
          <tbody>{a.checks.map((c) => (
            <tr key={c.quality} className="border-t border-line/40"><td className="py-0.5">{trIt(c.quality, lang)} ({t.design.req[c.requested] ?? c.requested})</td><td className="text-muted">{trIt(c.metric, lang)}</td><td>{num(c.before, 2)}</td><td>{num(c.after, 2)}</td><td><Badge tone={c.as_requested ? 'ok' : 'warn'}>{c.as_requested ? t.design.ok : t.design.notOk}</Badge></td></tr>
          ))}</tbody>
        </table>
      )}
      {a.warnings.map((w) => <p key={w} className="text-amber-200">{trIt(w, lang)}</p>)}
    </div>
  )
}

export function ChainCard() {
  const { t, lang } = useT()
  const tr = (x: string) => trIt(x, lang)
  const [goal, setGoal] = useState('techno_bass')
  const [bpm, setBpm] = useState('124')
  const [file, setFile] = useState('')
  const [p, setP] = useState<ChainProposal | null>(null)
  const r = useRun()
  return (
    <Card title={t.chain.title}>
      <Note>{t.chain.note}</Note>
      <div className="mt-3 flex flex-wrap gap-2">
        <select className="field !w-52" value={goal} onChange={(e) => setGoal(e.target.value)} aria-label={t.chain.goal}>{Object.entries(t.chain.goals).map(([k, v]) => <option key={k} value={k}>{v}</option>)}</select>
        <input className="field !w-24" type="number" value={bpm} onChange={(e) => setBpm(e.target.value)} aria-label="BPM" />
        <button className={btnGhost} onClick={async () => { const f = await open({ multiple: false, directory: false, filters: [{ name: 'Audio', extensions: ['wav', 'aif', 'aiff', 'caf', 'flac', 'mp3', 'm4a'] }] }); if (typeof f === 'string') setFile(f) }}><FileAudio size={14} />{file ? file.split('/').pop() : t.chain.choose}</button>
        <button className={btnPrimary} disabled={r.busy} onClick={() => r.run(async () => setP(await api.pluginsChain(goal, file, bpm ? Number(bpm) : null, false)))}>{t.chain.propose}</button>
        {file && <button className={btnGhost} disabled={r.busy} onClick={() => r.run(async () => setP(await api.pluginsChain(goal, file, bpm ? Number(bpm) : null, true)))}><Cpu size={14} />{t.chain.check}</button>}
      </div>
      <ErrorLine error={r.error} />
      {p && (
        <div className="mt-3 space-y-2 text-xs">
          <Badge tone={p.adapted_to_material ? 'ok' : 'warn'}>{p.adapted_to_material ? t.chain.adapted : t.chain.generic}</Badge>
          <ol className="space-y-1">{p.stages.map((s, n) => (
            <li key={n} className={`rounded-lg border border-line p-2 ${s.include ? '' : 'opacity-50'}`}>
              <div className="flex flex-wrap items-center gap-2"><span className="font-semibold">{n + 1}. {t.plugins.cats[s.category] ?? s.category}</span>{!s.include && <Badge tone="warn">{t.chain.skipped}</Badge>}<span className="text-muted">{s.plugin ? s.plugin.name : t.chain.noPlugin}</span><span className="text-[10px] text-muted">{s.source}</span></div>
              <div className="text-muted">{[lang === 'it' ? s.purpose_it : s.purpose, ...s.why.map(tr)].join(' · ')}</div>
              {s.settings.length > 0 && <div>{t.chain.settings}: {s.settings.map((x) => `${x.role} ${num(x.value, 3)} ${x.unit}${x.note ? ` (${x.note})` : ''}`).join(' · ')}</div>}
            </li>
          ))}</ol>
          {p.order_issues.length > 0 && <div className="text-amber-200">{t.chain.order}: {p.order_issues.map(tr).join(' · ')}</div>}
          {p.notes.map((x) => <p key={x} className="text-amber-200">{tr(x)}</p>)}
          {p.check && (
            <div className="rounded-lg border border-line p-2">
              <div>{t.chain.check}: {p.check.rendered_stages} · in {num(p.check.input.peak_dbfs)} dBFS → out {num(p.check.output.true_peak_dbtp ?? p.check.output.peak_dbfs)} dB(TP) · ΔRMS {num(p.check.rms_change_db)} dB</div>
              {p.check.risks.length ? p.check.risks.map((x) => <p key={x} className="text-rose-300">{t.chain.risks}: {tr(x)}</p>) : <p className="text-emerald-300">{t.chain.noRisk}</p>}
            </div>
          )}
        </div>
      )}
    </Card>
  )
}

export function DawCapsCard() {
  const { t } = useT()
  const [daw, setDaw] = useState('logic')
  const [c, setC] = useState<DawCaps | null>(null)
  useEffect(() => { api.pluginsDaw(daw).then(setC, () => {}) }, [daw])
  return (
    <Card title={t.dawCaps.title} actions={<select className="field !w-40 !py-1" value={daw} onChange={(e) => setDaw(e.target.value)} aria-label="DAW">{['logic', 'ableton', 'reaper', 'fl_studio', 'cubase', 'pro_tools'].map((x) => <option key={x} value={x}>{x}</option>)}</select>}>
      {c && (
        <ul className="space-y-1 text-xs">
          {c.operations.map((o) => <li key={o.operation}><Badge tone={o.status === 'available' ? 'ok' : o.status === 'planned' ? 'info' : 'off'}>{t.dawCaps.status[o.status]}</Badge> <span className="font-medium">{t.dawCaps.ops[o.operation] ?? o.operation}</span> <span className="text-muted">— {o.route}</span></li>)}
          <li className="pt-1 text-[10px] text-muted">{c.source}</li>
        </ul>
      )}
    </Card>
  )
}

export function Diagnostics({ ov }: { ov: PluginOverview | null }) {
  const { t } = useT()
  if (!ov) return null
  const bad = Object.entries(ov.by_status).filter(([s]) => s !== 'available')
  return (
    <Card title={t.diag.title}>
      <p className="text-xs">{t.plugins.host}: {ov.host_available ? t.plugins.hostOk : t.plugins.hostNo} · {ov.os} {ov.arch}</p>
      {bad.length === 0 && ov.errors.length === 0 ? <p className="mt-1 text-xs text-emerald-300">{t.diag.none}</p> : (
        <div className="mt-1 space-y-0.5 text-xs">
          {bad.map(([s, n]) => <p key={s}>{t.plugins.statuses[s] ?? s}: {n}</p>)}
          {ov.errors.slice(0, 8).map((e, n) => <p key={n} className="text-rose-300">{e.at} · {e.operation}: {e.error}</p>)}
        </div>
      )}
    </Card>
  )
}
