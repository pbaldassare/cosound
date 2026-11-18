import { useEffect, useState, type ReactNode } from 'react'
import { Link, NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom'
import { Menu, X } from 'lucide-react'
import { useAuth } from '../lib/auth'
import { supabase } from '../lib/supabase'
import { LanguageToggle, useT } from '../i18n'

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

const useNavLinks = () => {
  const { t } = useT()
  return [
    { to: '/#features', label: t.nav.features },
    { to: '/#daws', label: t.nav.daws },
    { to: '/pricing', label: t.nav.pricing },
    { to: '/download', label: t.nav.download },
  ]
}

function Navbar() {
  const { session, isAdmin } = useAuth()
  const { t } = useT()
  const navLinks = useNavLinks()
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
      {isAdmin && <Link to="/admin" className="text-sm text-muted hover:text-white">{t.nav.admin}</Link>}
      <Link to="/account" className={btnGhost + ' !px-4 !py-2'}>{t.nav.myAccount}</Link>
      <button onClick={signOut} className="text-sm text-muted hover:text-white">{t.nav.signOut}</button>
    </>
  ) : (
    <>
      <Link to="/login" className="text-sm text-muted hover:text-white">{t.nav.signIn}</Link>
      <Link to="/signup" className={btnPrimary + ' !px-4 !py-2'}>{t.nav.createAccount}</Link>
    </>
  )

  return (
    <header className="sticky top-0 z-50 border-b border-line/70 bg-ink/80 backdrop-blur-xl">
      <nav className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6">
        <Logo />
        <div className="hidden items-center gap-8 md:flex">{links}</div>
        <div className="hidden items-center gap-4 md:flex"><LanguageToggle />{actions}</div>
        <div className="flex items-center gap-3 md:hidden">
          <LanguageToggle />
          <button onClick={() => setOpen(!open)} aria-label={t.nav.menu} aria-expanded={open}>
            {open ? <X /> : <Menu />}
          </button>
        </div>
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
  const { t } = useT()
  const navLinks = useNavLinks()
  return (
    <footer className="border-t border-line bg-panel">
      <div className="mx-auto grid max-w-7xl gap-10 px-4 py-12 sm:px-6 md:grid-cols-3">
        <div>
          <Logo />
          <p className="mt-4 max-w-xs text-sm text-muted">{t.footer.tagline}</p>
        </div>
        <div className="grid grid-cols-2 gap-2 text-sm text-muted">
          {navLinks.map((l) => <Link key={l.to} to={l.to} className="hover:text-white">{l.label}</Link>)}
          <Link to="/signup" className="hover:text-white">{t.nav.createAccount}</Link>
          <Link to="/login" className="hover:text-white">{t.nav.signIn}</Link>
        </div>
        <p className="text-xs leading-relaxed text-muted/80">
          {t.footer.trademarks}
        </p>
      </div>
      <p className="border-t border-line py-5 text-center text-xs text-muted/70">© {new Date().getFullYear()} COSOUND. {t.footer.rights}</p>
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
