import { useEffect, useState, type ReactNode } from 'react'
import { Link, NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom'
import { Menu, X } from 'lucide-react'
import { useAuth } from '../lib/auth'
import { supabase } from '../lib/supabase'

export const btnPrimary =
  'inline-flex items-center justify-center gap-2 rounded-full bg-gradient-to-r from-violet via-electric to-sky px-6 py-3 text-sm font-semibold tracking-wide text-white shadow-[0_0_30px_-6px_rgb(168_85_247/0.7)] transition hover:brightness-110 disabled:opacity-50 disabled:cursor-not-allowed'
export const btnGhost =
  'inline-flex items-center justify-center gap-2 rounded-full border border-line bg-white/5 px-6 py-3 text-sm font-semibold tracking-wide text-white transition hover:border-violet/60 hover:bg-white/10 disabled:opacity-50 disabled:cursor-not-allowed'

export function Logo() {
  return (
    <Link to="/" className="flex items-center gap-2.5 font-display text-lg font-bold tracking-[0.18em]" aria-label="COSOUND home">
      <img src="/favicon.svg" alt="" className="h-8 w-8" />
      COSOUND
    </Link>
  )
}

export function SectionHeading({ eyebrow, title, children }: { eyebrow: string; title: ReactNode; children?: ReactNode }) {
  return (
    <div className="mx-auto mb-12 max-w-2xl text-center">
      <p className="mb-3 font-mono text-xs uppercase tracking-[0.3em] text-sky">{eyebrow}</p>
      <h2 className="font-display text-3xl font-bold sm:text-4xl">{title}</h2>
      {children && <p className="mt-4 text-muted">{children}</p>}
    </div>
  )
}

const navLinks = [
  { to: '/#features', label: 'Features' },
  { to: '/#daws', label: 'DAWs' },
  { to: '/pricing', label: 'Pricing' },
  { to: '/download', label: 'Download' },
]

function Navbar() {
  const { session, isAdmin } = useAuth()
  const [open, setOpen] = useState(false)
  const navigate = useNavigate()
  const location = useLocation()
  useEffect(() => setOpen(false), [location])

  const signOut = async () => {
    await supabase.auth.signOut()
    navigate('/')
  }

  const links = (
    <>
      {navLinks.map((l) => (
        <NavLink key={l.to} to={l.to} className="text-sm text-muted transition hover:text-white">
          {l.label}
        </NavLink>
      ))}
    </>
  )
  const actions = session ? (
    <>
      {isAdmin && <Link to="/admin" className="text-sm text-muted hover:text-white">Admin</Link>}
      <Link to="/account" className={btnGhost + ' !px-4 !py-2'}>My account</Link>
      <button onClick={signOut} className="text-sm text-muted hover:text-white">Sign out</button>
    </>
  ) : (
    <>
      <Link to="/login" className="text-sm text-muted hover:text-white">Sign in</Link>
      <Link to="/signup" className={btnPrimary + ' !px-4 !py-2'}>Create account</Link>
    </>
  )

  return (
    <header className="sticky top-0 z-50 border-b border-line/70 bg-ink/80 backdrop-blur-xl">
      <nav className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6">
        <Logo />
        <div className="hidden items-center gap-8 md:flex">{links}</div>
        <div className="hidden items-center gap-4 md:flex">{actions}</div>
        <button className="md:hidden" onClick={() => setOpen(!open)} aria-label="Toggle menu" aria-expanded={open}>
          {open ? <X /> : <Menu />}
        </button>
      </nav>
      {open && (
        <div className="flex flex-col gap-4 border-t border-line px-4 py-5 md:hidden">
          {links}
          <div className="flex flex-wrap items-center gap-4 pt-2">{actions}</div>
        </div>
      )}
    </header>
  )
}

function Footer() {
  return (
    <footer className="border-t border-line bg-panel">
      <div className="mx-auto grid max-w-7xl gap-10 px-4 py-12 sm:px-6 md:grid-cols-3">
        <div>
          <Logo />
          <p className="mt-4 max-w-xs text-sm text-muted">The AI music production engineer that works inside your studio, with your own sounds.</p>
        </div>
        <div className="grid grid-cols-2 gap-2 text-sm text-muted">
          {navLinks.map((l) => <Link key={l.to} to={l.to} className="hover:text-white">{l.label}</Link>)}
          <Link to="/signup" className="hover:text-white">Create account</Link>
          <Link to="/login" className="hover:text-white">Sign in</Link>
        </div>
        <p className="text-xs leading-relaxed text-muted/80">
          Logic Pro, Ableton Live, FL Studio, Pro Tools and other product names are trademarks of their respective owners.
          COSOUND is an independent product and is not affiliated with, endorsed or sponsored by them.
        </p>
      </div>
      <p className="border-t border-line py-5 text-center text-xs text-muted/70">© {new Date().getFullYear()} COSOUND. All rights reserved.</p>
    </footer>
  )
}

export default function Layout() {
  const { hash, pathname } = useLocation()
  useEffect(() => {
    if (hash) document.getElementById(hash.slice(1))?.scrollIntoView()
    else window.scrollTo(0, 0)
  }, [hash, pathname])

  return (
    <div className="flex min-h-screen flex-col">
      <Navbar />
      <main className="flex-1">
        <Outlet />
      </main>
      <Footer />
    </div>
  )
}
