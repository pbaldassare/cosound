import { createClient, type SupabaseClient } from '@supabase/supabase-js'

const url = import.meta.env.VITE_SUPABASE_URL as string | undefined
const key = import.meta.env.VITE_SUPABASE_PUBLISHABLE_KEY as string | undefined

/** null when the build has no Supabase config: local features keep working, account features say so. */
export const supabase: SupabaseClient | null = url && key ? createClient(url, key) : null
export const siteUrl = (import.meta.env.VITE_SITE_URL as string | undefined) || null

export type Plan = { code: 'base' | 'pro' | 'premium'; name: string; tagline: string; features: string[]; features_it: string[] | null; sort: number }
export type Subscription = { plan_code: string; status: 'trialing' | 'active' | 'past_due' | 'canceled'; billing_interval: 'month' | 'year' | null; current_period_end: string | null }
export type Profile = { id: string; first_name: string; last_name: string; assistant_name: string | null }
