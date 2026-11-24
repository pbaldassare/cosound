import { useState } from 'react'
import { RefreshCw } from 'lucide-react'
import { api } from '../lib/api'
import { useStore } from '../lib/store'
import { fill, useT } from '../i18n'
import { Badge, Card, Empty, ErrorLine, Note, PageHeader, btnPrimary, useLoad } from '../components/ui'

export default function Plugins() {
  const { settings } = useStore()
  const { t } = useT()
  const allowed = settings.perm_plugin_scan !== '0'
  const list = useLoad(() => (allowed ? api.plugins() : Promise.resolve([])), [allowed])
  const [q, setQ] = useState('')
  const [format, setFormat] = useState('')
  const rows = (list.data ?? []).filter((p) => (!format || p.format === format) && `${p.name} ${p.vendor ?? ''}`.toLowerCase().includes(q.toLowerCase()))

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.plugins.title}>
        <button className={btnPrimary} disabled={!allowed || list.loading} onClick={list.reload}><RefreshCw size={15} className={list.loading ? 'animate-spin' : ''} />{t.plugins.scan}</button>
      </PageHeader>
      <Note>{allowed ? t.plugins.note : `${t.settings.pluginScan}: ${t.common.no}`}</Note>
      <ErrorLine error={list.error} />
      <Card title={fill(t.plugins.found, { n: rows.length })} actions={
        <div className="flex gap-2">
          <input className="field !w-48 !py-1" placeholder={t.plugins.filter} aria-label={t.plugins.filter} value={q} onChange={(e) => setQ(e.target.value)} />
          <select className="field !w-28 !py-1" aria-label={t.plugins.format} value={format} onChange={(e) => setFormat(e.target.value)}>
            <option value="">{t.common.all}</option>
            {['VST3', 'AU', 'CLAP', 'VST2'].map((f) => <option key={f}>{f}</option>)}
          </select>
        </div>
      }>
        {!list.loading && rows.length === 0 ? <Empty>{t.plugins.empty}</Empty> : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead className="text-[11px] uppercase tracking-wider text-muted">
                <tr>{[t.plugins.name, t.plugins.format, t.plugins.kind, t.plugins.vendor, t.plugins.version, t.plugins.scope, t.plugins.control].map((h) => <th key={h} className="px-2 py-2 font-medium">{h}</th>)}</tr>
              </thead>
              <tbody className="divide-y divide-line/50">
                {rows.map((p) => (
                  <tr key={p.path} title={p.path}>
                    <td className="px-2 py-1.5 font-medium selectable">{p.name}</td>
                    <td className="px-2"><Badge tone="info">{p.format}</Badge></td>
                    <td className="px-2">{p.kind === 'unknown' ? '—' : <Badge tone={p.kind === 'instrument' ? 'violet' : 'off'}>{t.plugins.kinds[p.kind]}</Badge>}</td>
                    <td className="px-2 text-muted">{p.vendor ?? '—'}</td>
                    <td className="px-2 text-muted">{p.version ?? '—'}</td>
                    <td className="px-2 text-muted">{t.plugins.scopes[p.scope]}</td>
                    <td className="px-2 text-xs text-muted">{t.plugins.notMeasured}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>
    </div>
  )
}
