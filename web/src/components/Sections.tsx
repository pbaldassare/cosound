import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Apple, AudioWaveform, Check, Disc3, Drum, FolderSearch, Mic, Monitor, SlidersHorizontal, Sparkles, Waves } from 'lucide-react'
import { supabase, formatPrice, type Daw, type Plan, type Release } from '../lib/supabase'
import { useQuery } from '../lib/useQuery'
import { useAuth } from '../lib/auth'
import { SectionHeading, btnGhost, btnPrimary } from './Layout'

const features = [
  { icon: FolderSearch, title: 'Smart Library', text: 'Scans and catalogues your personal sample libraries, MIDI files, synths, plugins and presets, and listens to them to know what each sound really is.' },
  { icon: Sparkles, title: 'AI Music Producer', text: 'Turns your ideas into structure: harmony, chords, melodies and a full arrangement from intro to drop to outro.' },
  { icon: Drum, title: 'Rhythm Engineer', text: 'Picks the right kick from your library and builds drums, groove, swing, syncopation and BPM-locked rhythm sections.' },
  { icon: Waves, title: 'Sound Designer', text: 'Works your installed synths: oscillators, filters, envelopes, LFOs, presets and automation that evolve across the track.' },
  { icon: SlidersHorizontal, title: 'Mixing Engineer', text: 'Analyses balance, EQ, compression, kick/bass interaction, sidechain and stereo image, and explains every move.' },
  { icon: Disc3, title: 'Mastering Engineer', text: 'Guides tonal balance, dynamics, loudness and True Peak to deliver a release-ready master.' },
  { icon: Mic, title: 'Voice Prompts', text: 'Say what you hear in your head. COSOUND transcribes it and rewrites it into a precise production brief before acting.' },
]

export function FeaturesSection() {
  return (
    <section id="features" className="scroll-mt-20 px-4 py-24 sm:px-6">
      <div className="mx-auto max-w-7xl">
        <SectionHeading eyebrow="What COSOUND is being built to do" title={<>A full production team, <span className="text-gradient">in one engineer</span></>}>
          One COSOUND you talk to. Specialised agents behind it, sharing everything they know about your project.
        </SectionHeading>
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {features.map(({ icon: Icon, title, text }) => (
            <article key={title} className="card group p-6 transition hover:border-violet/50">
              <div className="mb-5 grid h-12 w-12 place-items-center rounded-xl bg-gradient-to-br from-violet/25 to-sky/20 text-sky ring-1 ring-white/10 transition group-hover:scale-105">
                <Icon size={22} />
              </div>
              <h3 className="font-display text-lg font-semibold">{title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">{text}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  )
}

const statusLabel: Record<Daw['status'], { text: string; cls: string }> = {
  available: { text: 'Available', cls: 'bg-emerald-500/15 text-emerald-300 ring-emerald-400/30' },
  in_development: { text: 'In development', cls: 'bg-violet/15 text-violet ring-violet/40' },
  planned: { text: 'Planned', cls: 'bg-white/5 text-muted ring-line' },
}

export function DawSection() {
  const { data } = useQuery(() => supabase.from('daws').select('*').order('sort'))
  const daws = (data ?? []) as Daw[]
  return (
    <section id="daws" className="scroll-mt-20 border-y border-line bg-panel/60 px-4 py-24 sm:px-6">
      <div className="mx-auto max-w-7xl">
        <SectionHeading eyebrow="Multi-DAW" title="One engine. Every studio.">
          A single central intelligence with a dedicated bridge for each DAW. We only mark an integration as available once it has been built and tested.
        </SectionHeading>
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {daws.map((d) => (
            <article key={d.slug} className="card flex flex-col p-6">
              <div className="mb-5 flex items-start justify-between gap-2">
                <div className="grid h-12 w-12 place-items-center rounded-xl bg-panel-2 font-display text-lg font-bold ring-1 ring-line" aria-hidden>
                  {d.name.split(' ').map((w) => w[0]).join('')}
                </div>
                <span className={`rounded-full px-2.5 py-1 text-[11px] font-medium ring-1 ${statusLabel[d.status].cls}`}>{statusLabel[d.status].text}</span>
              </div>
              <h3 className="font-display text-lg font-semibold">{d.name}</h3>
              <p className="font-mono text-[11px] uppercase tracking-widest text-muted">{d.vendor}</p>
              <p className="mt-3 text-sm leading-relaxed text-muted">{d.description}</p>
            </article>
          ))}
        </div>
        <p className="mt-8 text-center text-sm text-muted">More DAWs, such as Cubase, Studio One and Reaper, will be evaluated after the first integrations.</p>
      </div>
    </section>
  )
}

export function PricingSection() {
  const { session } = useAuth()
  const [yearly, setYearly] = useState(false)
  const { data, loading } = useQuery(() => supabase.from('plans').select('*').order('sort'))
  const plans = (data ?? []) as Plan[]

  return (
    <section id="pricing" className="scroll-mt-20 px-4 py-24 sm:px-6">
      <div className="mx-auto max-w-7xl">
        <SectionHeading eyebrow="Plans" title="Choose your studio level">
          Monthly or yearly subscriptions. Final prices will be announced before launch.
        </SectionHeading>
        <div className="mb-10 flex justify-center">
          <div className="inline-flex rounded-full border border-line bg-panel p-1 text-sm" role="group" aria-label="Billing period">
            {(['Monthly', 'Yearly'] as const).map((p) => (
              <button key={p} onClick={() => setYearly(p === 'Yearly')} aria-pressed={yearly === (p === 'Yearly')}
                className={`rounded-full px-5 py-2 transition ${yearly === (p === 'Yearly') ? 'bg-white/10 text-white' : 'text-muted'}`}>
                {p}
              </button>
            ))}
          </div>
        </div>
        {loading && <p className="text-center text-muted">Loading plans…</p>}
        <div className="grid gap-6 lg:grid-cols-3">
          {plans.map((p) => {
            const highlight = p.code === 'pro'
            const price = formatPrice(yearly ? p.price_yearly_cents : p.price_monthly_cents, p.currency)
            return (
              <article key={p.code} className={`card relative flex flex-col p-8 ${highlight ? 'border-violet/60 shadow-[0_0_60px_-20px_rgb(168_85_247/0.7)]' : ''}`}>
                {highlight && <span className="absolute -top-3 left-8 rounded-full bg-gradient-to-r from-violet to-electric px-3 py-1 text-[11px] font-semibold">Most complete for producers</span>}
                <p className="font-mono text-xs uppercase tracking-[0.3em] text-sky">{p.tagline}</p>
                <h3 className="mt-2 font-display text-2xl font-bold">{p.name}</h3>
                <p className="mt-5 font-display text-3xl font-bold">
                  {price ?? <span className="text-lg font-medium text-muted">Price announced soon</span>}
                  {price && <span className="text-base font-medium text-muted"> / {yearly ? 'year' : 'month'}</span>}
                </p>
                <ul className="mt-6 flex-1 space-y-3 text-sm">
                  {p.features.map((f) => (
                    <li key={f} className="flex gap-3"><Check size={18} className="shrink-0 text-sky" />{f}</li>
                  ))}
                </ul>
                {/* Stripe checkout plugs in here once prices and the checkout function exist (docs/SETUP.md). Nothing is charged until then. */}
                {session ? (
                  <button className={`${btnGhost} mt-8`} disabled>Subscriptions open soon</button>
                ) : (
                  <Link to="/signup" className={`${highlight ? btnPrimary : btnGhost} mt-8`}>Create your account</Link>
                )}
              </article>
            )
          })}
        </div>
      </div>
    </section>
  )
}

const platforms = [
  { key: 'macos', label: 'COSOUND for macOS', icon: Apple },
  { key: 'windows', label: 'COSOUND for Windows', icon: Monitor },
] as const

export function DownloadSection() {
  const { session } = useAuth()
  const { data } = useQuery(() => supabase.from('releases').select('*').eq('published', true).order('released_at', { ascending: false }))
  const releases = (data ?? []) as Release[]

  return (
    <section id="download" className="scroll-mt-20 px-4 py-24 sm:px-6">
      <div className="mx-auto max-w-5xl">
        <SectionHeading eyebrow="Desktop app" title="Runs on your machine, next to your DAW">
          COSOUND is a native desktop application. Your libraries stay on your computer.
        </SectionHeading>
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
                    <p className="mt-2 font-mono text-xs text-muted">v{latest.version} · {new Date(latest.released_at).toLocaleDateString()}</p>
                    {latest.notes && <p className="mt-3 whitespace-pre-line text-sm text-muted">{latest.notes}</p>}
                    <Link to={session ? '/account#downloads' : '/login'} className={`${btnPrimary} mt-6`}>
                      {session ? 'Go to your downloads' : 'Sign in to download'}
                    </Link>
                  </>
                ) : (
                  <p className="mt-6 font-display text-sm font-semibold tracking-[0.25em] text-gradient">COMING SOON – COSOUND DESKTOP</p>
                )}
              </article>
            )
          })}
        </div>
      </div>
    </section>
  )
}
