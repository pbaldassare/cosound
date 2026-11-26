// The only bridge between the UI and the Rust core. Pages never touch engines directly.
import { invoke } from '@tauri-apps/api/core'

export type AppInfo = { version: string; os: string; arch: string; data_dir: string; temp_connector: boolean }
export type Settings = Record<string, string>

export type Volume = {
  id: string; label: string; kind: 'internal' | 'external' | 'network' | 'folder'
  disk_type: string | null; fs_type: string | null; mount_point: string | null; online: boolean
  total_bytes: number | null; available_bytes: number | null; root_count: number; file_count: number; last_seen_at: string | null; excluded: boolean
}
export type Root = {
  id: string; volume_id: string; volume_label: string; volume_kind: string; path: string; online: boolean
  scan_state: 'idle' | 'scanning' | 'done' | 'offline' | 'error'; file_count: number; last_scan_at: string | null; error: string | null
}
export type FileKind = 'audio' | 'midi' | 'project' | 'preset'
export type FileHit = { id: number; root_id: string; name: string; rel_path: string; kind: FileKind; size: number; path: string | null; volume_label: string }

type Est<T> = { value: T | null; source: string | null; confidence: number | null }
type Lbl = { tag: string; source: string; confidence: number }
export type LibFile = {
  id: number; name: string; kind: string; ext: string; format: string | null; size: number; status: 'present' | 'modified' | 'moved' | 'missing'
  path: string | null; rel_path: string; device: string; device_online: boolean; duration_s: number | null; sample_rate: number | null; channels: number | null; bit_depth: number | null
  lufs: number | null; peak_dbfs: number | null; bpm: Est<number>; key: Est<string>; is_loop: boolean | null; loop_bars: number | null; category: string | null
  categories: Lbl[]; types: Lbl[]; genres: Lbl[]; descriptors: Lbl[]; plugins: Lbl[]; analysis: string
  score?: number; why?: string[]; similarity?: number; fit?: { project_bpm: number; loop_bpm: number; relation: string; stretch_percent: number; key: string | null }
  details?: Record<string, unknown> | null; corrections?: { field: string; value: string | null }[]; moved_from?: string | null; missing_since?: string | null; analysis_error?: string | null
}
export type Understood = { categories: string[]; types: string[]; genres: string[]; descriptors: string[]; key: string | null; mode: string | null; bpm: number | null; bpm_range: [number, number] | null; max_duration_s: number | null; min_duration_s: number | null; bars: number | null; formats: string[]; kind: string | null; plugin: string | null; device: string | null; text: string[] }
export type LibQuery = { query?: string; understood: Understood; count: number; offline_results?: number; results: LibFile[] }
export type DupGroup = { method: string; files: { id: number; name: string; rel_path: string; device: string; path: string | null; size: number }[] }
export type Duplicates = { identical_files: DupGroup[]; same_audio: DupGroup[]; similar_not_duplicates: { similarity: number; files: DupGroup['files'] }[]; note: string }
export type LibOverview = {
  stats: { files: number; by_status: Record<string, number>; by_kind: Record<string, number>; by_category: Record<string, number>; analysis: Record<string, number>; offline_files: number }
  root_states: { root_id: string; scan_state: string; error: string | null; missing_files: number; pending_analysis: number }[]
  exclusions: { root_id: string; folder: string }[]
  service: { activity: string; paused_reason: string | null; queued_scans: number; analyzed_this_session: number }
  analysis: { pending: number; pending_offline: number; done: number; errors: number }
  formats: Record<string, string[]>
}

export type Project = {
  id: string; name: string; daw: string | null; bpm: number | null; musical_key: string | null; notes: string
  created_at: string; updated_at: string; opened_at: string | null; archived_at: string | null
}
export type Provenance = 'own' | 'suno' | 'udio' | 'other_ai' | 'unknown'
export type Role = 'track' | 'stem' | 'loop' | 'sample' | 'voice' | 'midi' | 'project' | 'preset'
export type Import = {
  id: string; project_id: string | null; path: string; name: string; kind: string; role: Role
  provenance: Provenance; size: number; sha256: string; imported_at: string; exists: boolean
}
export type OpEntry = { id: number; ts: string; project_id: string | null; op: string; risk: string; params: Record<string, unknown>; undoable: boolean; undone_at: string | null }

export type DawId = 'logic' | 'ableton' | 'fl' | 'protools' | 'cubase' | 'reaper'
export type DawFound = { id: DawId; supported_on_this_os: boolean; detected: boolean; path: string | null; version: string | null; connected: boolean }
export type PluginRow = { id: string; name: string; vendor: string | null; kind: string; available: boolean; categories: string[]; formats: string[]; parameters: number; presets: number; last_verified: string | null }
export type PluginOverview = { products: number; by_kind: Record<string, number>; by_format: Record<string, number>; by_status: Record<string, number>; by_category: Record<string, number>; presets: Record<string, number>; last_scan: string | null; host_available: boolean; host_formats: string[]; errors: { install_id: string | null; operation: string; error: string; at: string }[]; os: string; arch: string; taxonomy: { instrument: string[]; effect: string[] } }
export type PluginParam = { address: number; name: string; unit: string | null; min: number | null; max: number | null; default: number | null; writable: boolean; role: string | null }
export type PluginInstall = { id: string; product_id: string; format: string; name: string; vendor: string | null; version: string | null; path: string | null; scope: string; archs: string[]; au_code: string | null; registered: boolean | null; status: string; status_reason: string | null; details: { daw_compat?: { daw: string; name: string; basis: string; source: string }[]; declared_categories?: string[]; control_verified_at?: string }; last_verified: string; levels: Record<string, unknown>; parameters: PluginParam[] }
export type PluginPreset = { id: number; product_id: string; source: string; name: string; number: number | null; path: string | null; format: string | null; category: string | null; readable: boolean; plugin: string; details: { timbre?: { descriptors: Record<string, number>; centroid_hz: number; lufs: number | null } }; why?: string[]; score?: number; origin?: string }
export type PluginDetail = { product: { id: string; name: string; vendor: string | null; kind: string; available: boolean; first_seen: string; last_verified: string | null }; categories: { category: string; source: string; confidence: number }[]; installs: PluginInstall[]; knowledge: { documentation: { parameters?: { constant: string; id: number; doc?: string }[]; source: Record<string, string> } | null; synth_profile: { name: string; synthesis: string; source: string } | null; effect_facts: { category: string; what_it_does: string; source: string }[]; limits: string[] }; presets: PluginPreset[]; library_presets: LibFile[]; errors: { operation: string; error: string; at: string }[] }
export type DesignChange = { slot: number; unit: string; address: number; parameter: string; role: string; unit_of_measure: string; from: number; to: number; reasons: string[] }
export type Design = { id: string; title: string; request: string | null; status: string; changes: DesignChange[]; files: { path: string; kind: string; location: string; unit?: string }[]; spec: { instrument?: { name: string }; chain?: { name: string }[]; notes?: unknown[] } | null; instrument_choice?: { name: string; why: string[]; product_id: string }; alternatives?: { name: string; product_id: string; controllable: boolean; why: string[] }[]; effects_added?: string[]; automation?: string | null; not_available?: string[]; music?: { key: string; bpm: number; part: string } | null; created_at: string }
export type Measured = { descriptors: Record<string, number>; centroid_hz: number; lufs: number | null; peak_dbfs: number | null; true_peak_dbtp: number | null }
export type Audition = { before?: { file: string; measured: Measured }; after: { file: string; measured: Measured }; checks: { quality: string; requested: string; metric: string; before: number; after: number; as_requested: boolean }[]; warnings: string[] }
export type ChainStage = { category: string; purpose: string; purpose_it: string; include: boolean; why: string[]; settings: { role: string; value: number; unit: string; note?: string }[]; plugin: { install_id: string; name: string; hostable: boolean } | null; source: string }
export type ChainProposal = { goal: string; label: string; stages: ChainStage[]; order_issues: string[]; notes: string[]; adapted_to_material: boolean; status: string; check?: { rendered_stages: number; not_rendered: string[]; output: { peak_dbfs: number | null; true_peak_dbtp: number | null; lufs: number | null }; input: { peak_dbfs: number | null; lufs: number | null }; rms_change_db: number | null; risks: string[] } }
export type DawCaps = { daw: string; name: string; source: string; operations: { operation: string; status: 'available' | 'planned' | 'none'; route: string }[] }

export type MidiNote = { pitch: number; start: number; dur: number; vel: number }
export type PartId = 'chords' | 'bass' | 'melody' | 'drums'
export type Part = { id: PartId; name: string; channel: number; program: number; notes: MidiNote[] }
export type ChordView = { symbol: string; roman: string; root: string; quality: string; notes: string[]; pitch_classes: number[]; degree: number; function: 'tonic' | 'predominant' | 'dominant' | 'borrowed'; tension: number }
export type KeyView = { name: string; tonic: string; mode: string; scale: string[]; intervals: string[]; pitch_classes: number[]; signature: { count: number; accidental: 'sharps' | 'flats' | 'none' }; relative: string | null; parallel: string | null }
export type Sketch = {
  engine: string; genre: string | null; key: KeyView; bpm: number; bars: number; ppq: number; length_ticks: number; seed: number; swing: number
  styles: { chords: string; bass: string; drums: string; fill: boolean }
  progression: { chords: ChordView[]; final_cadence: string; loop_cadence: string; tension_curve: number[] }
  parts: Part[]; checks: Record<string, Record<string, boolean>>; all_checks_passed: boolean
}
export type GenreOption = { id: string; label: string; bpm: number; key: string; progression: string; bass_style: string; drum_style: string; chord_rhythm: string; swing: number }
export type MusicOptions = { genres: GenreOption[]; modes: { id: string; default_progression: string }[]; bass_styles: string[]; drum_styles: string[]; chord_rhythms: string[] }
export type SketchInput = Record<string, unknown>

export type PortHealth = { port_id: string; is_temporary: boolean; connected: boolean }
export type AilError = { code: string; message: string }
export type TaskStatus = 'PENDING' | 'RUNNING' | 'WAITING_APPROVAL' | 'COMPLETED' | 'FAILED' | 'CANCELLED'
export type ErrorInfo = { code: string; message: string; retryable: boolean }
export type FileRef = { path: string; kind: string; bytes: number; sha256: string }
export type ApprovalRequest = { approval_id: string; task_id: string; agent: string; operation: string; risk: string; summary: string; approvable_by: string; requested_at_ms: number }
export type TaskResult = {
  contract_version: string; request_id: string; task_id: string; trace_id: string | null; project_id: string | null
  agent: string; agent_version: string | null; operation: string; requested_by: string | null; status: TaskStatus
  result: unknown; basis: string | null; files: FileRef[]; tools_used: string[]; errors: ErrorInfo[]; limitations: string[]
  needs: { agent: string; operation: string; why: string }[]; proposed_daw_ops: unknown[]; approval: ApprovalRequest | null
  created_at_ms: number; started_at_ms: number | null; finished_at_ms: number | null
}
export type AgentEvent =
  | { kind: 'assistant_text'; text: string }
  | { kind: 'task_result'; result: TaskResult }
  | { kind: 'error'; error: AilError }
type Bi = { it: string; en: string }
type BiList = { it: string[]; en: string[] }
export type OpSpec = { name: string; status: 'available' | 'planned'; risk: 'read' | 'low' | 'medium' | 'high'; requires_approval: boolean; summary: Bi; output: string; tools: string[]; permissions: string[]; basis?: string; requires?: string }
export type AgentInfo = {
  id: string; name: string; version: string; role: Bi; competencies: BiList; operations: OpSpec[]; tools: string[]; permissions: string[]; limits: BiList
  input_formats: string[]; output_formats: string[]; collaborates_with: string[]; enabled: boolean; implemented: boolean; state: 'active' | 'partial' | 'planned'
}
export type ToolInfo = { id: string; name: string; layer: number; status: 'available' | 'planned'; module: string | null; description: Bi; requires?: string; used_by: string[] }
export type PermissionInfo = { id: string; setting: string | null; default: boolean; grantable: boolean; label: Bi; description: Bi; granted: boolean }
export type Registry = { contract_version: string; agents: AgentInfo[]; tools: ToolInfo[]; permissions: PermissionInfo[] }
export type Fact = { id: string; category: string; key: string; value: Record<string, unknown>; origin: string; scope: 'standard' | 'custom'; confidence: number; source_ref: string | null; created_at: string; updated_at: string }

export const api = {
  appInfo: () => invoke<AppInfo>('app_info'),
  settings: () => invoke<Settings>('settings_get'),
  setSetting: (key: string, value: string) => invoke<void>('settings_set', { key, value }),
  ops: (projectId?: string | null, limit?: number) => invoke<OpEntry[]>('oplog_list', { projectId, limit }),
  undo: (id: number) => invoke<void>('oplog_undo', { id }),

  volumes: () => invoke<Volume[]>('library_volumes'),
  roots: () => invoke<Root[]>('library_roots'),
  addRoot: (path: string) => invoke<string>('library_add_root', { path }),
  removeRoot: (id: string) => invoke<void>('library_remove_root', { id }),
  scan: (id: string) => invoke<void>('library_scan', { id }),
  search: (query: string, kind?: FileKind | null) => invoke<FileHit[]>('library_search', { query, kind }),
  libQuery: (text: string, kind?: string | null) => invoke<LibQuery>('library_query', { text, kind: kind || null, limit: 100 }),
  libDetail: (id: number) => invoke<LibFile>('library_detail', { id }),
  libSimilar: (id: number) => invoke<{ note: string; results: LibFile[] }>('library_similar', { id }),
  libLoops: (bpm: number, key: string, category: string) => invoke<{ note: string; results: LibFile[] }>('library_loops', { bpm, key, category }),
  libDuplicates: (similar: boolean) => invoke<Duplicates>('library_duplicates', { similar }),
  libOverview: () => invoke<LibOverview>('library_overview'),
  libCorrect: (id: number, field: 'bpm' | 'key' | 'add' | 'remove', value: string) => invoke<LibFile>('library_correct', { id, field, value }),
  libExclude: (rootId: string, folder: string, excluded: boolean) => invoke<void>('library_exclude', { rootId, folder, excluded }),
  libExcludeDevice: (volumeId: string, excluded: boolean) => invoke<void>('library_exclude_device', { volumeId, excluded }),
  libCancel: () => invoke<void>('library_cancel'),

  projects: (includeArchived: boolean) => invoke<Project[]>('projects_list', { includeArchived }),
  createProject: (p: { name: string; daw: string | null; bpm: number | null; musicalKey: string | null }) => invoke<Project>('project_create', p),
  openProject: (id: string) => invoke<Project>('project_open', { id }),
  saveProject: (id: string, fields: { daw: string | null; bpm: number | null; musical_key: string | null; notes: string }) => invoke<Project>('project_save', { id, fields }),
  renameProject: (id: string, name: string) => invoke<Project>('project_rename', { id, name }),
  duplicateProject: (id: string, suffix: string) => invoke<Project>('project_duplicate', { id, suffix }),
  archiveProject: (id: string, archived: boolean) => invoke<Project>('project_archive', { id, archived }),
  projectRoots: (id: string) => invoke<string[]>('project_roots', { id }),
  linkRoot: (id: string, rootId: string, linked: boolean) => invoke<void>('project_link_root', { id, rootId, linked }),

  importFiles: (projectId: string | null, paths: string[], provenance: Provenance, role: Role | null) => invoke<string[]>('imports_add', { projectId, paths, provenance, role }),
  imports: (projectId?: string | null) => invoke<Import[]>('imports_list', { projectId }),
  removeImport: (id: string) => invoke<void>('imports_remove', { id }),
  saveTake: (projectId: string | null, bytes: number[], ext: string, mode: string) => invoke<string>('voice_save_take', { projectId, bytes, ext, mode }),

  daws: () => invoke<DawFound[]>('daw_detect'),
  pluginsScan: () => invoke<{ plugins: { new: number; missing: number; back: number; incompatible: number; errors: number; notes: string[] }; preset_folders: { stored: number } }>('plugins_scan'),
  pluginsOverview: () => invoke<PluginOverview>('plugins_overview'),
  pluginsList: (text: string, kind: string, category: string) => invoke<PluginRow[]>('plugins_list', { text, kind, category }),
  pluginsDetail: (id: string) => invoke<PluginDetail>('plugins_detail', { id }),
  pluginsDiscover: (installId: string) => invoke<unknown>('plugins_discover', { installId }),
  pluginsPresetSearch: (text: string, productId?: string) => invoke<{ results: PluginPreset[]; understood: Record<string, string[]> }>('plugins_presets_search', { text, productId: productId ?? null }),
  pluginsPresetAnalyze: (id: number) => invoke<{ timbre: { descriptors: Record<string, number> } }>('plugins_preset_analyze', { id }),
  pluginsCompare: (ids: string[]) => invoke<{ compared: { product: { id: string; name: string; vendor: string | null }; categories: { category: string }[]; formats: string[]; versions: (string | null)[]; sound_roles: string[]; presets: number }[] }>('plugins_compare', { ids }),
  pluginsChain: (goal: string, file: string, bpm: number | null, check: boolean) => invoke<ChainProposal>('plugins_chain', { goal, file: file || null, bpm, check }),
  pluginsDaw: (daw: string) => invoke<DawCaps>('plugins_daw', { daw }),
  designCreate: (text: string, role: string, genre: string, key: string, bpm: number | null, productId: string) => invoke<Design>('plugins_design_create', { text, role, genre, key, bpm, productId }),
  designModify: (id: string, text: string) => invoke<Design>('plugins_design_modify', { id, text }),
  designAudition: (id: string) => invoke<Audition>('plugins_design_audition', { id }),
  designSave: (id: string, name: string) => invoke<{ files: { path: string; kind: string }[] }>('plugins_design_save', { id, name }),
  designInstall: (id: string) => invoke<{ installed: { path: string }[] }>('plugins_design_install', { id }),
  designUninstall: (id: string) => invoke<{ removed: string[] }>('plugins_design_uninstall', { id }),
  designs: () => invoke<Design[]>('plugins_designs'),
  designGet: (id: string) => invoke<Design>('plugins_design_get', { id }),
  designPlan: (id: string, daw: string) => invoke<{ steps: { step: string; by: string; status: string; fallback?: string }[] }>('plugins_design_plan', { id, daw }),
  pluginsRemember: (role: string, productId: string) => invoke<unknown>('plugins_remember', { role, productId }),

  musicOptions: () => invoke<MusicOptions>('music_options'),
  musicSketch: (input: SketchInput) => invoke<Sketch>('music_sketch', { input }),
  musicExport: (input: SketchInput) => invoke<{ path: string; bytes: number; import_id: string | null }>('music_export_midi', { input }),

  ailStatus: () => invoke<PortHealth>('ail_status'),
  ailSend: (text: string) => invoke<AgentEvent[]>('ail_send', { text }),
  registry: () => invoke<Registry>('agents_registry'),
  agentTasks: (limit?: number) => invoke<TaskResult[]>('agents_tasks', { limit }),
  approve: (approvalId: string, approve: boolean) => invoke<TaskResult>('agents_approval_respond', { approvalId, approve }),
  cancelTask: (taskId: string) => invoke<TaskResult>('agents_cancel', { taskId }),
  setAgentEnabled: (agent: string, enabled: boolean) => invoke<void>('agents_set_enabled', { agent, enabled }),
  memory: (category?: string) => invoke<Fact[]>('memory_list', { category }),
  memoryDelete: (id: string) => invoke<void>('memory_delete', { id }),
  memoryClear: (category: string) => invoke<number>('memory_clear', { category }),
}

export const fmtBytes = (n: number | null | undefined) => {
  if (n == null) return '—'
  const u = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  while (n >= 1024 && i < u.length - 1) { n /= 1024; i++ }
  return `${n.toFixed(i > 1 ? 1 : 0)} ${u[i]}`
}

/** SQLite datetime('now') is UTC without a zone marker. */
export const fmtDate = (s: string | null | undefined, lang: string) =>
  s ? new Date(s.includes('T') ? s : s.replace(' ', 'T') + 'Z').toLocaleString(lang, { dateStyle: 'medium', timeStyle: 'short' }) : '—'

export const errText = (e: unknown) => (typeof e === 'string' ? e : e instanceof Error ? e.message : JSON.stringify(e))
