import { Undo2 } from 'lucide-react'
import { api, errText, fmtDate, type OpEntry } from '../lib/api'
import { fill, useT } from '../i18n'
import { Badge, Empty, btnIcon } from './ui'

/** Operation register / project history with undo where the core supports it. */
export function OpList({ ops, empty, onChange }: { ops: OpEntry[]; empty: string; onChange: () => void }) {
  const { t, lang } = useT()
  if (ops.length === 0) return <Empty>{empty}</Empty>
  const undo = async (id: number) => {
    try { await api.undo(id) } catch (e) { alert(errText(e)) }
    onChange()
  }
  return (
    <ul className="divide-y divide-line/60 text-sm">
      {ops.map((o) => (
        <li key={o.id} className="flex items-center gap-3 py-2">
          <span className="w-40 shrink-0 text-xs text-muted">{fmtDate(o.ts, lang)}</span>
          <span className={`min-w-0 flex-1 truncate selectable ${o.undone_at ? 'text-muted line-through' : ''}`} title={JSON.stringify(o.params)}>
            {fill(t.ops[o.op] ?? o.op, o.params)}
          </span>
          {o.undone_at && <Badge>{t.common.undone}</Badge>}
          <Badge tone={o.risk === 'high' ? 'err' : o.risk === 'medium' ? 'warn' : 'off'}>{o.risk}</Badge>
          {o.undoable && !o.undone_at && (
            <button className={btnIcon} onClick={() => undo(o.id)} title={t.common.undo} aria-label={t.common.undo}><Undo2 size={15} /></button>
          )}
        </li>
      ))}
    </ul>
  )
}
