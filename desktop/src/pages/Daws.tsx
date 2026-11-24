import { RefreshCw } from 'lucide-react'
import { api } from '../lib/api'
import { useStore } from '../lib/store'
import { useT } from '../i18n'
import { Badge, Card, Dot, ErrorLine, Note, PageHeader, btnPrimary, useLoad } from '../components/ui'

const FIRST = new Set(['logic', 'ableton', 'fl', 'protools'])

export default function Daws() {
  const { settings } = useStore()
  const { t } = useT()
  const allowed = settings.perm_daw_detection !== '0'
  const list = useLoad(() => (allowed ? api.daws() : Promise.resolve([])), [allowed])

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.daws.title}>
        <button className={btnPrimary} disabled={!allowed || list.loading} onClick={list.reload}><RefreshCw size={15} className={list.loading ? 'animate-spin' : ''} />{t.daws.detect}</button>
      </PageHeader>
      <Note>{allowed ? t.daws.note : `${t.settings.dawDetection}: ${t.common.no}`}</Note>
      <ErrorLine error={list.error} />
      <div className="grid gap-4 md:grid-cols-2">
        {list.data?.map((d) => (
          <Card key={d.id} className={d.supported_on_this_os ? '' : 'opacity-60'}>
            <div className="mb-3 flex items-center gap-2" data-daw={d.id}>
              <Dot on={d.connected} />
              <h2 className="flex-1 font-display text-lg font-semibold">{t.production.daws[d.id]}</h2>
              <Badge tone={FIRST.has(d.id) ? 'violet' : 'off'}>{FIRST.has(d.id) ? t.daws.first : t.daws.later}</Badge>
            </div>
            <dl className="grid grid-cols-[130px_1fr] gap-x-3 gap-y-1.5 text-sm">
              <dt className="text-muted">{t.daws.compatibility}</dt>
              <dd>{d.supported_on_this_os ? <Badge tone="ok">✓</Badge> : <Badge>{t.daws.notSupported}</Badge>}</dd>
              <dt className="text-muted">{t.daws.detection}</dt>
              <dd>{d.detected ? <Badge tone="ok">{t.daws.detected}</Badge> : <Badge>{t.daws.notDetected}</Badge>}</dd>
              <dt className="text-muted">{t.daws.version}</dt>
              <dd>{d.version ?? (d.detected ? <span className="text-muted">{t.daws.versionNa}</span> : '—')}</dd>
              {d.path && <><dt className="text-muted">{t.daws.path}</dt><dd className="truncate text-xs selectable" title={d.path}>{d.path}</dd></>}
              <dt className="text-muted">{t.daws.connection}</dt>
              <dd className="text-amber-200/90">{t.daws.notConnected}</dd>
              <dt className="text-muted">{t.daws.functions}</dt>
              <dd className="text-xs leading-relaxed">{t.dawInfo[d.id].route}</dd>
              <dt className="text-muted">{t.daws.limits}</dt>
              <dd className="text-xs leading-relaxed text-muted">{t.dawInfo[d.id].limits}</dd>
            </dl>
          </Card>
        ))}
      </div>
    </div>
  )
}
