import React from 'react';
import { ConsolidatedModProfile } from '../services/api';

interface ResultCardProps {
  result: ConsolidatedModProfile;
  shortlisted: boolean;
  onToggleShortlist: (modId: string, selected: boolean) => void;
}

export default function ResultCard({ result, shortlisted, onToggleShortlist }: ResultCardProps) {
  const summary =
    result.provider_records[0]?.summary ||
    Object.values(result.descriptions || {})[0] ||
    'No summary available.';

  const providerLabels = result.provider_records.map((record) => record.source).join(', ');

  return (
    <article className="result-card">
      <h3>{result.canonical_name}</h3>
      <p>{summary}</p>
      <div className="result-card__meta">
        <span>Providers: {providerLabels}</span>
        <span>Downloads: {result.total_downloads.toLocaleString()}</span>
      </div>
      <div className="result-card__meta">
        <span>Versions: {result.all_supported_versions.join(', ') || 'N/A'}</span>
      </div>
      <div className="result-card__meta">
        <span>Loaders: {result.all_supported_loaders.join(', ') || 'N/A'}</span>
      </div>
      <div className="result-card__meta">
        <label>
          <input
            type="checkbox"
            checked={shortlisted}
            onChange={(event) => onToggleShortlist(result.id, event.target.checked)}
          />{' '}
          Shortlist
        </label>
        <a href={`#/mods/${result.id}`}>View details</a>
      </div>
    </article>
  );
}
