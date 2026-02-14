import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { listSessions } from '../services/api';

interface SavedSessionsListProps {
  onSelectSession: (sessionId: string) => void;
}

export default function SavedSessionsList({ onSelectSession }: SavedSessionsListProps) {
  const sessionsQuery = useQuery({
    queryKey: ['saved-sessions'],
    queryFn: () => listSessions(),
  });

  if (sessionsQuery.isPending) {
    return <p>Loading saved sessions...</p>;
  }

  if (sessionsQuery.isError) {
    return <p className="search-page__error">Unable to load saved sessions.</p>;
  }

  if (!sessionsQuery.data || sessionsQuery.data.length === 0) {
    return <p>No saved sessions yet.</p>;
  }

  return (
    <section className="saved-sessions-list">
      <h3>Saved Sessions</h3>
      <ul>
        {sessionsQuery.data.map((session) => (
          <li key={session.id}>
            <button type="button" onClick={() => onSelectSession(session.id)}>
              {session.query_description || session.original_query.keywords.join(' ')} ·{' '}
              {new Date(session.updated_at).toLocaleString()}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
