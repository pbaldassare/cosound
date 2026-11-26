import { useEffect, useState } from 'react'
import { RefreshCw } from 'lucide-react'
import { api, errText, type PluginOverview } from '../lib/api'
import { useStore } from '../lib/store'
import { fill, useT } from '../i18n'
import { Badge, ErrorLine, Note, PageHeader, btnPrimary } from '../components/ui'
import { Catalog, ChainCard, DawCapsCard, DesignStudio, Diagnostics, PresetSearch } from '../components/PluginIntelligence'

export default function Plugins() {
  const { settings } = useStore()
  const { t } = useT()
  const allowed = settings.perm_plugin_scan !== '0'
  const [ov, setOv] = useState<PluginOverview | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [info, setInfo] = useState<string | null>(null)
  const [version, setVersion] = useState(0)
  const loadOv = () => api.pluginsOverview().then(setOv, (e) => setError(errText(e)))
  const scan = async () => {
    setBusy(true)
    setError(null)
    try {
      const r = await api.pluginsScan()
      setInfo(fill(t.plugins.scanned, { n: r.plugins.new, m: r.plugins.missing, b: r.plugins.back, i: r.plugins.incompatible }) + (r.plugins.notes.length ? ` · ${r.plugins.notes.join(' · ')}` : ''))
      await loadOv()
      setVersion((v) => v + 1)
    } catch (e) { setError(errText(e)) } finally { setBusy(false) }
  }
  useEffect(() => {
    loadOv().then(() => api.pluginsOverview().then((o) => { if (o.products === 0 && allowed) scan() }))
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <PageHeader title={t.plugins.title}>
        <button className={btnPrimary} disabled={!allowed || busy} onClick={scan}><RefreshCw size={15} className={busy ? 'animate-spin' : ''} />{busy ? t.plugins.scanning : t.plugins.scan}</button>
      </PageHeader>
      <Note>{allowed ? t.plugins.note : t.plugins.noScan}</Note>
      <ErrorLine error={error} />
      {ov && (
        <div className="flex flex-wrap items-center gap-2 text-xs">
          {Object.entries(ov.by_kind).map(([k, n]) => <Badge key={k} tone={k === 'instrument' ? 'violet' : 'off'}>{t.plugins.kinds[k] ?? k} {n}</Badge>)}
          {Object.entries(ov.by_format).map(([f, n]) => <Badge key={f} tone="info">{f} {n}</Badge>)}
          {Object.values(ov.presets).reduce((a, b) => a + b, 0) > 0 && <Badge>{t.plugins.presets} {Object.values(ov.presets).reduce((a, b) => a + b, 0)}</Badge>}
          <Badge tone={ov.host_available ? 'ok' : 'warn'}>{t.plugins.host}: {ov.host_available ? t.plugins.hostOk : t.plugins.hostNo}</Badge>
          {info && <span className="text-muted">{info}</span>}
        </div>
      )}
      <Catalog ov={ov} version={version} />
      <DesignStudio ov={ov} />
      <div className="grid gap-5 lg:grid-cols-2">
        <ChainCard />
        <PresetSearch />
        <DawCapsCard />
        <Diagnostics ov={ov} />
      </div>
    </div>
  )
}
