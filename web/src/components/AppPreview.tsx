import { Mic, Send } from 'lucide-react'
import { useT } from '../i18n'

// Concept mock-up of the future desktop interface: static markup, no real functionality.
const tracks = [
  { name: 'Kick', src: 'Library/Drums/Kicks/Industrial_K04.wav', color: 'from-violet to-fuchsia-500', clips: [[0, 100]] },
  { name: 'Bass', src: 'Serum · Dark Reese (user preset)', color: 'from-electric to-indigo-500', clips: [[12.5, 50], [62.5, 37.5]] },
  { name: 'Perc', src: 'Library/Loops/Metal_Perc_140.wav', color: 'from-sky to-cyan-600', clips: [[25, 25], [62.5, 37.5]] },
  { name: 'Synth', src: 'Predator · Hypno Lead (factory)', color: 'from-fuchsia-500 to-violet', clips: [[37.5, 62.5]] },
] as const

const bars = [0.5, 0.9, 0.35, 0.7, 1, 0.45, 0.8, 0.6, 0.95, 0.4, 0.75, 0.55]

export default function AppPreview() {
  const { t } = useT()
  return (
    <figure className="relative" aria-label={t.preview.label}>
      <div className="glow absolute -inset-6 -z-10 rounded-[2rem] bg-gradient-to-r from-violet/30 via-electric/20 to-sky/30 blur-3xl" />
      <div className="card overflow-hidden shadow-2xl">
        <div className="flex items-center gap-2 border-b border-line bg-panel-2 px-4 py-2.5">
          <span className="h-3 w-3 rounded-full bg-red-500/70" />
          <span className="h-3 w-3 rounded-full bg-yellow-500/70" />
          <span className="h-3 w-3 rounded-full bg-green-500/70" />
          <span className="ml-3 font-mono text-[11px] text-muted">COSOUND — Techno Session · 140 BPM · F minor</span>
        </div>

        <div className="grid md:grid-cols-[280px_1fr]">
          {/* Chat */}
          <div className="flex flex-col gap-3 border-b border-line p-4 text-[13px] md:border-r md:border-b-0">
            <div className="ml-auto max-w-[90%] rounded-2xl rounded-br-sm bg-violet/20 px-3 py-2">
              {t.preview.user}
            </div>
            <div className="max-w-[95%] rounded-2xl rounded-bl-sm border border-line bg-panel-2 px-3 py-2 text-muted">
              <span className="font-semibold text-white">{t.preview.ready}</span> {t.preview.reply}
              <div className="mt-2 flex gap-2">
                <span className="rounded-full bg-sky/15 px-2 py-0.5 text-[11px] text-sky">{t.preview.approve}</span>
                <span className="rounded-full bg-white/5 px-2 py-0.5 text-[11px]">{t.preview.changeKick}</span>
              </div>
            </div>
            <div className="mt-auto flex items-center gap-2 rounded-xl border border-line bg-ink px-3 py-2 text-muted">
              <span className="flex-1 truncate">{t.preview.input}</span>
              <Mic size={15} className="text-sky" />
              <Send size={15} />
            </div>
          </div>

          {/* Timeline */}
          <div className="p-4">
            <div className="mb-2 flex justify-between pl-24 font-mono text-[10px] text-muted">
              {['1', '9', '17', '25', '33'].map((b) => <span key={b}>{b}</span>)}
            </div>
            <div className="relative space-y-2">
              {tracks.map((t) => (
                <div key={t.name} className="flex items-center gap-3">
                  <div className="w-21 shrink-0">
                    <p className="text-xs font-semibold">{t.name}</p>
                    <p className="truncate font-mono text-[9px] text-muted" title={t.src}>{t.src}</p>
                  </div>
                  <div className="relative h-9 flex-1 rounded-md bg-ink/80 grid-bg">
                    {t.clips.map(([left, width], i) => (
                      <div key={i} className={`absolute inset-y-1 rounded bg-gradient-to-r ${t.color} opacity-85`} style={{ left: `${left}%`, width: `${width}%` }} />
                    ))}
                  </div>
                </div>
              ))}
              <div className="pointer-events-none absolute inset-y-0 left-24 right-0">
                <div className="playhead absolute inset-y-0 w-px bg-sky shadow-[0_0_8px_#22d3ee]" />
              </div>
            </div>
            <div className="mt-4 flex items-end justify-between gap-4 border-t border-line pt-3">
              <div className="flex h-10 items-end gap-1" aria-hidden>
                {bars.map((h, i) => (
                  <span key={i} className="eq-bar w-1.5 rounded-full bg-gradient-to-t from-violet to-sky" style={{ height: `${h * 100}%`, animationDelay: `${i * 0.11}s` }} />
                ))}
              </div>
              <div className="flex gap-4 font-mono text-[10px] text-muted">
                <span>LUFS <b className="text-white">-8.2</b></span>
                <span>TP <b className="text-white">-1.0</b></span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <figcaption className="mt-3 text-center font-mono text-[11px] text-muted">{t.preview.caption}</figcaption>
    </figure>
  )
}
