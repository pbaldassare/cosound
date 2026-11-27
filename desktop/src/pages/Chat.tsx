import { useRef, useState, type FormEvent } from 'react'
import { MessageSquarePlus, Send, ShieldAlert, Sparkles } from 'lucide-react'
import { api, errText, type AgentEvent, type Interpretation } from '../lib/api'
import { useStore } from '../lib/store'
import { fill, useT } from '../i18n'
import { VoicePanel } from '../components/Voice'
import { btnGhost, btnPrimary } from '../components/ui'

type Msg = { id: number; from: 'user' | 'assistant' | 'system'; text: string; tone?: 'ok' | 'err'; detail?: string; interp?: Interpretation }

const show = (v: unknown) => (typeof v === 'object' && v !== null ? Object.entries(v as Record<string, unknown>).filter(([, x]) => x !== null).map(([k, x]) => (k === 'id' || k === 'direction' || k === 'db' ? String(x) : `${k} ${x}`)).join(' ') : String(v))

/** What the Prompt Engine understood: intents, elements, parameters said or deduced, constraints, approvals. */
function Understanding({ i, onAnswer }: { i: Interpretation; onAnswer: (s: string) => void }) {
  const { t } = useT()
  const u = t.understanding
  const chip = 'rounded-full border border-line px-2 py-0.5 text-[11px]'
  return (
    <div className="mt-2 space-y-1.5 text-xs">
      <div className="flex flex-wrap items-center gap-1">
        <span className={chip + ' border-violet/50 text-violet'}>{u.acts[i.conversation.act] ?? i.conversation.act}</span>
        {i.intents.map((x) => <span key={x.id} className={chip}>{u.intents[x.id] ?? x.id}</span>)}
        {i.targets.map((x, k) => <span key={k} className={chip + ' border-sky/40 text-sky'}>{x.track_name ?? x.element}</span>)}
        <span className="text-muted">{u.confidence} {Math.round(i.confidence * 100)}%</span>
      </div>
      {i.clarification.needed && i.clarification.options.length > 0 && (
        <div className="flex flex-wrap gap-1">{i.clarification.options.slice(0, 8).map((o) => <button key={o} className={btnGhost + ' !px-2 !py-0.5 !text-xs'} onClick={() => onAnswer(o)}>{o}</button>)}</div>
      )}
      {i.approval.required && (
        <p className="flex items-center gap-1 text-amber-200"><ShieldAlert size={12} />{i.approval.destructive_operations.length ? `${u.destructive} ${i.approval.destructive_operations.map((d) => { const tr = i.targets.find((x) => x.element === d.subject)?.track_name; return `${u.removes[d.kind] ?? d.kind} (${tr ?? u.elements[d.subject] ?? d.subject})` }).join(', ')}.` : u.approval}</p>
      )}
      {i.issues.filter((x) => x.severity !== 'info').map((x, k) => <p key={k} className={x.severity === 'error' ? 'text-rose-300' : 'text-amber-200'}>{x.message}{x.suggestion ? ` ${x.suggestion}` : ''}</p>)}
      <details>
        <summary className="cursor-pointer text-muted">{u.details}</summary>
        <div className="mt-1 space-y-1">
          {i.parameters.explicit.length > 0 && <p><span className="text-muted">{u.said}:</span> {i.parameters.explicit.map((p) => `${p.name} ${show(p.value)}${p.unit ? ' ' + p.unit : ''}`).join(' · ')}</p>}
          {i.parameters.inferred.length > 0 && <p><span className="text-muted">{u.deduced}:</span> {i.parameters.inferred.map((p) => `${p.name} ${show(p.value)} (${p.evidence ?? p.source})`).join(' · ')}</p>}
          {i.defaults_used.length > 0 && <p><span className="text-muted">{u.defaults}:</span> {i.defaults_used.map((d) => `${d.name} ${String(d.value)}`).join(' · ')}</p>}
          {i.constraints.length > 0 && <p><span className="text-muted">{u.constraints}:</span> {i.constraints.map((c) => `${c.kind} ${c.subject}${c.scope !== 'request' ? ` (${c.scope})` : ''}`).join(' · ')}</p>}
          {i.enhancement.semantic && <p className="italic text-muted">{i.enhancement.semantic}</p>}
          {i.enhancement.suggested_technical.length > 0 && <p><span className="text-muted">{u.technical}:</span> {i.enhancement.suggested_technical.map((x) => `${x.aspect} → ${x.direction}`).join(' · ')}</p>}
          {i.instructions.length > 0 && (
            <div><span className="text-muted">{u.instructions}:</span>
              <ul className="ml-3 list-disc">{i.instructions.map((x, k) => <li key={k}>{x.component} · <span className="font-mono">{x.agent}{x.suggested_operation ? `.${x.suggested_operation}` : ''}</span>{!x.operation_available && x.suggested_operation ? ` (${u.planned})` : ''}{x.requires_approval ? ' · ✋' : ''}</li>)}</ul>
            </div>
          )}
          <p className="font-mono text-[10px] text-muted">schema {i.schema_version} · {i.interpreter.adapter}{i.interpreter.fallback ? ' (fallback)' : ''} · {i.request_id.slice(0, 8)}</p>
        </div>
      </details>
    </div>
  )
}

export default function Chat() {
  const { assistantName, agent } = useStore()
  const { t } = useT()
  const [msgs, setMsgs] = useState<Msg[]>([])
  const [text, setText] = useState('')
  const [busy, setBusy] = useState(false)
  const n = useRef(0)
  const end = useRef<HTMLDivElement>(null)

  const toMsg = (e: AgentEvent): Msg => {
    const id = ++n.current
    if (e.kind === 'assistant_text')
      // a temporary (MOCK) port is never presented as the assistant
      return e.text === 'TEMP_CONNECTOR_EXPLICIT_ONLY' ? { id, from: 'system', text: t.chat.tempExplicitOnly } : { id, from: agent?.is_temporary ? 'system' : 'assistant', text: e.text }
    if (e.kind === 'interpretation') return { id, from: 'assistant', text: e.interpretation.reply.text, interp: e.interpretation }
    if (e.kind === 'task_result') {
      const r = e.result
      const tone = r.status === 'COMPLETED' ? 'ok' : r.status === 'FAILED' || r.status === 'CANCELLED' ? 'err' : undefined
      const err = r.errors[0] ? `\n${r.errors[0].code}: ${r.errors[0].message}` : ''
      const lim = r.limitations.length ? `\n${t.chat.limits}: ${r.limitations.join(' · ')}` : ''
      return { id, from: 'system', tone, text: fill(t.chat.task, { cap: `${r.agent}.${r.operation}`, status: t.agents.statuses[r.status] }) + err + lim, detail: r.result ? JSON.stringify(r.result, null, 2) : undefined }
    }
    if (e.error.code === 'AGENT_UNAVAILABLE') return { id, from: 'system', text: t.chat.agentUnavailable }
    return { id, from: 'system', tone: 'err', text: `${e.error.code}: ${e.error.message}` }
  }

  async function send(ev?: FormEvent, override?: string) {
    ev?.preventDefault()
    const body = (override ?? text).trim()
    if (!body || busy) return
    if (override === undefined) setText('')
    setBusy(true)
    setMsgs((m) => [...m, { id: ++n.current, from: 'user', text: body }])
    try {
      const events = await api.ailSend(body)
      const understood = events.some((e) => e.kind === 'interpretation')
      // with an interpretation, "Agent not connected" is said in the context of what was understood
      setMsgs((m) => [...m, ...events.map((e) => (understood && e.kind === 'error' && e.error.code === 'AGENT_UNAVAILABLE' ? { id: ++n.current, from: 'system' as const, text: t.understanding.notConnected } : toMsg(e)))])
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
        <div className="flex items-center border-b border-line px-4 py-3">
          <span className="flex-1 font-display font-semibold">{fill(t.chat.title, { name: assistantName })}</span>
          <button className={btnGhost + ' !px-2 !py-1 !text-xs'} onClick={() => api.promptNewSession().then(() => setMsgs([]))} title={t.understanding.newChat}><MessageSquarePlus size={14} />{t.understanding.newChat}</button>
        </div>
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
                {m.interp && <Understanding i={m.interp} onAnswer={(o) => send(undefined, o)} />}
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
