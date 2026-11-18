import { useCallback, useEffect, useState } from 'react'

type Result<T> = { data: T | null; error: { message: string } | null }

/** Minimal loader for a Supabase query. `reload` re-runs it after a mutation. */
export function useQuery<T>(run: () => PromiseLike<Result<T>>, deps: unknown[] = []) {
  const [state, setState] = useState<{ data: T | null; error: string | null; loading: boolean }>({ data: null, error: null, loading: true })
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const reload = useCallback(async () => {
    const { data, error } = await run()
    setState({ data, error: error?.message ?? null, loading: false })
  }, deps)
  useEffect(() => { reload() }, [reload])
  return { ...state, reload }
}
