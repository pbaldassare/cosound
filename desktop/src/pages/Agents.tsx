import { useEffect } from 'react'
import { Bot, Cpu } from 'lucide-react'
import { api, fmtDate } from '../lib/api'
import { useStore } from '../lib/store'
import { useT } from '../i18n'
import { Badge, Card, Dot, Empty, PageHeader, useLoad } from '../components/ui'

export default function Agents() {
  const { agent } = useStore()
  const { t, lang } = useT()
  const registry = useLoad(() => api.registry(), [])
  const tasks = useLoad(() => api.ailTasks(), [])
  useEffect(() => { const h = setInterval(tasks.reload, 3000); return () => clearInterval(h) }, [tasks.reload])
  const portName = (agent && (t.agents.ports as Record<string, string>)[agent.port_id]) ?? agent?.port_id
  const realConnected = !!agent?.connected && !agent.is_temporary

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
          </div>
          <div className="w-full sm:w-64">
            <div className="mb-1 font-mono text-[10px] uppercase tracking-widest text-sky">{t.agents.waiting}</div>
            <ul className="space-y-0.5 text-xs text-muted">{t.agents.decisions.map((d) => <li key={d}>⏳ {d}</li>)}</ul>
          </div>
        </div>
      </Card>

      <Card title={t.agents.sub}>
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {registry.data?.agents.map((a) => {
            const avail = a.capabilities.filter((c) => c.status === 'available')
            return (
              <div key={a.id} className="rounded-xl border border-line bg-panel-2/50 p-3" data-agent={a.id}>
                <div className="flex items-center gap-2">
                  <Cpu size={15} className={avail.length ? 'text-sky' : 'text-muted'} />
                  <span className="flex-1 font-medium">{a.name}</span>
                  <Dot on={avail.length > 0} />
                </div>
                <p className="mt-1 text-xs text-muted">{a.role[lang]}</p>
                <ul className="mt-2 space-y-1">
                  {a.capabilities.map((c) => (
                    <li key={c.name} className="flex items-center justify-between gap-2 font-mono text-[11px]">
                      <span className="truncate">{c.name.split('.')[1]}</span>
                      <Badge tone={c.status === 'available' ? 'ok' : 'off'}>{c.status === 'available' ? t.agents.available : t.agents.planned}</Badge>
                    </li>
                  ))}
                </ul>
              </div>
            )
          })}
        </div>
      </Card>

      <div className="grid gap-5 lg:grid-cols-2">
        <Card title={t.agents.tasks}>
          {tasks.data?.length === 0 ? <Empty>{t.agents.noTasks}</Empty> : (
            <ul className="max-h-80 divide-y divide-line/60 overflow-y-auto text-sm">
              {tasks.data?.map((k) => (
                <li key={k.task_id} className="py-2">
                  <div className="flex items-center gap-2">
                    <span className="flex-1 font-mono text-xs">{k.capability}</span>
                    <Badge tone={k.status === 'done' ? 'ok' : 'err'}>{t.agents.status[k.status]}</Badge>
                    <span className="text-[11px] text-muted">{k.duration_ms} ms</span>
                  </div>
                  <div className="text-[11px] text-muted">{fmtDate(new Date(k.started_at_ms).toISOString(), lang)}</div>
                  {k.status === 'error' && <div className="text-xs text-rose-300">{(k.output as { code: string; message: string }).code}: {(k.output as { message: string }).message}</div>}
                </li>
              ))}
            </ul>
          )}
        </Card>
        <Card title={t.agents.confirmations}><Empty>{t.agents.noConfirmations}</Empty></Card>
      </div>
    </div>
  )
}
