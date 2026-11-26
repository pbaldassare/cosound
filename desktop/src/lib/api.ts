// The only bridge between the UI and the Rust core. Pages never touch engines directly.
import { invoke } from '@tauri-apps/api/core'

export type AppInfo = { version: string; os: string; arch: string; data_dir: string; temp_connector: boolean }
export type Settings = Record<string, string>

export type Volume = {
  id: string; label: string; kind: 'internal' | 'external' | 'network' | 'folder'
  disk_type: string | null; fs_type: string | null; mount_point: string | null; online: boolean
  total_bytes: number | null; available_bytes: number | null; root_count: number; file_count: number; last_seen_at: string | null
}
export type Root = {
  id: string; volume_id: string; volume_label: string; volume_kind: string; path: string; online: boolean
  scan_state: 'idle' | 'scanning' | 'done' | 'offline' | 'error'; file_count: number; last_scan_at: string | null; error: string | null
}
export type FileKind = 'audio' | 'midi' | 'project' | 'preset'
export type FileHit = { id: number; root_id: string; name: string; rel_path: string; kind: FileKind; size: number; path: string | null; volume_label: string }

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
export type Plugin = { name: string; format: 'VST3' | 'AU' | 'CLAP' | 'VST2'; kind: 'instrument' | 'effect' | 'unknown'; vendor: string | null; version: string | null; scope: 'system' | 'user'; path: string }

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
  plugins: () => invoke<Plugin[]>('plugins_scan'),

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
