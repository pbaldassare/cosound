import type { ReactNode } from 'react'
import { AudioLines, Bot, Cable, FolderInput, HardDrive, MessageSquare, Piano, Puzzle, Settings, User, TriangleAlert } from 'lucide-react'
import { useStore, type Page } from '../lib/store'
import { LanguageToggle, useT } from '../i18n'
import { Dot } from './ui'

const items: { page: Page; icon: typeof Bot }[] = [
  { page: 'chat', icon: MessageSquare },
  { page: 'production', icon: AudioLines },
  { page: 'sketch', icon: Piano },
  { page: 'library', icon: HardDrive },
  { page: 'imports', icon: FolderInput },
  { page: 'plugins', icon: Puzzle },
  { page: 'daws', icon: Cable },
  { page: 'agents', icon: Bot },
  { page: 'settings', icon: Settings },
  { page: 'account', icon: User },
]

export function Shell({ children }: { children: ReactNode }) {
  const { page, go, project, agent, assistantName, session } = useStore()
  const { t } = useT()
  const connected = !!agent?.connected && !agent.is_temporary

  return (
    <div className="flex h-full">
      <aside className="flex w-56 shrink-0 flex-col border-r border-line bg-panel/80">
        <div className="flex items-center gap-2.5 px-4 py-4 font-display text-base font-bold tracking-[0.18em]">
          <img src="/favicon.svg" alt="" className="h-7 w-7" />
          COSOUND
        </div>
        <nav className="flex-1 space-y-0.5 px-2" aria-label="COSOUND">
          {items.map(({ page: p, icon: Icon }) => (
            <button key={p} onClick={() => go(p)} aria-current={page === p ? 'page' : undefined} data-nav={p}
              className={`group relative flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition ${page === p ? 'bg-white/[0.07] text-white' : 'text-muted hover:bg-white/[0.04] hover:text-white'}`}>
              {page === p && <span className="absolute inset-y-1.5 left-0 w-0.5 rounded-full bg-gradient-to-b from-violet to-sky" />}
              <Icon size={17} className={page === p ? 'text-sky' : ''} />
              {t.nav[p]}
            </button>
          ))}
        </nav>
        <div className="border-t border-line p-3 text-xs text-muted">
          <div className="flex items-center gap-2"><Dot on={connected} />{connected ? t.shell.agentConnected : t.shell.agentNotConnected}</div>
          {session?.user.email && <div className="mt-1.5 truncate">{session.user.email}</div>}
        </div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-14 shrink-0 items-center justify-between gap-4 border-b border-line bg-ink/80 px-5 backdrop-blur">
          <div className="flex min-w-0 items-center gap-3 text-sm">
            <span className="font-display font-semibold text-gradient">{assistantName}</span>
            <span className="text-line">/</span>
            <button onClick={() => go('production')} className="truncate text-muted hover:text-white">
              {t.shell.currentProject}: <span className="text-white">{project?.name ?? t.common.noProject}</span>
            </button>
          </div>
          <LanguageToggle />
        </header>
        {agent?.is_temporary && (
          <div role="status" className="flex items-center gap-2 border-b border-amber-400/30 bg-amber-400/10 px-5 py-1.5 text-xs text-amber-200">
            <TriangleAlert size={14} /> {t.shell.tempBanner}
          </div>
        )}
        <main className="grid-bg min-h-0 flex-1 overflow-y-auto p-6">{children}</main>
      </div>
    </div>
  )
}
