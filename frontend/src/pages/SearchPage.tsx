import React, { useMemo, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import DegradedModeNotice from '../components/DegradedModeNotice';
import FilterPanel, { FilterPanelState } from '../components/FilterPanel';
import ProgressIndicator from '../components/ProgressIndicator';
import ResultList from '../components/ResultList';
import SavedSessionsList from '../components/SavedSessionsList';
import SearchForm, { SearchFormInput } from '../components/SearchForm';
import SessionComparison from '../components/SessionComparison';
import SortSelector from '../components/SortSelector';
import {
  SearchRequest,
  SearchResponse,
  SortMode,
  getSession,
  saveSession,
  searchMods,
} from '../services/api';
import { getSessionCache, removeSessionCache, setSessionCache } from '../services/sessionCache';

const DEFAULT_FILTERS: FilterPanelState = {
  loaders: [],
  categories: [],
  open_source_only: false,
};

function buildFilterRequest(formVersion: string | undefined, filters: FilterPanelState): SearchRequest['filters'] {
  const minecraftVersion = filters.minecraft_version?.trim() || formVersion;

  const payload: NonNullable<SearchRequest['filters']> = {
    ...(minecraftVersion ? { minecraft_version: minecraftVersion } : {}),
    ...(filters.loaders.length > 0 ? { loaders: filters.loaders } : {}),
    ...(filters.categories.length > 0 ? { categories: filters.categories } : {}),
    ...(typeof filters.update_recency_window_days === 'number'
      ? { update_recency_window_days: filters.update_recency_window_days }
      : {}),
    ...(typeof filters.min_downloads === 'number' ? { min_downloads: filters.min_downloads } : {}),
    ...(filters.open_source_only ? { open_source_only: true } : {}),
  };

  return Object.keys(payload).length > 0 ? payload : undefined;
}

async function performSearch(request: SearchRequest): Promise<SearchResponse> {
  if (!request.force_refresh) {
    const cached = getSessionCache(request);
    if (cached) {
      return cached;
    }
  } else {
    removeSessionCache(request);
  }

  const response = await searchMods(request);
  setSessionCache(request, response);
  return response;
}

function randomUuid(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }

  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (char) => {
    const random = Math.floor(Math.random() * 16);
    const value = char === 'x' ? random : (random & 0x3) | 0x8;
    return value.toString(16);
  });
}

export default function SearchPage() {
  const [filters, setFilters] = useState<FilterPanelState>(DEFAULT_FILTERS);
  const [sortMode, setSortMode] = useState<SortMode>('Relevance');
  const [activeSearchId, setActiveSearchId] = useState<string>();
  const [shortlistedIds, setShortlistedIds] = useState<Set<string>>(new Set());
  const [sessionDescription, setSessionDescription] = useState('');
  const [sessionMessage, setSessionMessage] = useState<string>();
  const [selectedSessionId, setSelectedSessionId] = useState<string>();

  const searchMutation = useMutation<SearchResponse, Error, SearchRequest>({
    mutationFn: performSearch,
  });
  const saveSessionMutation = useMutation({
    mutationFn: saveSession,
    onSuccess: (session) => {
      setSessionMessage(`Saved session ${session.id}`);
    },
    onError: (error) => {
      setSessionMessage(`Failed to save session: ${error.message}`);
    },
  });
  const selectedSessionQuery = useQuery({
    queryKey: ['session', selectedSessionId],
    queryFn: () => getSession(selectedSessionId as string),
    enabled: Boolean(selectedSessionId),
  });

  const handleSearch = (input: SearchFormInput) => {
    const keywords = input.keywords
      .split(/\s+/)
      .map((keyword) => keyword.trim())
      .filter(Boolean);

    if (keywords.length === 0) {
      return;
    }

    const searchId = randomUuid();
    setActiveSearchId(searchId);
    setSessionMessage(undefined);
    setShortlistedIds(new Set());

    const request: SearchRequest = {
      keywords,
      filters: buildFilterRequest(input.version, filters),
      provider_scope: input.providers,
      sort_mode: sortMode,
      force_refresh: input.forceRefresh,
      search_id: searchId,
    };

    searchMutation.mutate(request);
  };

  const handleToggleShortlist = (modId: string, selected: boolean) => {
    setShortlistedIds((current) => {
      const next = new Set(current);
      if (selected) {
        next.add(modId);
      } else {
        next.delete(modId);
      }
      return next;
    });
  };

  const handleSaveSession = () => {
    if (!searchMutation.data?.search_id) {
      return;
    }

    const selected = shortlistedIds.size
      ? Array.from(shortlistedIds)
      : searchMutation.data.data.slice(0, 3).map((result) => result.id);
    if (selected.length === 0) {
      setSessionMessage('No results available to save.');
      return;
    }

    saveSessionMutation.mutate({
      query_id: searchMutation.data.search_id,
      query_description: sessionDescription || undefined,
      shortlisted_mod_ids: selected,
      comparison_notes: selected.reduce<Record<string, string>>((notes, modId) => {
        notes[modId] = '';
        return notes;
      }, {}),
    });
  };

  const totalResults = searchMutation.data?.metadata.pagination?.total ?? searchMutation.data?.data.length ?? 0;
  const returnedResults = searchMutation.data?.metadata.pagination?.returned ?? searchMutation.data?.data.length ?? 0;

  const modLookup = useMemo(() => {
    const entries = searchMutation.data?.data || [];
    return entries.reduce<Record<string, SearchResponse['data'][number]>>((acc, result) => {
      acc[result.id] = result;
      return acc;
    }, {});
  }, [searchMutation.data]);

  return (
    <main className="search-page">
      <h1>MCM-Finder</h1>
      <p>Search Minecraft mods across multiple providers.</p>

      <SearchForm onSearch={handleSearch} isLoading={searchMutation.isPending} />
      <FilterPanel value={filters} onChange={setFilters} disabled={searchMutation.isPending} />
      <SortSelector value={sortMode} onChange={setSortMode} disabled={searchMutation.isPending} />
      <ProgressIndicator searchId={activeSearchId} active={searchMutation.isPending} />

      {searchMutation.error && <p className="search-page__error">Search failed: {searchMutation.error.message}</p>}

      {searchMutation.data && (
        <>
          <p className="search-page__summary">
            Found {returnedResults} of {totalResults} results in {searchMutation.data.metadata.total_time_ms}ms.
          </p>
          <p className="search-page__summary">Discovery state: {searchMutation.data.discovery_state}</p>
          {searchMutation.data.metadata.pagination?.has_more && (
            <p className="search-page__summary">More results available; refine filters to narrow matches.</p>
          )}
          <DegradedModeNotice providerErrors={searchMutation.data.metadata.provider_errors} />

          <section className="search-form" style={{ marginBottom: '1rem' }}>
            <label htmlFor="session-description">Session description</label>
            <input
              id="session-description"
              type="text"
              placeholder="Optional label for this shortlist"
              value={sessionDescription}
              onChange={(event) => setSessionDescription(event.target.value)}
            />
            <button type="button" onClick={handleSaveSession} disabled={saveSessionMutation.isPending}>
              {saveSessionMutation.isPending ? 'Saving...' : 'Save shortlist session'}
            </button>
            {sessionMessage && <p>{sessionMessage}</p>}
          </section>

          <ResultList
            results={searchMutation.data.data}
            shortlistedIds={shortlistedIds}
            onToggleShortlist={handleToggleShortlist}
          />
        </>
      )}

      <SavedSessionsList onSelectSession={setSelectedSessionId} />
      {selectedSessionQuery.isError && (
        <p className="search-page__error">Failed to load selected session details.</p>
      )}
      <SessionComparison session={selectedSessionQuery.data} modLookup={modLookup} />
    </main>
  );
}
