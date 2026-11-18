import { Link } from 'react-router-dom'
import { Cpu, FolderOpen, MessageSquareText, ShieldCheck } from 'lucide-react'
import AppPreview from '../components/AppPreview'
import { SectionHeading, btnGhost, btnPrimary } from '../components/Layout'
import { DawSection, DownloadSection, FeaturesSection, PricingSection } from '../components/Sections'
import { useT } from '../i18n'

const stepIcons = [FolderOpen, MessageSquareText, Cpu]

export default function Home() {
  const { t } = useT()
  const h = t.home
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden px-4 pt-20 pb-24 sm:px-6 lg:pt-28">
        <div className="grid-bg absolute inset-0 -z-10 [mask-image:radial-gradient(ellipse_at_top,black_30%,transparent_75%)]" />
        <div className="absolute top-0 left-1/2 -z-10 h-[500px] w-[900px] -translate-x-1/2 rounded-full bg-violet/20 blur-[120px]" />
        <div className="mx-auto max-w-7xl">
          <div className="mx-auto max-w-4xl text-center">
            <p className="mb-6 inline-flex items-center gap-2 rounded-full border border-line bg-panel/80 px-4 py-1.5 font-mono text-xs tracking-widest text-muted">
              <span className="h-1.5 w-1.5 rounded-full bg-sky shadow-[0_0_8px_#22d3ee]" /> {h.badge}
            </p>
            <h1 className="font-display text-4xl leading-[1.05] font-bold tracking-tight sm:text-6xl lg:text-7xl">
              {h.title1}<br /><span className="text-gradient">{h.title2}</span>
            </h1>
            <p className="mx-auto mt-6 max-w-2xl text-lg text-muted">{h.subtitle}</p>
            <div className="mt-10 flex flex-col justify-center gap-4 sm:flex-row">
              <a href="#how" className={btnPrimary}>{h.discover}</a>
              <Link to="/signup" className={btnGhost}>{h.create}</Link>
            </div>
          </div>
          <div className="mx-auto mt-20 max-w-5xl">
            <AppPreview />
          </div>
        </div>
      </section>

      {/* How it works */}
      <section id="how" className="scroll-mt-20 border-y border-line bg-panel/60 px-4 py-24 sm:px-6">
        <div className="mx-auto max-w-7xl">
          <SectionHeading eyebrow={h.howEyebrow} title={<>{h.howTitleA} <span className="text-gradient">{h.howTitleB}</span></>}>
            {h.howText}
          </SectionHeading>
          <ol className="grid gap-6 md:grid-cols-3">
            {h.steps.map(({ title, text }, i) => {
              const Icon = stepIcons[i]
              return (
                <li key={title} className="card p-7">
                  <div className="flex items-center gap-3">
                    <span className="font-mono text-sm text-sky">0{i + 1}</span>
                    <Icon size={20} className="text-violet" />
                  </div>
                  <h3 className="mt-4 font-display text-xl font-semibold">{title}</h3>
                  <p className="mt-2 text-sm leading-relaxed text-muted">{text}</p>
                </li>
              )
            })}
          </ol>
        </div>
      </section>

      {/* Your sounds principle */}
      <section className="px-4 py-24 sm:px-6">
        <div className="card mx-auto grid max-w-6xl items-center gap-10 p-8 sm:p-12 lg:grid-cols-2">
          <div>
            <p className="mb-3 font-mono text-xs uppercase tracking-[0.3em] text-sky">{h.principleEyebrow}</p>
            <h2 className="font-display text-3xl font-bold sm:text-4xl">{h.principleTitle}</h2>
            <p className="mt-5 text-muted">{h.principle1}</p>
            <p className="mt-4 text-muted">{h.principle2}</p>
          </div>
          <ul className="space-y-4">
            {h.points.map((p) => (
              <li key={p} className="flex gap-3 rounded-xl border border-line bg-ink/60 p-4 text-sm">
                <ShieldCheck size={20} className="shrink-0 text-sky" /> {p}
              </li>
            ))}
          </ul>
        </div>
      </section>

      <FeaturesSection />
      <DawSection />
      <PricingSection />
      <DownloadSection />
    </>
  )
}
