import React, { useEffect, useState } from 'react';
import { ProgressSnapshot, subscribeSearchProgress } from '../services/api';

interface ProgressIndicatorProps {
  searchId?: string;
  active: boolean;
}

export default function ProgressIndicator({ searchId, active }: ProgressIndicatorProps) {
  const [snapshot, setSnapshot] = useState<ProgressSnapshot | null>(null);

  useEffect(() => {
    if (!searchId || !active) {
      return;
    }

    const unsubscribe = subscribeSearchProgress(searchId, setSnapshot);
    return () => unsubscribe();
  }, [searchId, active]);

  if (!active && !snapshot) {
    return null;
  }

  const percentage = snapshot?.percentage ?? 0;
  const stage = snapshot?.current_stage || 'searching_providers';
  const relationships = snapshot?.relationships_discovered ?? 0;

  return (
    <section className="progress-indicator">
      <div className="progress-indicator__header">
        <strong>Search Progress</strong>
        <span>{percentage}%</span>
      </div>
      <div className="progress-indicator__bar">
        <div className="progress-indicator__fill" style={{ width: `${percentage}%` }} />
      </div>
      <p>
        {stage.replace(/_/g, ' ')} · discovered relationships: {relationships}
      </p>
    </section>
  );
}
