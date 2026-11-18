-- COSOUND foundation schema: profiles, roles, plans, subscriptions, DAWs, releases, devices, announcements.

-- ---------- Roles ----------
create table public.admins (
  user_id uuid primary key references auth.users on delete cascade,
  created_at timestamptz not null default now()
);
alter table public.admins enable row level security;
create policy "admins: read own" on public.admins for select to authenticated using (user_id = (select auth.uid()));

create or replace function public.is_admin() returns boolean
language sql stable security definer set search_path = '' as $$
  select exists (select 1 from public.admins where user_id = (select auth.uid()));
$$;

-- ---------- Profiles ----------
create table public.profiles (
  id uuid primary key references auth.users on delete cascade,
  first_name text not null default '' check (char_length(first_name) <= 80),
  last_name text not null default '' check (char_length(last_name) <= 80),
  created_at timestamptz not null default now()
);
alter table public.profiles enable row level security;
create policy "profiles: read own or admin" on public.profiles for select to authenticated
  using (id = (select auth.uid()) or (select public.is_admin()));
create policy "profiles: update own" on public.profiles for update to authenticated
  using (id = (select auth.uid())) with check (id = (select auth.uid()));

create or replace function public.handle_new_user() returns trigger
language plpgsql security definer set search_path = '' as $$
begin
  insert into public.profiles (id, first_name, last_name)
  values (new.id,
          left(coalesce(new.raw_user_meta_data->>'first_name', ''), 80),
          left(coalesce(new.raw_user_meta_data->>'last_name', ''), 80));
  return new;
end $$;
create trigger on_auth_user_created after insert on auth.users
  for each row execute function public.handle_new_user();

-- ---------- Plans (prices stay NULL until defined) ----------
create table public.plans (
  code text primary key check (code in ('base', 'pro', 'premium')),
  name text not null,
  tagline text not null,
  features text[] not null default '{}',
  price_monthly_cents integer check (price_monthly_cents >= 0),
  price_yearly_cents integer check (price_yearly_cents >= 0),
  currency text not null default 'EUR',
  stripe_price_monthly text,
  stripe_price_yearly text,
  sort integer not null default 0
);
alter table public.plans enable row level security;
create policy "plans: public read" on public.plans for select to anon, authenticated using (true);
create policy "plans: admin write" on public.plans for all to authenticated
  using ((select public.is_admin())) with check ((select public.is_admin()));

-- ---------- Subscriptions (written only by admin / future Stripe webhook with service role) ----------
create table public.subscriptions (
  user_id uuid primary key references auth.users on delete cascade,
  plan_code text not null references public.plans,
  status text not null check (status in ('trialing', 'active', 'past_due', 'canceled')),
  billing_interval text check (billing_interval in ('month', 'year')),
  current_period_end timestamptz,
  stripe_customer_id text,
  stripe_subscription_id text unique,
  updated_at timestamptz not null default now()
);
alter table public.subscriptions enable row level security;
create policy "subscriptions: read own or admin" on public.subscriptions for select to authenticated
  using (user_id = (select auth.uid()) or (select public.is_admin()));
create policy "subscriptions: admin write" on public.subscriptions for all to authenticated
  using ((select public.is_admin())) with check ((select public.is_admin()));

create or replace function public.has_download_access() returns boolean
language sql stable security definer set search_path = '' as $$
  select (select public.is_admin()) or exists (
    select 1 from public.subscriptions
    where user_id = (select auth.uid()) and status in ('trialing', 'active'));
$$;

-- ---------- DAW integrations ----------
create table public.daws (
  slug text primary key,
  name text not null,
  vendor text not null,
  description text not null,
  status text not null check (status in ('available', 'in_development', 'planned')),
  sort integer not null default 0
);
alter table public.daws enable row level security;
create policy "daws: public read" on public.daws for select to anon, authenticated using (true);
create policy "daws: admin write" on public.daws for all to authenticated
  using ((select public.is_admin())) with check ((select public.is_admin()));

-- ---------- Desktop releases ----------
create table public.releases (
  id uuid primary key default gen_random_uuid(),
  platform text not null check (platform in ('macos', 'windows')),
  version text not null,
  released_at date not null default current_date,
  notes text not null default '',
  file_path text, -- object path in the private 'releases' storage bucket
  published boolean not null default false,
  created_at timestamptz not null default now(),
  unique (platform, version)
);
alter table public.releases enable row level security;
create policy "releases: read published or admin" on public.releases for select to anon, authenticated
  using (published or (select public.is_admin()));
create policy "releases: admin write" on public.releases for all to authenticated
  using ((select public.is_admin())) with check ((select public.is_admin()));

-- ---------- Authorized devices (populated by the future desktop app) ----------
create table public.devices (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users on delete cascade,
  name text not null,
  platform text not null check (platform in ('macos', 'windows')),
  last_seen_at timestamptz,
  revoked_at timestamptz,
  created_at timestamptz not null default now()
);
create index devices_user_id_idx on public.devices (user_id);
alter table public.devices enable row level security;
create policy "devices: read own or admin" on public.devices for select to authenticated
  using (user_id = (select auth.uid()) or (select public.is_admin()));
create policy "devices: admin write" on public.devices for all to authenticated
  using ((select public.is_admin())) with check ((select public.is_admin()));

-- ---------- Announcements ----------
create table public.announcements (
  id uuid primary key default gen_random_uuid(),
  title text not null,
  body text not null default '',
  published boolean not null default false,
  created_at timestamptz not null default now()
);
alter table public.announcements enable row level security;
create policy "announcements: read published or admin" on public.announcements for select to anon, authenticated
  using (published or (select public.is_admin()));
create policy "announcements: admin write" on public.announcements for all to authenticated
  using ((select public.is_admin())) with check ((select public.is_admin()));

-- ---------- Admin user listing (emails live in auth.users) ----------
create or replace function public.admin_list_users()
returns table (id uuid, email text, first_name text, last_name text, created_at timestamptz,
               email_confirmed boolean, plan_code text, status text)
language plpgsql stable security definer set search_path = '' as $$
begin
  if not public.is_admin() then raise exception 'forbidden'; end if;
  return query
    select u.id, u.email::text, p.first_name, p.last_name, u.created_at,
           u.email_confirmed_at is not null, s.plan_code, s.status
    from auth.users u
    left join public.profiles p on p.id = u.id
    left join public.subscriptions s on s.user_id = u.id
    order by u.created_at desc;
end $$;
revoke execute on function public.admin_list_users() from anon, public;

-- ---------- Private storage for installers ----------
insert into storage.buckets (id, name, public) values ('releases', 'releases', false);
create policy "releases bucket: entitled read" on storage.objects for select to authenticated
  using (bucket_id = 'releases' and (select public.has_download_access()));
create policy "releases bucket: admin insert" on storage.objects for insert to authenticated
  with check (bucket_id = 'releases' and (select public.is_admin()));
create policy "releases bucket: admin update" on storage.objects for update to authenticated
  using (bucket_id = 'releases' and (select public.is_admin()));
create policy "releases bucket: admin delete" on storage.objects for delete to authenticated
  using (bucket_id = 'releases' and (select public.is_admin()));

-- ---------- Seed: plans (no prices) and DAWs (nothing available yet) ----------
insert into public.plans (code, name, tagline, features, sort) values
 ('base', 'Music Producer', 'Base', array[
   'Personal library analysis', 'Sample search and selection', 'MIDI creation',
   'Assisted arrangement', 'Placing material in your DAW', 'Intelligent chat and voice prompts'], 1),
 ('pro', 'Advanced Producer', 'Pro', array[
   'Everything in Base', 'Advanced synthesizer control', 'Sound design', 'Automation',
   'Assisted mixing', 'Technical project analysis'], 2),
 ('premium', 'Studio Master', 'Premium', array[
   'Everything in Pro', 'Assisted mastering', 'Advanced mix analysis', 'Loudness and True Peak control',
   'Mastering chains', 'Final master technical check'], 3);

insert into public.daws (slug, name, vendor, description, status, sort) values
 ('logic-pro', 'Logic Pro', 'Apple', 'First integration target. COSOUND will build tracks, place samples and program MIDI inside your Logic projects on macOS.', 'in_development', 1),
 ('ableton-live', 'Ableton Live', 'Ableton', 'Session and Arrangement view integration with your racks, devices and sample libraries.', 'planned', 2),
 ('fl-studio', 'FL Studio', 'Image-Line', 'Channel rack, Piano roll and Playlist integration using your installed generators and samples.', 'planned', 3),
 ('pro-tools', 'Pro Tools', 'Avid', 'Session-level assistance for arrangement, editing and mix preparation.', 'planned', 4);
