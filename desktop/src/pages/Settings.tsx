import { useState, type ChangeEvent } from 'react'
import { FolderOpen, Trash2 } from 'lucide-react'
import { openPath } from '@tauri-apps/plugin-opener'
import { api, errText } from '../lib/api'
import { useStore } from '../lib/store'
import { supabase } from '../lib/supabase'
import { LanguageToggle, useT } from '../i18n'
import { OpList } from '../components/OpList'
import { Card, ErrorLine, PageHeader, Toggle, btnGhost, btnIcon, btnPrimary, useLoad } from '../components/ui'

/** Save the assistant name locally (master copy) and, if the user opted in, in the online profile. */
export async function saveAssistantName(name: string, setSetting: (k: string, v: string) => Promise<void>, sync: boolean) {
  const clean = name.trim().slice(0, 40)
  if (!clean) throw new Error('empty name')
  await setSetting('assistant_name', clean)
  if (sync && supabase) {
    const { data } = await supabase.auth.getUser()
    if (data.user) {
      const { error } = await supabase.from('profiles').update({ assistant_name: clean }).eq('id', data.user.id)
      if (error) throw error
    }
  }
}

/** Style, detail, interaction mode and reply language of the assistant (Prompt Engine profile). */
function AssistantStyle() {
  const { t } = useT()
  const prof = useLoad(() => api.assistantProfile(), [])
  const model = useLoad(() => api.promptModelConfig(), [])
  const [error, setError] = useState<string | null>(null)
  const set = (field: string) => (e: ChangeEvent<HTMLSelectElement>) =>
    api.assistantProfileSet(field, e.target.value).then(prof.reload, (x) => setError(errText(x)))
  const p = prof.data
  if (!p) return null
  const sel = (label: string, field: string, value: string, opts: Record<string, string>) => (
    <label className="block text-sm font-medium">{label}
      <select className="field mt-1" value={value} onChange={set(field)}>{Object.entries(opts).map(([k, v]) => <option key={k} value={k}>{v}</option>)}</select>
    </label>
  )
  return (
    <div className="mt-3 space-y-3">
      <div className="grid gap-3 sm:grid-cols-2">
        {sel(t.settings.tone, 'style', p.style, t.settings.tones)}
        {sel(t.settings.detail, 'detail', p.detail, t.settings.details)}
        {sel(t.settings.mode, 'mode', p.mode, t.settings.modes)}
        {sel(t.settings.replyLanguage, 'language', p.language, t.settings.replyLanguages)}
      </div>
      <p className="text-xs text-muted">{t.settings.toneHelp}</p>
      <ErrorLine error={error} />
      <div className="rounded-lg border border-line p-3 text-xs">
        <div className="mb-1 font-medium text-white">{t.settings.model}</div>
        <p className="text-muted">{t.settings.modelNone}</p>
        {model.data && model.data.config.provider !== 'none' && <p className="mt-1 text-muted">{model.data.config.provider} · {model.data.config.model} · {model.data.usage.requests}/{model.data.config.max_requests_per_day}</p>}
      </div>
    </div>
  )
}

export default function SettingsPage() {
  const { settings, setSetting, info, assistantName, go } = useStore()
  const { t } = useT()
  const [name, setName] = useState(settings.assistant_name ?? '')
  const [msg, setMsg] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const roots = useLoad(() => api.roots(), [])
  const log = useLoad(() => api.ops(null, 300), [])
  const flag = (k: string, dflt = true) => (settings[k] === undefined ? dflt : settings[k] === '1')
  const setFlag = (k: string) => (v: boolean) => setSetting(k, v ? '1' : '0').then(log.reload)
  const sync = flag('cloud_sync_assistant_name', false)

  const saveName = async () => {
    setError(null); setMsg(null)
    try { await saveAssistantName(name, setSetting, sync); setMsg(t.common.saved); log.reload() } catch (e) { setError(errText(e)) }
  }

  return (
    <div className="mx-auto max-w-4xl space-y-5">
      <PageHeader title={t.settings.title} />

      <Card title={t.settings.assistant}>
        <label className="text-sm font-medium" htmlFor="aname">{t.settings.assistantName}</label>
        <p className="mb-2 text-xs text-muted">{t.settings.assistantNameHelp}</p>
        <div className="flex gap-2">
          <input id="aname" className="field" maxLength={40} value={name} placeholder={assistantName} onChange={(e) => setName(e.target.value)} />
          <button className={btnPrimary} onClick={saveName} disabled={!name.trim()}>{t.common.save}</button>
        </div>
        {msg && <p className="mt-1 text-xs text-emerald-300">{msg}</p>}
        <ErrorLine error={error} />
        <Toggle label={t.settings.syncName} checked={sync} onChange={setFlag('cloud_sync_assistant_name')} disabled={!supabase} />
        <AssistantStyle />
      </Card>

      <Card title={t.settings.language}><LanguageToggle /></Card>

      <Card title={t.settings.permissions}>
        <div className="divide-y divide-line/60">
          <div className="py-2">
            <div className="text-sm font-medium">{t.settings.folders}</div>
            <p className="mb-1 text-xs text-muted">{t.settings.foldersHelp}</p>
            <ul className="space-y-1">
              {roots.data?.map((r) => (
                <li key={r.id} className="flex items-center gap-2 text-sm">
                  <span className="min-w-0 flex-1 truncate selectable">{r.path}</span>
                  <button className={btnIcon} title={t.common.remove} aria-label={t.common.remove}
                    onClick={async () => { if (confirm(t.library.revokeConfirm)) { await api.removeRoot(r.id); roots.reload(); log.reload() } }}><Trash2 size={14} /></button>
                </li>
              ))}
            </ul>
            <button className={btnGhost + ' mt-2 !py-1 !text-xs'} onClick={() => go('library')}>{t.nav.library}</button>
          </div>
          <Toggle label={t.settings.microphone} help={t.settings.micHelp} checked={flag('perm_microphone', false)} onChange={setFlag('perm_microphone')} />
          <Toggle label={t.settings.externalDevices} help={t.settings.externalHelp} checked={flag('perm_external_devices')} onChange={setFlag('perm_external_devices')} />
          <Toggle label={t.settings.dawDetection} help={t.settings.dawHelp} checked={flag('perm_daw_detection')} onChange={setFlag('perm_daw_detection')} />
          <Toggle label={t.settings.pluginScan} help={t.settings.pluginHelp} checked={flag('perm_plugin_scan')} onChange={setFlag('perm_plugin_scan')} />
          <Toggle label={t.settings.pluginHost} help={t.settings.pluginHostHelp} checked={flag('perm_plugin_host')} onChange={setFlag('perm_plugin_host')} />
          <Toggle label={t.settings.musicMemory} help={t.settings.musicMemoryHelp} checked={flag('perm_music_memory', false)} onChange={setFlag('perm_music_memory')} />
          <Toggle label={t.settings.presetWrite} help={t.settings.presetWriteHelp} checked={flag('perm_preset_write', false)} onChange={setFlag('perm_preset_write')} />
          <Toggle label={t.settings.workflowMemory} help={t.settings.workflowHelp} checked={flag('perm_workflow_memory', false)} onChange={setFlag('perm_workflow_memory')} />
        </div>
      </Card>

      <Card title={t.settings.account}><button className={btnGhost} onClick={() => go('account')}>{t.nav.account}</button></Card>

      <Card title={t.settings.privacy}>
        <p className="text-sm text-muted">{t.settings.privacyBody}</p>
        {info && (
          <div className="mt-3 flex items-center gap-2 text-xs">
            <span className="text-muted">{t.settings.dataFolder}:</span>
            <span className="min-w-0 flex-1 truncate font-mono selectable">{info.data_dir}</span>
            <button className={btnIcon} aria-label={t.common.open} title={t.common.open} onClick={() => openPath(info.data_dir)}><FolderOpen size={14} /></button>
          </div>
        )}
      </Card>

      <Card title={t.settings.updates}>
        <p className="text-sm">{t.settings.version}: <span className="font-mono">{info?.version}</span> · {info?.os} {info?.arch}</p>
        <p className="mt-1 text-xs text-muted">{t.settings.updatesBody}</p>
      </Card>

      <Card title={t.settings.log} actions={<button className={btnGhost + ' !py-1 !text-xs'} onClick={() => setSetting('onboarding_done', '0')}>{t.settings.reopenSetup}</button>}>
        <div className="max-h-96 overflow-y-auto"><OpList ops={log.data ?? []} empty={t.settings.logEmpty} onChange={log.reload} /></div>
      </Card>
    </div>
  )
}
