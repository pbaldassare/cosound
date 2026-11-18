import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Apple, AudioWaveform, Check, Disc3, Drum, FolderSearch, Mic, Monitor, SlidersHorizontal, Sparkles, Waves } from 'lucide-react'
import { supabase, formatPrice, type Daw, type Plan, type Release } from '../lib/supabase'
import { useQuery } from '../lib/useQuery'
import { useAuth } from '../lib/auth'
import { useT } from '../i18n'
import { SectionHeading, btnGhost, btnPrimary } from './Layout'

// Same order as t.features.items
const featureIcons = [FolderSearch, Sparkles, Drum, Waves, SlidersHorizontal, Disc3, Mic]

export function FeaturesSection() {
  const { t } = useT()
  return (
    <section id="features" className="scroll-mt-20 px-4 py-24 sm:px-6">
      <div className="mx-auto max-w-7xl">
        <SectionHeading eyebrow={t.features.eyebrow} title={<>{t.features.titleA} <span className="text-gradient">{t.features.titleB}</span></>}>
          {t.features.text}
        </SectionHeading>
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {t.features.items.map(({ title, text }, i) => {
            const Icon = featureIcons[i]
            return (
              <article key={title} className="card group p-6 transition hover:border-violet/50">
                <div className="mb-5 grid h-12 w-12 place-items-center rounded-xl bg-gradient-to-br from-violet/25 to-sky/20 text-sky ring-1 ring-white/10 transition group-hover:scale-105">
                  <Icon size={22} />
                </div>
                <h3 className="font-display text-lg font-semibold">{title}</h3>
                <p className="mt-2 text-sm leading-relaxed text-muted">{text}</p>
              </article>
            )
          })}
        </div>
      </div>
    </section>
  )
}

const statusCls: Record<Daw['status'], string> = {
  available: 'bg-emerald-500/15 text-emerald-300 ring-emerald-400/30',
  in_development: 'bg-violet/15 text-violet ring-violet/40',
  planned: 'bg-white/5 text-muted ring-line',
}

export function DawSection() {
  const { t, lang } = useT()
  const { data } = useQuery(() => supabase.from('daws').select('*').order('sort'))
  const daws = (data ?? []) as Daw[]
  return (
    <section id="daws" className="scroll-mt-20 border-y border-line bg-panel/60 px-4 py-24 sm:px-6">
      <div className="mx-auto max-w-7xl">
        <SectionHeading eyebrow={t.daws.eyebrow} title={t.daws.title}>{t.daws.text}</SectionHeading>
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {daws.map((d) => (
            <article key={d.slug} className="card flex flex-col p-6">
              <div className="mb-5 flex items-start justify-between gap-2">
                <div className="grid h-12 w-12 place-items-center rounded-xl bg-panel-2 font-display text-lg font-bold ring-1 ring-line" aria-hidden>
                  {d.name.split(' ').map((w) => w[0]).join('')}
                </div>
                <span className={`rounded-full px-2.5 py-1 text-[11px] font-medium ring-1 ${statusCls[d.status]}`}>{t.daws.status[d.status]}</span>
              </div>
              <h3 className="font-display text-lg font-semibold">{d.name}</h3>
              <p className="font-mono text-[11px] uppercase tracking-widest text-muted">{d.vendor}</p>
              <p className="mt-3 text-sm leading-relaxed text-muted">{(lang === 'it' && d.description_it) || d.description}</p>
            </article>
          ))}
        </div>
        <p className="mt-8 text-center text-sm text-muted">{t.daws.more}</p>
      </div>
    </section>
  )
}

export function PricingSection() {
  const { session } = useAuth()
  const { t, lang } = useT()
  const [yearly, setYearly] = useState(false)
  const { data, loading } = useQuery(() => supabase.from('plans').select('*').order('sort'))
  const plans = (data ?? []) as Plan[]

  return (
    <section id="pricing" className="scroll-mt-20 px-4 py-24 sm:px-6">
      <div className="mx-auto max-w-7xl">
        <SectionHeading eyebrow={t.pricing.eyebrow} title={t.pricing.title}>{t.pricing.text}</SectionHeading>
        <div className="mb-10 flex justify-center">
          <div className="inline-flex rounded-full border border-line bg-panel p-1 text-sm" role="group" aria-label={t.pricing.period}>
            {[false, true].map((y) => (
              <button key={String(y)} onClick={() => setYearly(y)} aria-pressed={yearly === y}
                className={`rounded-full px-5 py-2 transition ${yearly === y ? 'bg-white/10 text-white' : 'text-muted'}`}>
                {y ? t.pricing.yearly : t.pricing.monthly}
              </button>
            ))}
          </div>
        </div>
        {loading && <p className="text-center text-muted">{t.pricing.loading}</p>}
        <div className="grid gap-6 lg:grid-cols-3">
          {plans.map((p) => {
            const highlight = p.code === 'pro'
            const price = formatPrice(yearly ? p.price_yearly_cents : p.price_monthly_cents, p.currency, t.locale)
            const features = lang === 'it' && p.features_it.length ? p.features_it : p.features
            return (
              <article key={p.code} className={`card relative flex flex-col p-8 ${highlight ? 'border-violet/60 shadow-[0_0_60px_-20px_rgb(168_85_247/0.7)]' : ''}`}>
                {highlight && <span className="absolute -top-3 left-8 rounded-full bg-gradient-to-r from-violet to-electric px-3 py-1 text-[11px] font-semibold">{t.pricing.highlight}</span>}
                <p className="font-mono text-xs uppercase tracking-[0.3em] text-sky">{p.tagline}</p>
                <h3 className="mt-2 font-display text-2xl font-bold">{p.name}</h3>
                <p className="mt-5 font-display text-3xl font-bold">
                  {price ?? <span className="text-lg font-medium text-muted">{t.pricing.soon}</span>}
                  {price && <span className="text-base font-medium text-muted"> / {yearly ? t.pricing.year : t.pricing.month}</span>}
                </p>
                <ul className="mt-6 flex-1 space-y-3 text-sm">
                  {features.map((f) => (
                    <li key={f} className="flex gap-3"><Check size={18} className="shrink-0 text-sky" />{f}</li>
                  ))}
                </ul>
                {/* Stripe checkout plugs in here once prices and the checkout function exist (docs/SETUP.md). Nothing is charged until then. */}
                {session ? (
                  <button className={`${btnGhost} mt-8`} disabled>{t.pricing.openSoon}</button>
                ) : (
                  <Link to="/signup" className={`${highlight ? btnPrimary : btnGhost} mt-8`}>{t.pricing.create}</Link>
                )}
              </article>
            )
          })}
        </div>
      </div>
    </section>
  )
}

export function DownloadSection() {
  const { session } = useAuth()
  const { t } = useT()
  const { data } = useQuery(() => supabase.from('releases').select('*').eq('published', true).order('released_at', { ascending: false }))
  const releases = (data ?? []) as Release[]
  const platforms = [
    { key: 'macos', label: t.download.mac, icon: Apple },
    { key: 'windows', label: t.download.win, icon: Monitor },
  ] as const

  return (
    <section id="download" className="scroll-mt-20 px-4 py-24 sm:px-6">
      <div className="mx-auto max-w-5xl">
        <SectionHeading eyebrow={t.download.eyebrow} title={t.download.title}>{t.download.text}</SectionHeading>
        <div className="grid gap-6 md:grid-cols-2">
          {platforms.map(({ key, label, icon: Icon }) => {
            const latest = releases.find((r) => r.platform === key && r.file_path)
            return (
              <article key={key} className="card relative overflow-hidden p-8">
                <AudioWaveform className="absolute -right-6 -bottom-6 h-40 w-40 text-white/[0.03]" aria-hidden />
                <Icon size={30} className="text-sky" />
                <h3 className="mt-4 font-display text-xl font-semibold">{label}</h3>
                {latest ? (
                  <>
                    <p className="mt-2 font-mono text-xs text-muted">v{latest.version} · {new Date(latest.released_at).toLocaleDateString(t.locale)}</p>
                    {latest.notes && <p className="mt-3 whitespace-pre-line text-sm text-muted">{latest.notes}</p>}
                    <Link to={session ? '/account#downloads' : '/login'} className={`${btnPrimary} mt-6`}>
                      {session ? t.download.goDownloads : t.download.signInToDownload}
                    </Link>
                  </>
                ) : (
                  <p className="mt-6 font-display text-sm font-semibold tracking-[0.25em] text-gradient">{t.download.comingSoon}</p>
                )}
              </article>
            )
          })}
        </div>
      </div>
    </section>
  )
}
