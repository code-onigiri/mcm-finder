import React from 'react';
import { ConsolidatedModProfile } from '../services/api';
import ResultCard from './ResultCard';

interface ResultListProps {
  results: ConsolidatedModProfile[];
  shortlistedIds: Set<string>;
  onToggleShortlist: (modId: string, selected: boolean) => void;
}

export default function ResultList({ results, shortlistedIds, onToggleShortlist }: ResultListProps) {
  if (results.length === 0) {
    return <p className="result-list__empty">No results yet.</p>;
  }

  return (
    <section className="result-list">
      {results.map((result) => (
        <ResultCard
          key={result.id}
          result={result}
          shortlisted={shortlistedIds.has(result.id)}
          onToggleShortlist={onToggleShortlist}
        />
      ))}
    </section>
  );
}
