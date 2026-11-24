import { useEffect, useRef, useState } from 'react'
import { Mic, Square } from 'lucide-react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { api, errText, fmtDate, type Import } from '../lib/api'
import { useStore } from '../lib/store'
import { useT } from '../i18n'
import { Badge, Card, ErrorLine, Note, btnGhost, btnPrimary } from './ui'

const MODES = ['speak', 'describe', 'sing', 'whistle', 'hum', 'beatbox'] as const
type MicState = 'off' | 'unknown' | 'ready' | 'denied'

/** Ask the OS/WebView for the microphone once, then release it. */
export async function requestMic(): Promise<boolean> {
  try {
    const s = await navigator.mediaDevices.getUserMedia({ audio: true })
    s.getTracks().forEach((t) => t.stop())
    return true
  } catch {
    return false
  }
}

export function VoicePanel() {
  const { settings, setSetting, project } = useStore()
  const { t, lang } = useT()
  const enabled = settings.perm_microphone === '1'
  const [mic, setMic] = useState<MicState>(enabled ? 'unknown' : 'off')
  const [mode, setMode] = useState<(typeof MODES)[number]>('hum')
  const [recording, setRecording] = useState(false)
  const [level, setLevel] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const [msg, setMsg] = useState<string | null>(null)
  const [takes, setTakes] = useState<Import[]>([])
  const rec = useRef<{ mr: MediaRecorder; stream: MediaStream; ctx: AudioContext; raf: number } | null>(null)

  const loadTakes = () => api.imports(project?.id ?? null).then((all) => setTakes(all.filter((i) => i.role === 'voice')))
  useEffect(() => { loadTakes() }, [project?.id]) // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => { setMic(enabled ? (mic === 'off' ? 'unknown' : mic) : 'off') }, [enabled]) // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => () => stopAll(), [])

  function stopAll() {
    const r = rec.current
    if (!r) return
    cancelAnimationFrame(r.raf)
    r.stream.getTracks().forEach((tr) => tr.stop())
    r.ctx.close()
    rec.current = null
  }

  async function start() {
    setError(null); setMsg(null)
    let stream: MediaStream
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false } })
    } catch (e) {
      setMic('denied'); setError(errText(e)); return
    }
    setMic('ready')
    const ctx = new AudioContext()
    const an = ctx.createAnalyser()
    ctx.createMediaStreamSource(stream).connect(an)
    const buf = new Float32Array(an.fftSize)
    const tick = () => {
      an.getFloatTimeDomainData(buf)
      setLevel(Math.min(1, Math.sqrt(buf.reduce((s, v) => s + v * v, 0) / buf.length) * 4))
      if (rec.current) rec.current.raf = requestAnimationFrame(tick)
    }
    const chunks: Blob[] = []
    const mr = new MediaRecorder(stream)
    mr.ondataavailable = (e) => chunks.push(e.data)
    mr.onstop = async () => {
      const blob = new Blob(chunks, { type: mr.mimeType })
      const ext = mr.mimeType.includes('mp4') ? 'mp4' : mr.mimeType.includes('ogg') ? 'ogg' : 'webm'
      try {
        await api.saveTake(project?.id ?? null, Array.from(new Uint8Array(await blob.arrayBuffer())), ext, mode)
        setMsg(t.voice.saved)
        loadTakes()
      } catch (e) { setError(errText(e)) }
    }
    rec.current = { mr, stream, ctx, raf: requestAnimationFrame(tick) }
    mr.start()
    setRecording(true)
  }

  function stop() {
    rec.current?.mr.stop()
    stopAll()
    setRecording(false)
    setLevel(0)
  }

  const micBadge = { off: ['off', t.voice.micOff], unknown: ['warn', t.voice.micUnknown], ready: ['ok', t.voice.micReady], denied: ['err', t.voice.micDenied] } as const

  return (
    <Card title={t.voice.title} actions={<Badge tone={micBadge[mic][0]}>{micBadge[mic][1]}</Badge>}>
      <div className="mb-3 flex flex-wrap gap-1.5" role="radiogroup" aria-label={t.voice.title}>
        {MODES.map((m) => (
          <button key={m} role="radio" aria-checked={mode === m} onClick={() => setMode(m)} disabled={recording}
            className={`rounded-full border px-2.5 py-1 text-xs transition ${mode === m ? 'border-violet/60 bg-violet/15 text-white' : 'border-line text-muted hover:text-white'}`}>
            {t.voice.modes[m]}
          </button>
        ))}
      </div>
      <div className="flex items-center gap-3">
        {!enabled ? (
          <button className={btnGhost} onClick={() => setSetting('perm_microphone', '1')}><Mic size={16} />{t.voice.enableMic}</button>
        ) : recording ? (
          <button className={btnPrimary + ' !from-rose-500 !via-rose-500 !to-violet'} onClick={stop}><Square size={14} />{t.voice.stop}</button>
        ) : (
          <button className={btnPrimary} onClick={start}><Mic size={16} />{t.voice.record}</button>
        )}
        <div className="h-2 flex-1 overflow-hidden rounded-full bg-white/5" aria-hidden>
          <div className="h-full rounded-full bg-gradient-to-r from-electric via-violet to-sky transition-[width] duration-75" style={{ width: `${level * 100}%` }} />
        </div>
        {recording && <span className="pulse-dot text-xs text-rose-300">● {t.voice.recording}</span>}
      </div>
      <div className="mt-3 space-y-2">
        <ErrorLine error={error} />
        {msg && <p className="text-xs text-emerald-300">{msg}</p>}
        <Note>{t.voice.notConverted}</Note>
        {takes.length === 0 ? <p className="text-xs text-muted">{t.voice.empty}</p> : (
          <ul className="max-h-48 space-y-1.5 overflow-y-auto">
            {takes.map((tk) => (
              <li key={tk.id} className="flex items-center gap-2 text-xs">
                <span className="w-36 shrink-0 text-muted">{fmtDate(tk.imported_at, lang)}</span>
                <audio controls preload="none" src={convertFileSrc(tk.path)} className="h-7 flex-1" />
              </li>
            ))}
          </ul>
        )}
      </div>
    </Card>
  )
}
