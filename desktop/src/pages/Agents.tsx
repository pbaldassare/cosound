import { useEffect, useState } from 'react'
import { Bot, Check, ChevronDown, Cpu, Square, Trash2, Wrench, X } from 'lucide-react'
import { api, errText, fmtDate, type AgentInfo, type TaskResult } from '../lib/api'
import { useStore } from '../lib/store'
import { fill, useT } from '../i18n'
import { Badge, Card, Dot, Empty, ErrorLine, PageHeader, Toggle, btnGhost, btnIcon, btnPrimary, useLoad } from '../components/ui'

const statusTone = { PENDING: 'off', RUNNING: 'info', WAITING_APPROVAL: 'warn', COMPLETED: 'ok', FAILED: 'err', CANCELLED: 'off' } as const
const riskTone = { read: 'off', low: 'info', medium: 'warn', high: 'err' } as const

function AgentCard({ a, onToggle }: { a: AgentInfo; onToggle: (on: boolean) => void }) {
  const { t, lang } = useT()
  const [open, setOpen] = useState(false)
  const avail = a.operations.filter((o) => o.status === 'available').length
  return (
    <div className={`rounded-xl border border-line bg-panel-2/50 p-3 ${a.enabled ? '' : 'opacity-60'}`} data-agent={a.id}>
      <div className="flex items-center gap-2">
        <Cpu size={15} className={avail ? 'text-sky' : 'text-muted'} />
        <span className="min-w-0 flex-1 truncate font-medium">{a.name}</span>
        <span className="font-mono text-[10px] text-muted">{t.agents.version} {a.version}</span>
        <Dot on={a.enabled && avail > 0} />
      </div>
      <p className="mt-1 text-xs text-muted">{a.role[lang]}</p>
      <div className="mt-2 flex flex-wrap items-center gap-1.5">
        <Badge tone={a.state === 'active' ? 'ok' : a.state === 'partial' ? 'info' : 'off'}>{t.agents.states[a.state]}</Badge>
        <Badge>{avail}/{a.operations.length} {t.agents.capabilities.toLowerCase()}</Badge>
      </div>
      <Toggle label={t.agents.enabled} checked={a.enabled} onChange={onToggle} />
      <button className="flex w-full items-center justify-between py-1 text-xs text-muted hover:text-white" aria-expanded={open} onClick={() => setOpen(!open)}>
        {t.agents.operations} · {t.agents.competencies} · {t.agents.limits}<ChevronDown size={14} className={open ? 'rotate-180' : ''} />
      </button>
      {open && (
        <div className="mt-1 space-y-3 text-xs">
          <ul className="space-y-1.5">
            {a.operations.map((o) => (
              <li key={o.name}>
                <div className="flex flex-wrap items-center gap-1.5">
                  <span className="font-mono text-[11px]">{o.name}</span>
                  <Badge tone={o.status === 'available' ? 'ok' : 'off'}>{o.status === 'available' ? t.agents.available : t.agents.planned}</Badge>
                  <Badge tone={riskTone[o.risk]}>{t.agents.risk[o.risk]}</Badge>
                  {o.requires_approval && <Badge tone="warn">{t.agents.approval}</Badge>}
                </div>
                <div className="text-muted">{o.summary[lang]}</div>
                {o.requires && <div className="text-amber-200/80">{t.agents.requires}: {o.requires}</div>}
              </li>
            ))}
          </ul>
          <div><div className="mb-0.5 font-mono text-[10px] uppercase tracking-widest text-sky">{t.agents.competencies}</div>
            <ul className="list-disc space-y-0.5 pl-4 text-muted">{a.competencies[lang].map((c) => <li key={c}>{c}</li>)}</ul></div>
          <div><div className="mb-0.5 font-mono text-[10px] uppercase tracking-widest text-sky">{t.agents.limits}</div>
            <ul className="list-disc space-y-0.5 pl-4 text-muted">{a.limits[lang].map((c) => <li key={c}>{c}</li>)}</ul></div>
          {a.permissions.length > 0 && <div className="flex flex-wrap gap-1">{a.permissions.map((p) => <Badge key={p}>{p}</Badge>)}</div>}
        </div>
      )}
    </div>
  )
}

function TaskRow({ k, onCancel }: { k: TaskResult; onCancel: () => void }) {
  const { t, lang } = useT()
  const ms = k.finished_at_ms && k.started_at_ms ? k.finished_at_ms - k.started_at_ms : null
  return (
    <li className="py-2">
      <div className="flex items-center gap-2">
        <span className="min-w-0 flex-1 truncate font-mono text-xs">{k.agent}.{k.operation}</span>
        <Badge tone={statusTone[k.status]}>{t.agents.statuses[k.status]}</Badge>
        {ms != null && <span className="text-[11px] text-muted">{ms} ms</span>}
        {(k.status === 'RUNNING' || k.status === 'PENDING') && <button className={btnIcon} title={t.agents.cancel} aria-label={t.agents.cancel} onClick={onCancel}><Square size={13} /></button>}
      </div>
      <div className="text-[11px] text-muted">{fmtDate(new Date(k.created_at_ms).toISOString(), lang)} · {t.agents.by[k.requested_by ?? ''] ?? k.requested_by}{k.basis ? ` · ${k.basis}` : ''}{k.tools_used.length ? ` · ${k.tools_used.join(', ')}` : ''}</div>
      {k.errors[0] && <div className="text-xs text-rose-300">{k.errors[0].code}: {k.errors[0].message}</div>}
      {k.limitations.length > 0 && <div className="text-[11px] text-amber-200/80">{k.limitations.join(' · ')}</div>}
    </li>
  )
}

export default function Agents() {
  const { agent, settings, setSetting } = useStore()
  const { t, lang } = useT()
  const registry = useLoad(() => api.registry(), [])
  const tasks = useLoad(() => api.agentTasks(100), [])
  const memOn = settings.perm_workflow_memory === '1'
  const memory = useLoad(() => api.memory('workflow'), [])
  const [error, setError] = useState<string | null>(null)
  useEffect(() => { const h = setInterval(tasks.reload, 2000); return () => clearInterval(h) }, [tasks.reload])
  const portName = (agent && (t.agents.ports as Record<string, string>)[agent.port_id]) ?? agent?.port_id
  const realConnected = !!agent?.connected && !agent.is_temporary
  const act = async (f: () => Promise<unknown>) => { try { await f(); setError(null) } catch (e) { setError(errText(e)) } tasks.reload(); registry.reload(); memory.reload() }
  const reg = registry.data
  const ops = reg?.agents.flatMap((a) => a.operations) ?? []
  const pending = tasks.data?.filter((k) => k.status === 'WAITING_APPROVAL') ?? []

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.agents.title} />
      <Card>
        <div className="flex flex-wrap items-start gap-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-br from-violet/40 to-electric/30"><Bot className="text-white" /></div>
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="font-display text-lg font-semibold">{t.agents.central}</h2>
              <Badge tone={realConnected ? 'ok' : 'warn'}>{realConnected ? t.shell.agentConnected : t.shell.agentNotConnected}</Badge>
              <Badge tone="violet">{t.common.inDevelopment}</Badge>
            </div>
            <p className="mt-1 text-sm text-muted">{t.agents.centralBody}</p>
            <p className="mt-2 text-xs text-muted">{t.agents.port}: <span className={agent?.is_temporary ? 'text-amber-200' : 'text-white'}>{portName}</span></p>
            {reg && <p className="mt-1 text-xs text-muted">{fill(t.agents.summary, { agents: reg.agents.length, ops: ops.filter((o) => o.status === 'available').length, total: ops.length, tools: reg.tools.filter((x) => x.status === 'available').length, alltools: reg.tools.length })}</p>}
          </div>
          <div className="w-full sm:w-64">
            <div className="mb-1 font-mono text-[10px] uppercase tracking-widest text-sky">{t.agents.waiting}</div>
            <ul className="space-y-0.5 text-xs text-muted">{t.agents.decisions.map((d) => <li key={d}>⏳ {d}</li>)}</ul>
          </div>
        </div>
      </Card>
      <ErrorLine error={error ?? registry.error} />

      <Card title={t.agents.confirmations}>
        {pending.length === 0 ? <Empty>{t.agents.noConfirmations}</Empty> : (
          <ul className="divide-y divide-line/60">
            {pending.map((k) => (
              <li key={k.task_id} className="flex flex-wrap items-center gap-3 py-2">
                <div className="min-w-0 flex-1">
                  <div className="font-mono text-xs">{k.agent}.{k.operation} <Badge tone={riskTone[(k.approval?.risk ?? 'medium') as keyof typeof riskTone]}>{t.agents.risk[k.approval?.risk ?? 'medium']}</Badge></div>
                  <div className="text-xs text-muted">{reg?.agents.find((a) => a.id === k.agent)?.operations.find((o) => o.name === k.operation)?.summary[lang] ?? k.approval?.summary}</div>
                </div>
                <button className={btnPrimary + ' !py-1'} onClick={() => act(() => api.approve(k.approval!.approval_id, true))}><Check size={14} />{t.agents.approve}</button>
                <button className={btnGhost + ' !py-1'} onClick={() => act(() => api.approve(k.approval!.approval_id, false))}><X size={14} />{t.agents.reject}</button>
              </li>
            ))}
          </ul>
        )}
      </Card>

      <Card title={t.agents.sub}>
        <div className="grid items-start gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {reg?.agents.map((a) => <AgentCard key={a.id} a={a} onToggle={(on) => act(() => api.setAgentEnabled(a.id, on))} />)}
        </div>
      </Card>

      <div className="grid gap-5 lg:grid-cols-2">
        <Card title={t.agents.tasks}>
          {tasks.data?.length === 0 ? <Empty>{t.agents.noTasks}</Empty> : (
            <ul className="max-h-[28rem] divide-y divide-line/60 overflow-y-auto text-sm">
              {tasks.data?.map((k) => <TaskRow key={k.task_id} k={k} onCancel={() => act(() => api.cancelTask(k.task_id))} />)}
            </ul>
          )}
        </Card>
        <div className="space-y-5">
          <Card title={t.agents.memory}>
            <p className="text-xs text-muted">{t.agents.memoryBody}</p>
            <Toggle label={t.agents.memoryOn} checked={memOn} onChange={(on) => act(() => setSetting('perm_workflow_memory', on ? '1' : '0'))} />
            {memory.data?.length === 0 ? <Empty>{t.agents.memoryEmpty}</Empty> : (
              <ul className="max-h-56 divide-y divide-line/60 overflow-y-auto">
                {memory.data?.map((f) => (
                  <li key={f.id} className="flex items-center gap-2 py-1.5 text-sm">
                    <div className="min-w-0 flex-1">
                      <div className="truncate">{f.key} <Badge tone={f.scope === 'custom' ? 'violet' : 'off'}>{t.agents.scope[f.scope]}</Badge></div>
                      <div className="truncate font-mono text-[11px] text-muted">{[f.value.daw, f.value.keys, Array.isArray(f.value.steps) ? (f.value.steps as string[]).join(' → ') : f.value.value].filter(Boolean).join(' · ')}</div>
                    </div>
                    <button className={btnIcon} title={t.common.remove} aria-label={t.common.remove} onClick={() => act(() => api.memoryDelete(f.id))}><Trash2 size={14} /></button>
                  </li>
                ))}
              </ul>
            )}
            {!!memory.data?.length && <button className={btnGhost + ' mt-2 !py-1 !text-xs'} onClick={() => { if (confirm(t.agents.memoryClearConfirm)) act(() => api.memoryClear('workflow')) }}><Trash2 size={13} />{t.agents.memoryClear}</button>}
          </Card>
          <Card title={t.agents.tools}>
            <ul className="max-h-72 space-y-1 overflow-y-auto text-xs">
              {reg?.tools.map((x) => (
                <li key={x.id} className="flex items-start gap-2">
                  <Wrench size={12} className={`mt-0.5 shrink-0 ${x.status === 'available' ? 'text-sky' : 'text-muted'}`} />
                  <div className="min-w-0 flex-1">
                    <span className="font-mono">{x.id}</span> <Badge tone={x.status === 'available' ? 'ok' : 'off'}>{x.status === 'available' ? t.agents.available : t.agents.planned}</Badge>
                    <div className="text-muted">{x.description[lang]}{x.requires ? ` · ${t.agents.requires}: ${x.requires}` : ''}{x.used_by.length ? ` · ${t.agents.usedBy} ${x.used_by.length}` : ''}</div>
                  </div>
                </li>
              ))}
            </ul>
          </Card>
        </div>
      </div>
    </div>
  )
}
