import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Link, Route, Routes } from 'react-router-dom'
import './index.css'
import { AuthProvider, RequireAuth } from './lib/auth'
import Layout, { btnPrimary } from './components/Layout'
import { DownloadSection, PricingSection } from './components/Sections'
import Home from './pages/Home'
import { ForgotPassword, Login, ResetPassword, Signup } from './pages/Auth'
import Dashboard from './pages/account/Dashboard'
import Admin from './pages/admin/Admin'

function NotFound() {
  return (
    <div className="grid min-h-[60vh] place-items-center px-4 text-center">
      <div>
        <p className="font-mono text-sm text-sky">404</p>
        <h1 className="mt-2 font-display text-3xl font-bold">This page is off the grid</h1>
        <Link to="/" className={`${btnPrimary} mt-8`}>Back to home</Link>
      </div>
    </div>
  )
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <AuthProvider>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<Home />} />
            <Route path="pricing" element={<PricingSection />} />
            <Route path="download" element={<DownloadSection />} />
            <Route path="signup" element={<Signup />} />
            <Route path="login" element={<Login />} />
            <Route path="forgot-password" element={<ForgotPassword />} />
            <Route path="reset-password" element={<ResetPassword />} />
            <Route path="account" element={<RequireAuth><Dashboard /></RequireAuth>} />
            <Route path="admin" element={<RequireAuth admin><Admin /></RequireAuth>} />
            <Route path="*" element={<NotFound />} />
          </Route>
        </Routes>
      </AuthProvider>
    </BrowserRouter>
  </StrictMode>,
)
