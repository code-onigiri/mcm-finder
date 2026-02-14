import React from 'react';
import { MaintenanceSignals } from '../../services/api';

interface MaintenanceSectionProps {
  maintenance: MaintenanceSignals;
  mostRecentUpdate?: string;
}

export default function MaintenanceSection({ maintenance, mostRecentUpdate }: MaintenanceSectionProps) {
  const details = formatMaintenance(maintenance);

  return (
    <section className="mod-detail-section">
      <h3>Maintenance</h3>
      <p>{details.label}</p>
      {mostRecentUpdate && <p>Last known update: {new Date(mostRecentUpdate).toLocaleDateString()}</p>}
      {details.lastUpdateDays !== undefined && <p>Recency: {details.lastUpdateDays} days ago</p>}
    </section>
  );
}

function formatMaintenance(maintenance: MaintenanceSignals): { label: string; lastUpdateDays?: number } {
  if (typeof maintenance === 'string') {
    return { label: maintenance };
  }

  if ('Maintenance' in maintenance) {
    return {
      label: 'Maintenance mode',
      lastUpdateDays: maintenance.Maintenance.last_update_days,
    };
  }

  return { label: 'Unknown' };
}
