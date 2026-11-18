import { Link } from 'react-router-dom'
import { Cpu, FolderOpen, MessageSquareText, ShieldCheck } from 'lucide-react'
import AppPreview from '../components/AppPreview'
import { SectionHeading, btnGhost, btnPrimary } from '../components/Layout'
import { DawSection, DownloadSection, FeaturesSection, PricingSection } from '../components/Sections'

const steps = [
  { icon: FolderOpen, title: 'It learns your studio', text: 'With your permission, COSOUND maps your samples, loops, MIDI, synths, plugins, presets and installed DAWs, and keeps the catalogue updated.' },
  { icon: MessageSquareText, title: 'You describe the track', text: 'Type or speak: "Techno at 140 BPM, punchy kick, dark aggressive bass". COSOUND answers with a concrete proposal built from your own sounds.' },
  { icon: Cpu, title: 'It works in your DAW', text: 'Once you approve, it creates tracks, places your samples, programs MIDI and sets up instruments, and edits the project when you ask, without starting over.' },
]

export default function Home() {
  return (
    <>
      {/* Hero */}
      <section className="relative overflow-hidden px-4 pt-20 pb-24 sm:px-6 lg:pt-28">
        <div className="grid-bg absolute inset-0 -z-10 [mask-image:radial-gradient(ellipse_at_top,black_30%,transparent_75%)]" />
        <div className="absolute top-0 left-1/2 -z-10 h-[500px] w-[900px] -translate-x-1/2 rounded-full bg-violet/20 blur-[120px]" />
        <div className="mx-auto max-w-7xl">
          <div className="mx-auto max-w-4xl text-center">
            <p className="mb-6 inline-flex items-center gap-2 rounded-full border border-line bg-panel/80 px-4 py-1.5 font-mono text-xs tracking-widest text-muted">
              <span className="h-1.5 w-1.5 rounded-full bg-sky shadow-[0_0_8px_#22d3ee]" /> AI MUSIC PRODUCTION ENGINEER
            </p>
            <h1 className="font-display text-4xl leading-[1.05] font-bold tracking-tight sm:text-6xl lg:text-7xl">
              YOUR MUSIC. YOUR STUDIO.<br /><span className="text-gradient">YOUR AI PRODUCER.</span>
            </h1>
            <p className="mx-auto mt-6 max-w-2xl text-lg text-muted">
              Meet COSOUND, your intelligent music production engineer. Create, arrange, design, mix and master using your own instruments, plugins and sound libraries.
            </p>
            <div className="mt-10 flex flex-col justify-center gap-4 sm:flex-row">
              <a href="#how" className={btnPrimary}>DISCOVER COSOUND</a>
              <Link to="/signup" className={btnGhost}>CREATE YOUR ACCOUNT</Link>
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
          <SectionHeading eyebrow="How it works" title={<>Not a song generator. <span className="text-gradient">Your production partner.</span></>}>
            COSOUND doesn't generate a track for you from nothing. It works the way a producer, arranger, sound designer and engineer would, inside your DAW, with the material you already own.
          </SectionHeading>
          <ol className="grid gap-6 md:grid-cols-3">
            {steps.map(({ icon: Icon, title, text }, i) => (
              <li key={title} className="card p-7">
                <div className="flex items-center gap-3">
                  <span className="font-mono text-sm text-sky">0{i + 1}</span>
                  <Icon size={20} className="text-violet" />
                </div>
                <h3 className="mt-4 font-display text-xl font-semibold">{title}</h3>
                <p className="mt-2 text-sm leading-relaxed text-muted">{text}</p>
              </li>
            ))}
          </ol>
        </div>
      </section>

      {/* Your sounds principle */}
      <section className="px-4 py-24 sm:px-6">
        <div className="card mx-auto grid max-w-6xl items-center gap-10 p-8 sm:p-12 lg:grid-cols-2">
          <div>
            <p className="mb-3 font-mono text-xs uppercase tracking-[0.3em] text-sky">The core principle</p>
            <h2 className="font-display text-3xl font-bold sm:text-4xl">Built with your sounds. Signed by you.</h2>
            <p className="mt-5 text-muted">
              If you have ten kicks, COSOUND listens to all ten, compares them and picks the one that fits the track. It never replaces them with a synthetic one.
              If nothing in your library fits, it tells you and suggests what to add.
            </p>
            <p className="mt-4 text-muted">You approve every step. The result is a normal project in your DAW, made of your material, that you can open and change track by track.</p>
          </div>
          <ul className="space-y-4">
            {[
              'Your libraries stay on your computer: no need to upload them',
              'Every proposal waits for your approval',
              'A log of the samples, presets and licenses used in each track',
              'Memory of your preferences and production style',
            ].map((t) => (
              <li key={t} className="flex gap-3 rounded-xl border border-line bg-ink/60 p-4 text-sm">
                <ShieldCheck size={20} className="shrink-0 text-sky" /> {t}
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
