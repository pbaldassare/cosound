import { createClient } from '@supabase/supabase-js'

const url = import.meta.env.VITE_SUPABASE_URL
const key = import.meta.env.VITE_SUPABASE_PUBLISHABLE_KEY
if (!url || !key) throw new Error('Missing VITE_SUPABASE_URL / VITE_SUPABASE_PUBLISHABLE_KEY (see .env.example)')

export const supabase = createClient(url, key)

export type Plan = {
  code: 'base' | 'pro' | 'premium'
  name: string
  tagline: string
  features: string[]
  features_it: string[]
  price_monthly_cents: number | null
  price_yearly_cents: number | null
  currency: string
  stripe_price_monthly: string | null
  stripe_price_yearly: string | null
  sort: number
}

export type Daw = {
  slug: string
  name: string
  vendor: string
  description: string
  description_it: string
  status: 'available' | 'in_development' | 'planned'
  sort: number
}

export type Release = {
  id: string
  platform: 'macos' | 'windows'
  version: string
  released_at: string
  notes: string
  file_path: string | null
  published: boolean
}

export type Subscription = {
  plan_code: string
  status: 'trialing' | 'active' | 'past_due' | 'canceled'
  billing_interval: 'month' | 'year' | null
  current_period_end: string | null
}

export type Device = {
  id: string
  name: string
  platform: string
  last_seen_at: string | null
  revoked_at: string | null
  created_at: string
}

export type Announcement = { id: string; title: string; body: string; published: boolean; created_at: string }

export const formatPrice = (cents: number | null, currency: string, locale: string) =>
  cents == null ? null : new Intl.NumberFormat(locale, { style: 'currency', currency }).format(cents / 100)
