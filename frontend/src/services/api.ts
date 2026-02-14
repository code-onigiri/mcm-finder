import axios from 'axios';

export type SortMode = 'Relevance' | 'UpdateRecencyVersionAware' | 'Downloads' | 'CreatedDate';
export type DiscoveryState = 'pending' | 'processing' | 'complete';

export interface SearchFilters {
  minecraft_version?: string;
  loaders?: string[];
  categories?: string[];
  update_recency_window_days?: number;
  min_downloads?: number;
  open_source_only?: boolean;
}

export interface SearchRequest {
  keywords: string[];
  filters?: SearchFilters;
  provider_scope?: string[];
  sort_mode?: SortMode;
  offset?: number;
  limit?: number;
  force_refresh?: boolean;
  search_id?: string;
}

export interface ProviderRecord {
  source: string;
  provider_mod_id: string;
  provider_url: string;
  name: string;
  summary: string;
  downloads: number;
  followers?: number | null;
  supported_versions: string[];
  supported_loaders: string[];
  categories: string[];
  license?: string | null;
  source_url?: string | null;
  relevance_score: number;
}

export interface ConsolidatedModProfile {
  id: string;
  canonical_name: string;
  canonical_slug: string;
  provider_records: ProviderRecord[];
  authors: string[];
  descriptions: Record<string, string>;
  all_supported_versions: string[];
  all_supported_loaders: string[];
  total_downloads: number;
  max_followers?: number;
  earliest_created?: string;
  most_recent_update?: string;
  metadata_conflicts: MetadataConflict[];
  discovery_evidence: DiscoveryEvidenceItem[];
  compatibility_assessment: CompatibilityAssessment;
  maintenance_signals: MaintenanceSignals;
  adoption_risks: RiskFactor[];
  composite_relevance: number;
  confidence_score: number;
}

export interface MetadataConflict {
  field: string;
  values: Record<string, string>;
  severity: 'Minor' | 'Major' | 'Critical' | string;
}

export interface DiscoveryEvidenceItem {
  id: string;
  relationship_type: string;
  related_mod_name: string;
  evidence_source: string;
  confidence: number;
  discovered_at: string;
  context: string;
  metadata?: Record<string, unknown>;
}

export type CompatibilityAssessment =
  | 'FullyCompatible'
  | { PartiallyCompatible: { issues: string[] } }
  | { Incompatible: { reasons: string[] } }
  | string;

export type MaintenanceSignals =
  | 'ActivelyMaintained'
  | 'Abandoned'
  | 'Unknown'
  | { Maintenance: { last_update_days: number } }
  | string;

export interface RiskFactor {
  risk_type: string;
  description: string;
  severity: 'Low' | 'Medium' | 'High' | string;
}

export interface ProviderErrorMetadata {
  provider: string;
  error: string;
  severity: string;
  reason: string;
  timestamp: number;
}

export interface PaginationMetadata {
  offset: number;
  limit: number;
  returned: number;
  total: number;
  has_more: boolean;
}

export interface SearchMetadata {
  total_time_ms: number;
  providers_queried: string[];
  providers_succeeded: string[];
  provider_errors: ProviderErrorMetadata[];
  degradation_reason?: string;
  sort_mode?: SortMode;
  pre_filter_results?: number;
  post_filter_results?: number;
  pagination?: PaginationMetadata;
  cached_at?: string;
  expires_at?: string;
  cache_version?: string;
}

export interface SearchResponse {
  search_id: string;
  discovery_state: DiscoveryState;
  data: ConsolidatedModProfile[];
  metadata: SearchMetadata;
}

export interface ProgressSnapshot {
  search_id: string;
  discovery_state: DiscoveryState;
  phase: string;
  percentage: number;
  current_stage: string;
  mods_processed: number;
  total_mods: number;
  relationships_discovered: number;
  started_at_unix: number;
  updated_at_unix: number;
}

export type ModDetailsResponse = ConsolidatedModProfile & {
  integrated_summary: string;
  discovery_relationship_count: number;
  metadata_conflict_count: number;
};

export interface SaveSessionRequest {
  query_id: string;
  query_description?: string;
  shortlisted_mod_ids: string[];
  comparison_notes?: Record<string, string>;
  user_identifier?: string;
}

export interface SessionListResponse {
  data: SearchSessionSummary[];
}

export interface SearchSessionSummary {
  id: string;
  created_at: string;
  updated_at: string;
  expires_at?: string;
  original_query: {
    id: string;
    created_at: string;
    keywords: string[];
    filters: SearchFilters;
    provider_scope: string[];
    sort_mode: SortMode;
  };
  query_description?: string;
  shortlisted_mods: string[];
  comparison_notes: Record<string, string>;
  search_duration_ms: number;
  providers_used: string[];
  total_candidates_found: number;
  shareable_link?: string;
  user_identifier?: string;
}

const API_BASE_URL = process.env.REACT_APP_API_BASE_URL || 'http://localhost:3000';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 15000,
});

export async function searchMods(request: SearchRequest): Promise<SearchResponse> {
  const response = await apiClient.post<SearchResponse>('/api/v1/search', request);
  return response.data;
}

export async function getModDetails(modId: string): Promise<ModDetailsResponse> {
  const response = await apiClient.get<ModDetailsResponse>(`/api/v1/mods/${modId}`);
  return response.data;
}

export async function saveSession(request: SaveSessionRequest): Promise<SearchSessionSummary> {
  const response = await apiClient.post<SearchSessionSummary>('/api/v1/sessions', request);
  return response.data;
}

export async function getSession(sessionId: string): Promise<SearchSessionSummary> {
  const response = await apiClient.get<SearchSessionSummary>(`/api/v1/sessions/${sessionId}`);
  return response.data;
}

export async function listSessions(userIdentifier?: string): Promise<SearchSessionSummary[]> {
  const response = await apiClient.get<SessionListResponse>('/api/v1/sessions', {
    params: userIdentifier ? { user_identifier: userIdentifier } : undefined,
  });
  return response.data.data;
}

export function subscribeSearchProgress(
  searchId: string,
  onProgress: (progress: ProgressSnapshot) => void,
  onError?: (error: Event) => void
): () => void {
  const source = new EventSource(`${API_BASE_URL}/api/v1/search/${searchId}/progress`);
  const handler = (event: MessageEvent<string>) => {
    try {
      onProgress(JSON.parse(event.data) as ProgressSnapshot);
    } catch {
      // Ignore malformed payloads from interrupted streams.
    }
  };
  source.addEventListener('progress', handler as EventListener);
  source.onerror = (event) => {
    onError?.(event);
  };
  return () => {
    source.removeEventListener('progress', handler as EventListener);
    source.close();
  };
}
