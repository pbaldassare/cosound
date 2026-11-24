import { useStore } from '../lib/store'
import en from './en'
import it from './it'

export type Lang = 'it' | 'en'
const dicts = { en, it }

export const systemLang = (): Lang => (navigator.language?.toLowerCase().startsWith('it') ? 'it' : 'en')

/** Language lives in the local settings DB (key `language`), so it survives restarts. */
export function useT() {
  const { settings, setSetting } = useStore()
  const lang: Lang = settings.language === 'en' || settings.language === 'it' ? settings.language : systemLang()
  document.documentElement.lang = lang
  return { lang, t: dicts[lang], setLang: (l: Lang) => setSetting('language', l) }
}

/** Fill `{name}` placeholders. */
export const fill = (s: string, vars: Record<string, unknown>) => s.replace(/\{(\w+)\}/g, (_, k) => String(vars[k] ?? ''))

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
