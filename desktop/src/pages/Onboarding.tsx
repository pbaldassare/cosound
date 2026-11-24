import { useEffect, useState } from 'react'
import { Check, FolderPlus, Mic } from 'lucide-react'
import { api, errText, type DawFound, type Root } from '../lib/api'
import { useStore } from '../lib/store'
import { supabase } from '../lib/supabase'
import { fill, LanguageToggle, useT, type Lang } from '../i18n'
import { AuthForm } from './Account'
import { authorizeFolder } from './Library'
import { saveAssistantName } from './Settings'
import { requestMic } from '../components/Voice'
import { DAW_IDS } from './Production'
import { Badge, Dot, ErrorLine, Note, btnGhost, btnPrimary } from '../components/ui'

const NAMES = ['Nexus', 'Alex', 'Sonic', 'Athena']
const GENRES = ['House', 'Techno', 'Hip hop', 'Trap', 'Pop', 'R&B', 'Rock', 'EDM', 'Drum & bass', 'Ambient', 'Lo-fi', 'Afro house', 'Reggaeton', 'Cinematic']

export default function Onboarding() {
  const { settings, setSetting, session } = useStore()
  const { t, lang, setLang } = useT()
  const [step, setStep] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const [name, setName] = useState(settings.assistant_name ?? '')
  const [roots, setRoots] = useState<Root[]>([])
  const [toIndex, setToIndex] = useState<Set<string>>(new Set())
  const [daws, setDaws] = useState<DawFound[] | null>(null)
  const [mic, setMic] = useState<boolean | null>(settings.perm_microphone === '1' ? true : null)
  const [genres, setGenres] = useState<string[]>(settings.pref_genres ? JSON.parse(settings.pref_genres) : [])
  const [mainDaw, setMainDaw] = useState(settings.pref_daw ?? '')
  const [tempo, setTempo] = useState(settings.pref_bpm ?? '')
  const total = t.onboarding.steps.length

  useEffect(() => { api.roots().then(setRoots) }, [step])
  useEffect(() => { if (step === 5 && !daws) api.daws().then(setDaws) }, [step, daws])

  async function next() {
    setError(null)
    try {
      if (step === 1) await saveAssistantName(name || 'COSOUND', setSetting, !!session)
      if (step === 1 && session) await setSetting('cloud_sync_assistant_name', '1')
      if (step === 4) for (const id of toIndex) await api.scan(id)
      if (step === total - 1) {
        await setSetting('pref_genres', JSON.stringify(genres))
        await setSetting('pref_daw', mainDaw)
        await setSetting('pref_bpm', tempo)
        if (!settings.language) await setSetting('language', lang)
        await setSetting('onboarding_done', '1')
        return
      }
      setStep(step + 1)
    } catch (e) { setError(errText(e)) }
  }

  const addFolder = async () => {
    setError(null)
    try { const id = await authorizeFolder(); if (id) { setRoots(await api.roots()); setToIndex((s) => new Set(s).add(id)) } } catch (e) { setError(errText(e)) }
  }

  const body = [
    // 1 account
    <>
      <p className="text-muted">{t.onboarding.accountBody}</p>
      {session ? <p className="flex items-center gap-2 text-emerald-300"><Check size={16} />{session.user.email}</p> : supabase && <AuthForm />}
    </>,
    // 2 name
    <>
      <p className="text-muted">{t.onboarding.nameBody}</p>
      <input className="field !text-lg" maxLength={40} autoFocus value={name} onChange={(e) => setName(e.target.value)} placeholder="Nexus" aria-label={t.settings.assistantName} />
      <div className="flex flex-wrap items-center gap-2 text-sm"><span className="text-muted">{t.onboarding.nameSuggestions}:</span>
        {NAMES.map((n) => <button key={n} className={btnGhost + ' !px-3 !py-1'} onClick={() => setName(n)}>{n}</button>)}
      </div>
      <p className="text-xs text-muted">{t.settings.assistantNameHelp}</p>
    </>,
    // 3 language
    <>
      <p className="text-muted">{t.onboarding.langBody}</p>
      <div className="flex gap-3">
        {([['it', 'Italiano'], ['en', 'English']] as [Lang, string][]).map(([l, label]) => (
          <button key={l} onClick={() => setLang(l)} aria-pressed={lang === l} className={`flex-1 rounded-xl border px-4 py-4 text-left transition ${lang === l ? 'border-violet bg-violet/10' : 'border-line hover:border-violet/50'}`}>
            <div className="font-display text-lg font-semibold">{label}</div>
          </button>
        ))}
      </div>
    </>,
    // 4 folders
    <>
      <p className="text-muted">{t.onboarding.foldersBody}</p>
      <button className={btnPrimary} onClick={addFolder}><FolderPlus size={16} />{t.library.addFolder}</button>
      <ul className="space-y-1 text-sm">{roots.map((r) => <li key={r.id} className="flex items-center gap-2"><Dot on={r.online} /><span className="truncate">{r.path}</span><Badge>{t.library.kinds[r.volume_kind as 'internal']}</Badge></li>)}</ul>
      <Note>{t.library.readOnly}</Note>
    </>,
    // 5 index
    <>
      <p className="text-muted">{t.onboarding.indexBody}</p>
      {roots.length === 0 ? <Note>{t.library.noFolders}</Note> : (
        <ul className="space-y-1.5 text-sm">
          {roots.map((r) => (
            <li key={r.id}><label className="flex items-center gap-2">
              <input type="checkbox" checked={toIndex.has(r.id)} onChange={(e) => setToIndex((s) => { const n = new Set(s); if (e.target.checked) n.add(r.id); else n.delete(r.id); return n })} />
              <span className="truncate">{r.path}</span>
            </label></li>
          ))}
        </ul>
      )}
    </>,
    // 6 DAWs
    <>
      <p className="text-muted">{t.onboarding.dawBody}</p>
      <ul className="grid grid-cols-2 gap-2 text-sm">
        {(daws ?? []).map((d) => (
          <li key={d.id} className="flex items-center gap-2 rounded-lg border border-line px-3 py-2">
            <span className="flex-1">{t.production.daws[d.id]}</span>
            {!d.supported_on_this_os ? <Badge>{t.daws.notSupported}</Badge> : d.detected ? <Badge tone="ok">{t.daws.detected}{d.version ? ` ${d.version}` : ''}</Badge> : <Badge>{t.daws.notDetected}</Badge>}
          </li>
        ))}
      </ul>
      {daws === null && <p className="text-sm text-muted">{t.common.loading}</p>}
      <Note>{t.daws.note}</Note>
    </>,
    // 7 microphone
    <>
      <p className="text-muted">{t.onboarding.micBody}</p>
      <div className="flex gap-2">
        <button className={btnPrimary} onClick={async () => { const ok = await requestMic(); setMic(ok); await setSetting('perm_microphone', ok ? '1' : '0') }}><Mic size={16} />{t.onboarding.micAllow}</button>
        <button className={btnGhost} onClick={async () => { setMic(null); await setSetting('perm_microphone', '0'); setStep(step + 1) }}>{t.onboarding.micSkip}</button>
      </div>
      {mic === true && <p className="flex items-center gap-2 text-emerald-300"><Check size={16} />{t.voice.micReady}</p>}
      {mic === false && <p className="text-amber-200">{t.voice.micDenied}</p>}
      <p className="text-xs text-muted">{t.settings.micHelp}</p>
    </>,
    // 8 preferences
    <>
      <p className="text-muted">{t.onboarding.prefsBody}</p>
      <div>
        <div className="mb-1.5 text-sm font-medium">{t.onboarding.genres}</div>
        <div className="flex flex-wrap gap-1.5">
          {GENRES.map((g) => (
            <button key={g} aria-pressed={genres.includes(g)} onClick={() => setGenres((s) => (s.includes(g) ? s.filter((x) => x !== g) : [...s, g]))}
              className={`rounded-full border px-3 py-1 text-xs transition ${genres.includes(g) ? 'border-violet bg-violet/15' : 'border-line text-muted hover:text-white'}`}>{g}</button>
          ))}
        </div>
      </div>
      <div className="grid grid-cols-2 gap-3">
        <label className="text-sm font-medium">{t.onboarding.mainDaw}
          <select className="field mt-1" value={mainDaw} onChange={(e) => setMainDaw(e.target.value)}>
            <option value="">—</option>{DAW_IDS.map((d) => <option key={d} value={d}>{t.production.daws[d]}</option>)}
          </select>
        </label>
        <label className="text-sm font-medium">{t.onboarding.tempo}<input className="field mt-1" type="number" min={20} max={400} value={tempo} onChange={(e) => setTempo(e.target.value)} /></label>
      </div>
    </>,
  ]

  return (
    <div className="grid-bg flex h-full items-center justify-center overflow-y-auto p-6">
      <div className="card fade-in w-full max-w-2xl p-8 shadow-[0_0_80px_-30px_rgb(168_85_247/0.6)]">
        <div className="mb-6 flex items-center justify-between">
          <div className="flex items-center gap-2.5 font-display font-bold tracking-[0.18em]"><img src="/favicon.svg" alt="" className="h-7 w-7" />COSOUND</div>
          <LanguageToggle />
        </div>
        <p className="font-mono text-xs uppercase tracking-[0.25em] text-sky">{fill(t.onboarding.step, { n: step + 1, total })}</p>
        <h1 className="mt-1 font-display text-3xl font-bold">{step === 0 ? t.onboarding.title : t.onboarding.steps[step]}</h1>
        <div className="mt-2 flex gap-1" aria-hidden>{t.onboarding.steps.map((_, i) => <span key={i} className={`h-1 flex-1 rounded-full ${i <= step ? 'bg-gradient-to-r from-violet to-sky' : 'bg-white/10'}`} />)}</div>
        <div key={step} className="fade-in mt-6 min-h-[220px] space-y-4">{body[step]}</div>
        <ErrorLine error={error} />
        <div className="mt-6 flex items-center justify-between">
          <button className={btnGhost} disabled={step === 0} onClick={() => setStep(step - 1)}>{t.common.back}</button>
          <div className="flex gap-2">
            {[0, 3, 4].includes(step) && <button className="px-3 text-sm text-muted hover:text-white" onClick={() => setStep(step + 1)}>{t.common.skip}</button>}
            <button className={btnPrimary} onClick={next} data-next>{step === total - 1 ? t.common.finish : t.common.next}</button>
          </div>
        </div>
      </div>
    </div>
  )
}
