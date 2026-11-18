# COSOUND – Setup & status (Prompt 01: website foundations)

## Stack
- **web/**: Vite + React 19 + TypeScript + Tailwind CSS 4 + React Router. Icons: lucide-react.
- **Backend**: Supabase project `dhfdqmlbjmitpqpohgno` (Auth, Postgres with RLS, private Storage).
- **supabase/migrations/**: the database schema, already applied.

## Run locally
    cd web && cp .env.example .env && npm install && npm run dev

## Data model
| Table | Purpose | Who can write |
|---|---|---|
| profiles | first/last name (auto-created on sign-up) | the user (own row) |
| admins | who is an administrator | nobody via API (SQL only) |
| plans | Base / Pro / Premium, features, prices (NULL = not announced), Stripe price IDs | admin |
| subscriptions | plan + status per user | admin / future Stripe webhook |
| daws | DAW cards: available / in_development / planned | admin |
| releases | desktop versions; installers in the private `releases` bucket | admin |
| devices | authorized computers (filled by the future desktop app) | admin |
| announcements | messages shown in the personal area | admin |

Installers download only via a 60-second signed URL, for admins or users with an active/trialing subscription.

## To configure (owner)
1. **Make yourself admin**: sign up on the site, then in the Supabase SQL editor:
   `insert into public.admins (user_id) select id from auth.users where email = 'YOUR_EMAIL';`
2. **Auth → URL configuration**: Site URL = production domain; add `https://DOMAIN/account`, `https://DOMAIN/reset-password` and `http://localhost:5173/**` to Redirect URLs.
3. **Auth → Email**: custom SMTP (the built-in sender is rate-limited, not for production); brand the confirm/reset templates.
4. **Auth → Passwords**: minimum length 8, enable leaked-password protection.
5. **Stripe** (once prices are decided): create products/prices, put price IDs in Admin → Plans, then add a checkout Edge Function + webhook that writes `subscriptions` with the service role.
6. **Hosting**: deploy `web/` (Vercel/Netlify) with SPA fallback to `index.html` and the two `VITE_` env vars.
7. **Legal**: privacy policy, terms and cookie notice are required (GDPR) before public sign-ups.
8. **Test data**: delete users `e2e-test@cosound.test` and `cosound.e2e.*@gmail.com` (Auth → Users) if still present.
