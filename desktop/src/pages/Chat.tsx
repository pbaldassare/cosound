import { useRef, useState, type FormEvent } from 'react'
import { Send, Sparkles } from 'lucide-react'
import { api, errText, type AgentEvent } from '../lib/api'
import { useStore } from '../lib/store'
import { fill, useT } from '../i18n'
import { VoicePanel } from '../components/Voice'
import { btnPrimary } from '../components/ui'

type Msg = { id: number; from: 'user' | 'assistant' | 'system'; text: string; tone?: 'ok' | 'err'; detail?: string }

export default function Chat() {
  const { assistantName } = useStore()
  const { t } = useT()
  const [msgs, setMsgs] = useState<Msg[]>([])
  const [text, setText] = useState('')
  const [busy, setBusy] = useState(false)
  const n = useRef(0)
  const end = useRef<HTMLDivElement>(null)

  const toMsg = (e: AgentEvent): Msg => {
    const id = ++n.current
    if (e.kind === 'assistant_text')
      return e.text === 'TEMP_CONNECTOR_EXPLICIT_ONLY' ? { id, from: 'system', text: t.chat.tempExplicitOnly } : { id, from: 'assistant', text: e.text }
    if (e.kind === 'capability_result')
      return { id, from: 'system', tone: e.ok ? 'ok' : 'err', text: fill(e.ok ? t.chat.capOk : t.chat.capErr, { cap: e.capability }), detail: JSON.stringify(e.output, null, 2) }
    if (e.error.code === 'AGENT_UNAVAILABLE') return { id, from: 'system', text: t.chat.agentUnavailable }
    return { id, from: 'system', tone: 'err', text: `${e.error.code}: ${e.error.message}` }
  }

  async function send(ev: FormEvent) {
    ev.preventDefault()
    const body = text.trim()
    if (!body || busy) return
    setText('')
    setBusy(true)
    setMsgs((m) => [...m, { id: ++n.current, from: 'user', text: body }])
    try {
      const events = await api.ailSend(body)
      setMsgs((m) => [...m, ...events.map(toMsg)])
    } catch (e) {
      setMsgs((m) => [...m, { id: ++n.current, from: 'system', tone: 'err', text: errText(e) }])
    } finally {
      setBusy(false)
      setTimeout(() => end.current?.scrollIntoView({ behavior: 'smooth' }), 0)
    }
  }

  const label = (m: Msg) => (m.from === 'user' ? t.chat.you : m.from === 'assistant' ? assistantName : t.chat.system)

  return (
    <div className="mx-auto grid h-full max-w-6xl gap-5 lg:grid-cols-[minmax(0,1fr)_340px]">
      <section className="card flex min-h-[480px] flex-col overflow-hidden">
        <div className="border-b border-line px-4 py-3 font-display font-semibold">{fill(t.chat.title, { name: assistantName })}</div>
        <div className="flex-1 space-y-3 overflow-y-auto p-4" aria-live="polite">
          {msgs.length === 0 && (
            <div className="mx-auto mt-10 max-w-md text-center">
              <Sparkles className="mx-auto mb-3 text-violet" />
              <h2 className="font-display text-lg font-semibold">{t.chat.emptyTitle}</h2>
              <p className="mt-2 text-sm text-muted">{fill(t.chat.emptyBody, { name: assistantName })}</p>
            </div>
          )}
          {msgs.map((m) => (
            <div key={m.id} className={`fade-in flex ${m.from === 'user' ? 'justify-end' : 'justify-start'}`}>
              <div className={`max-w-[80%] rounded-2xl px-3.5 py-2.5 text-sm selectable ${m.from === 'user' ? 'bg-gradient-to-br from-violet/30 to-electric/25 text-white' : m.tone === 'err' ? 'border border-rose-400/30 bg-rose-400/10' : m.tone === 'ok' ? 'border border-emerald-400/30 bg-emerald-400/5' : 'border border-line bg-panel-2'}`}>
                <div className="mb-0.5 font-mono text-[10px] uppercase tracking-widest text-muted">{label(m)}</div>
                <p className="whitespace-pre-wrap">{m.text}</p>
                {m.detail && <pre className="mt-2 max-h-56 overflow-auto rounded-lg bg-ink p-2 font-mono text-[11px] text-muted">{m.detail}</pre>}
              </div>
            </div>
          ))}
          <div ref={end} />
        </div>
        <form onSubmit={send} className="flex gap-2 border-t border-line p-3">
          <input className="field" value={text} onChange={(e) => setText(e.target.value)} placeholder={fill(t.chat.placeholder, { name: assistantName })} aria-label={t.chat.send} />
          <button className={btnPrimary} disabled={busy || !text.trim()}><Send size={15} />{t.chat.send}</button>
        </form>
      </section>
      <VoicePanel />
    </div>
  )
}
