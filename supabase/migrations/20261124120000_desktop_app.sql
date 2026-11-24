-- Prompt 04 – desktop app: assistant name in the profile, configurable trial length.
-- Additive only.

alter table public.profiles
  add column assistant_name text check (assistant_name is null or char_length(assistant_name) between 1 and 40);

-- Public, non-secret app configuration. Admin-only writes.
create table public.app_settings (
  key text primary key,
  value jsonb not null,
  updated_at timestamptz not null default now()
);
alter table public.app_settings enable row level security;
create policy "app_settings readable by everyone" on public.app_settings for select using (true);
create policy "app_settings admin write" on public.app_settings for all
  using ((select public.is_admin())) with check ((select public.is_admin()));

-- Free trial: 3 or 7 days (Prompt 03/04). Payments and auto-renewal are not active.
insert into public.app_settings (key, value) values ('trial_days', '7');
alter table public.app_settings add constraint trial_days_3_or_7
  check (key <> 'trial_days' or value in ('3'::jsonb, '7'::jsonb));
