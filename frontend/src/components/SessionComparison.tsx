import React from 'react';
import { ConsolidatedModProfile, SearchSessionSummary } from '../services/api';

interface SessionComparisonProps {
  session?: SearchSessionSummary;
  modLookup: Record<string, ConsolidatedModProfile>;
}

export default function SessionComparison({ session, modLookup }: SessionComparisonProps) {
  if (!session) {
    return null;
  }

  return (
    <section className="session-comparison">
      <h3>Session Comparison</h3>
      <p>{session.query_description || 'Saved shortlist'}</p>
      <div className="session-comparison__grid">
        {session.shortlisted_mods.map((modId) => {
          const mod = modLookup[modId];
          const note = session.comparison_notes[modId];
          return (
            <article key={modId} className="session-comparison__card">
              <h4>{mod?.canonical_name || modId}</h4>
              {mod && <p>Composite relevance: {mod.composite_relevance.toFixed(2)}</p>}
              <p>{note || 'No note'}</p>
            </article>
          );
        })}
      </div>
    </section>
  );
}
