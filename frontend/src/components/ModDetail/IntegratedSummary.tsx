import React from 'react';

interface IntegratedSummaryProps {
  summary: string;
}

export default function IntegratedSummary({ summary }: IntegratedSummaryProps) {
  return (
    <section className="mod-detail-section mod-detail-summary">
      <h3>Integrated Summary</h3>
      <p>{summary}</p>
    </section>
  );
}
