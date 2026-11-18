-- Trigger-only function: never callable through the API.
revoke execute on function public.handle_new_user() from anon, authenticated, public;
