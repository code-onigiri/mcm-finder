import { SearchRequest, SearchResponse } from './api';

const STORAGE_PREFIX = 'mcm-finder:search:';
const CACHE_VERSION = '2';
const CACHE_TTL_MS = 7 * 24 * 60 * 60 * 1000;

interface StoredSession {
  cache_version: string;
  cached_at: string;
  expires_at: string;
  response: SearchResponse;
}

function normalizeRequest(request: SearchRequest): string {
  const normalized = {
    keywords: [...request.keywords].map((keyword) => keyword.trim().toLowerCase()).filter(Boolean),
    minecraft_version: request.filters?.minecraft_version?.trim().toLowerCase() || '',
    loaders: [...(request.filters?.loaders || [])].map((loader) => loader.toLowerCase()).sort(),
    categories: [...(request.filters?.categories || [])]
      .map((category) => category.trim().toLowerCase())
      .filter(Boolean)
      .sort(),
    update_recency_window_days: request.filters?.update_recency_window_days ?? null,
    min_downloads: request.filters?.min_downloads ?? null,
    open_source_only: request.filters?.open_source_only ?? false,
    providers: [...(request.provider_scope || ['modrinth', 'curseforge'])]
      .map((provider) => provider.toLowerCase())
      .sort(),
    sort_mode: request.sort_mode || 'Relevance',
    offset: request.offset ?? 0,
    limit: request.limit ?? null,
  };

  return JSON.stringify(normalized);
}

function getStorageKey(request: SearchRequest): string {
  return `${STORAGE_PREFIX}${encodeURIComponent(normalizeRequest(request))}`;
}

export function getSessionCache(request: SearchRequest): SearchResponse | null {
  try {
    const storageKey = getStorageKey(request);
    const raw = localStorage.getItem(storageKey);
    if (!raw) {
      return null;
    }

    const parsed = JSON.parse(raw) as StoredSession;

    // T067: freshness validation with cache_version and expires_at.
    if (parsed.cache_version !== CACHE_VERSION) {
      localStorage.removeItem(storageKey);
      return null;
    }

    if (Date.now() >= new Date(parsed.expires_at).getTime()) {
      localStorage.removeItem(storageKey);
      return null;
    }

    return {
      ...parsed.response,
      metadata: {
        ...parsed.response.metadata,
        cached_at: parsed.cached_at,
        expires_at: parsed.expires_at,
        cache_version: parsed.cache_version,
      },
    };
  } catch {
    return null;
  }
}

export function setSessionCache(request: SearchRequest, response: SearchResponse): void {
  const storageKey = getStorageKey(request);
  const cachedAt = new Date();
  const expiresAt = new Date(cachedAt.getTime() + CACHE_TTL_MS);

  const payload: StoredSession = {
    cache_version: CACHE_VERSION,
    cached_at: cachedAt.toISOString(),
    expires_at: expiresAt.toISOString(),
    response,
  };

  localStorage.setItem(storageKey, JSON.stringify(payload));
}

export function removeSessionCache(request: SearchRequest): void {
  localStorage.removeItem(getStorageKey(request));
}
