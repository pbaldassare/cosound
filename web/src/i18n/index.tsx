import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import en, { type Dict } from './en'
import it from './it'

export type Lang = 'en' | 'it'
const dicts: Record<Lang, Dict> = { en, it }

const initial = (): Lang => {
  try {
    const saved = localStorage.getItem('lang')
    if (saved === 'en' || saved === 'it') return saved
  } catch { /* storage unavailable */ }
  return navigator.language?.toLowerCase().startsWith('it') ? 'it' : 'en'
}

const Ctx = createContext<{ lang: Lang; t: Dict; setLang: (l: Lang) => void }>({ lang: 'en', t: en, setLang: () => {} })

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLang] = useState<Lang>(initial)
  useEffect(() => {
    document.documentElement.lang = lang
    try { localStorage.setItem('lang', lang) } catch { /* ignore */ }
  }, [lang])
  return <Ctx.Provider value={{ lang, t: dicts[lang], setLang }}>{children}</Ctx.Provider>
}

export const useT = () => useContext(Ctx)

export function LanguageToggle() {
  const { lang, setLang, t } = useT()
  return (
    <div className="inline-flex rounded-full border border-line bg-panel p-0.5 font-mono text-xs" role="group" aria-label={t.nav.language}>
      {(['it', 'en'] as const).map((l) => (
        <button key={l} onClick={() => setLang(l)} aria-pressed={lang === l} lang={l}
          className={`rounded-full px-2.5 py-1 uppercase transition ${lang === l ? 'bg-white/10 text-white' : 'text-muted hover:text-white'}`}>
          {l}
        </button>
      ))}
    </div>
  )
}
