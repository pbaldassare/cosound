import { useCallback, useEffect, useState, type InputHTMLAttributes, type ReactNode } from 'react'
import { Eye, EyeOff } from 'lucide-react'
import { errText } from '../lib/api'

export const btnPrimary =
  'inline-flex items-center justify-center gap-2 rounded-full bg-gradient-to-r from-violet via-electric to-sky px-4 py-2 text-sm font-semibold text-white shadow-[0_0_24px_-8px_rgb(168_85_247/0.8)] transition hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-50'
export const btnGhost =
  'inline-flex items-center justify-center gap-2 rounded-full border border-line bg-white/5 px-4 py-2 text-sm font-medium text-white transition hover:border-violet/60 hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50'
export const btnIcon = 'inline-flex h-8 w-8 items-center justify-center rounded-lg text-muted transition hover:bg-white/10 hover:text-white disabled:opacity-40'

export function PageHeader({ title, children }: { title: ReactNode; children?: ReactNode }) {
  return (
    <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
      <h1 className="font-display text-2xl font-bold tracking-tight">{title}</h1>
      <div className="flex flex-wrap items-center gap-2">{children}</div>
    </div>
  )
}

export function Card({ title, actions, children, className = '' }: { title?: ReactNode; actions?: ReactNode; children: ReactNode; className?: string }) {
  return (
    <section className={`card fade-in p-4 ${className}`}>
      {(title || actions) && (
        <div className="mb-3 flex items-center justify-between gap-2">
          {title && <h2 className="font-mono text-[11px] uppercase tracking-[0.22em] text-sky">{title}</h2>}
          {actions}
        </div>
      )}
      {children}
    </section>
  )
}

const tones = {
  ok: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-300',
  warn: 'border-amber-400/30 bg-amber-400/10 text-amber-300',
  off: 'border-line bg-white/5 text-muted',
  info: 'border-sky/30 bg-sky/10 text-sky',
  violet: 'border-violet/30 bg-violet/10 text-violet',
  err: 'border-rose-400/30 bg-rose-400/10 text-rose-300',
}
export function Badge({ tone = 'off', children }: { tone?: keyof typeof tones; children: ReactNode }) {
  return <span className={`inline-flex items-center gap-1 whitespace-nowrap rounded-full border px-2 py-0.5 text-[11px] font-medium ${tones[tone]}`}>{children}</span>
}

export function Dot({ on }: { on: boolean }) {
  return <span className={`inline-block h-2 w-2 rounded-full ${on ? 'bg-emerald-400 shadow-[0_0_8px] shadow-emerald-400' : 'bg-zinc-600'}`} />
}

export function Toggle({ checked, onChange, label, help, disabled }: { checked: boolean; onChange: (v: boolean) => void; label: string; help?: string; disabled?: boolean }) {
  return (
    <label className={`flex items-start justify-between gap-4 py-2 ${disabled ? 'opacity-50' : 'cursor-pointer'}`}>
      <span>
        <span className="block text-sm font-medium">{label}</span>
        {help && <span className="mt-0.5 block text-xs text-muted">{help}</span>}
      </span>
      <button type="button" role="switch" aria-checked={checked} aria-label={label} disabled={disabled} onClick={() => onChange(!checked)}
        className={`relative mt-0.5 h-5 w-9 shrink-0 rounded-full transition ${checked ? 'bg-violet' : 'bg-zinc-700'}`}>
        <span className={`absolute top-0.5 h-4 w-4 rounded-full bg-white transition-all ${checked ? 'left-[18px]' : 'left-0.5'}`} />
      </button>
    </label>
  )
}

export function Empty({ children }: { children: ReactNode }) {
  return <p className="rounded-lg border border-dashed border-line px-4 py-6 text-center text-sm text-muted">{children}</p>
}

export function Note({ children }: { children: ReactNode }) {
  return <p className="rounded-lg border border-line bg-white/[0.03] px-3 py-2 text-xs leading-relaxed text-muted">{children}</p>
}

export function ErrorLine({ error }: { error: string | null }) {
  return error ? <p role="alert" className="text-sm text-rose-300">{error}</p> : null
}

/** Load data from the core; `reload()` after mutations. */
export function useLoad<T>(fn: () => Promise<T>, deps: unknown[] = []) {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const reload = useCallback(() => {
    setLoading(true)
    return fn().then((d) => { setData(d); setError(null) }, (e) => setError(errText(e))).finally(() => setLoading(false))
  }, deps)
  useEffect(() => { reload() }, [reload])
  return { data, error, loading, reload, setData }
}

/** Password field with an eye button to show or hide what is typed. */
export function PasswordInput({ showLabel, hideLabel, ...props }: { showLabel: string; hideLabel: string } & InputHTMLAttributes<HTMLInputElement>) {
  const [shown, setShown] = useState(false)
  return (
    <span className="relative block">
      <input className="field pr-11" {...props} type={shown ? 'text' : 'password'} />
      <button type="button" onClick={() => setShown((v) => !v)} aria-pressed={shown} aria-label={shown ? hideLabel : showLabel} title={shown ? hideLabel : showLabel}
        className="absolute inset-y-0 right-0 flex w-10 items-center justify-center text-muted transition hover:text-white">
        {shown ? <EyeOff size={16} /> : <Eye size={16} />}
      </button>
    </span>
  )
}
